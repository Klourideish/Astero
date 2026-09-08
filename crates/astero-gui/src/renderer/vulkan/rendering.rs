//! Single-frame GUI rendering. Owns all synchronization and swapchain recreation.
use super::{Result, context::Context, swapchain::Swapchain};
use ash::vk;
use imgui_rs_vulkan_renderer::{Options, Renderer};
use winit::window::Window;

pub(crate) struct Graphics {
    context: Context,
    renderer: Option<Renderer>,
    swap: Option<Swapchain>,
    command: vk::CommandBuffer,
    acquired: vk::Semaphore,
    fence: vk::Fence,
    dirty: bool,
    presented: bool,
}

impl Graphics {
    pub fn new(window: &Window, imgui: &mut imgui::Context) -> Result<Self> {
        let context = Context::new(window)?;
        let mut graphics = Self {
            context,
            renderer: None,
            swap: None,
            command: vk::CommandBuffer::null(),
            acquired: vk::Semaphore::null(),
            fence: vk::Fence::null(),
            dirty: true,
            presented: false,
        };
        let device = graphics.context.device();
        // SAFETY: the pool belongs to this device/queue family, resources remain exclusively owned here.
        unsafe {
            graphics.command = device.allocate_command_buffers(
                &vk::CommandBufferAllocateInfo::default()
                    .command_pool(graphics.context.pool)
                    .level(vk::CommandBufferLevel::PRIMARY)
                    .command_buffer_count(1),
            )?[0];
            graphics.acquired =
                device.create_semaphore(&vk::SemaphoreCreateInfo::default(), None)?;
            graphics.fence = device.create_fence(
                &vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED),
                None,
            )?;
        }
        graphics.prepare(window, imgui)?;
        Ok(graphics)
    }

    pub fn resize(&mut self) {
        self.dirty = true;
    }

    /// Rebuild only before creating ImGui's frame, so font upload cannot invalidate draw data.
    pub fn prepare(&mut self, window: &Window, imgui: &mut imgui::Context) -> Result<bool> {
        let size = window.inner_size();
        if size.width == 0 || size.height == 0 {
            return Ok(false);
        }
        if self.dirty {
            // SAFETY: all submissions are on our queue; wait before destroying referenced objects.
            unsafe {
                self.context.device().device_wait_idle()?;
            }
            self.renderer.take();
            self.swap.take();
            self.swap = Some(Swapchain::new(&self.context, [size.width, size.height])?);
            let swap = self.swap.as_ref().expect("just created");
            self.renderer = Some(Renderer::with_default_allocator(
                &self.context.instance,
                self.context.physical,
                self.context.device().clone(),
                self.context.queue,
                self.context.pool,
                swap.pass,
                imgui,
                Some(Options {
                    in_flight_frames: 1,
                    ..Default::default()
                }),
            )?);
            self.dirty = false;
        }
        Ok(true)
    }

    pub fn draw(&mut self, data: &imgui::DrawData) -> Result<()> {
        let swap = self.swap.as_ref().ok_or("GUI swapchain unavailable")?;
        let device = self.context.device();
        // SAFETY: exactly one frame is in flight. Fence completion protects the command buffer,
        // acquire semaphore and renderer buffers. Finished semaphores belong to swapchain images.
        unsafe {
            device.wait_for_fences(&[self.fence], true, 5_000_000_000)?;
            let (index, suboptimal) = match swap.api.acquire_next_image(
                swap.handle,
                1_000_000_000,
                self.acquired,
                vk::Fence::null(),
            ) {
                Ok(result) => result,
                Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                    self.dirty = true;
                    return Ok(());
                }
                Err(vk::Result::TIMEOUT | vk::Result::NOT_READY) => return Ok(()),
                Err(error) => return Err(error.into()),
            };
            self.dirty |= suboptimal;
            device.reset_command_buffer(self.command, vk::CommandBufferResetFlags::empty())?;
            device.begin_command_buffer(
                self.command,
                &vk::CommandBufferBeginInfo::default()
                    .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT),
            )?;
            let clear = [vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.08, 0.09, 0.11, 1.0],
                },
            }];
            device.cmd_begin_render_pass(
                self.command,
                &vk::RenderPassBeginInfo::default()
                    .render_pass(swap.pass)
                    .framebuffer(swap.frames[index as usize])
                    .render_area(vk::Rect2D {
                        offset: vk::Offset2D::default(),
                        extent: swap.extent,
                    })
                    .clear_values(&clear),
                vk::SubpassContents::INLINE,
            );
            self.renderer
                .as_mut()
                .ok_or("ImGui renderer unavailable")?
                .cmd_draw(self.command, data)?;
            device.cmd_end_render_pass(self.command);
            device.end_command_buffer(self.command)?;
            let waits = [self.acquired];
            let stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
            let commands = [self.command];
            let signals = [swap.finished[index as usize]];
            let submits = [vk::SubmitInfo::default()
                .wait_semaphores(&waits)
                .wait_dst_stage_mask(&stages)
                .command_buffers(&commands)
                .signal_semaphores(&signals)];
            // Reset only when a submission is ready. A submission failure exits the GUI rather than waiting again.
            device.reset_fences(&[self.fence])?;
            device.queue_submit(self.context.queue, &submits, self.fence)?;
            let handles = [swap.handle];
            let indices = [index];
            match swap.api.queue_present(
                self.context.queue,
                &vk::PresentInfoKHR::default()
                    .wait_semaphores(&signals)
                    .swapchains(&handles)
                    .image_indices(&indices),
            ) {
                Ok(suboptimal) => {
                    self.dirty |= suboptimal;
                    if !self.presented {
                        eprintln!("GUI Vulkan: first frame presented");
                        self.presented = true;
                    }
                }
                Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => self.dirty = true,
                Err(error) => return Err(error.into()),
            }
        }
        Ok(())
    }
}

impl Drop for Graphics {
    fn drop(&mut self) {
        // SAFETY: idle before renderer/swapchain/sync destruction. Context then drops the pool/device/instance.
        unsafe {
            let device = self.context.device();
            let _ = device.device_wait_idle();
            self.renderer.take();
            self.swap.take();
            device.destroy_semaphore(self.acquired, None);
            device.destroy_fence(self.fence, None);
        }
    }
}
