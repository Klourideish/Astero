//! Window-size-dependent GUI presentation objects; no emulated GPU resources.
use super::{Result, context::Context};
use ash::vk;

pub(super) struct Swapchain {
    device: ash::Device,
    pub api: ash::khr::swapchain::Device,
    pub handle: vk::SwapchainKHR,
    pub extent: vk::Extent2D,
    views: Vec<vk::ImageView>,
    pub pass: vk::RenderPass,
    pub frames: Vec<vk::Framebuffer>,
    // Per-image semaphores: reacquisition guarantees the previous present wait completed.
    pub finished: Vec<vk::Semaphore>,
}

pub(super) fn choose_extent(
    caps: &vk::SurfaceCapabilitiesKHR,
    requested: [u32; 2],
) -> vk::Extent2D {
    if caps.current_extent.width != u32::MAX {
        return caps.current_extent;
    }
    vk::Extent2D {
        width: requested[0].clamp(caps.min_image_extent.width, caps.max_image_extent.width),
        height: requested[1].clamp(caps.min_image_extent.height, caps.max_image_extent.height),
    }
}

impl Swapchain {
    pub fn new(ctx: &Context, requested: [u32; 2]) -> Result<Self> {
        let device = ctx.device();
        // SAFETY: all queried handles belong to the live context. The caller skips zero-size windows.
        unsafe {
            let caps = ctx
                .surface_api
                .get_physical_device_surface_capabilities(ctx.physical, ctx.surface)?;
            let extent = choose_extent(&caps, requested);
            if extent.width == 0 || extent.height == 0 {
                return Err("Surface has zero extent".into());
            }
            if !caps
                .supported_usage_flags
                .contains(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            {
                return Err("Surface cannot be a color attachment".into());
            }
            let formats = ctx
                .surface_api
                .get_physical_device_surface_formats(ctx.physical, ctx.surface)?;
            let mut format = formats
                .iter()
                .find(|f| {
                    f.format == vk::Format::B8G8R8A8_UNORM
                        && f.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
                })
                .or_else(|| formats.first())
                .copied()
                .ok_or("Surface has no formats")?;
            if format.format == vk::Format::UNDEFINED {
                format.format = vk::Format::B8G8R8A8_UNORM;
            }
            let alpha = [
                vk::CompositeAlphaFlagsKHR::OPAQUE,
                vk::CompositeAlphaFlagsKHR::PRE_MULTIPLIED,
                vk::CompositeAlphaFlagsKHR::POST_MULTIPLIED,
                vk::CompositeAlphaFlagsKHR::INHERIT,
            ]
            .into_iter()
            .find(|a| caps.supported_composite_alpha.contains(*a))
            .ok_or("No composite alpha mode")?;
            let desired = caps.min_image_count.saturating_add(1);
            let count = if caps.max_image_count == 0 {
                desired
            } else {
                desired.min(caps.max_image_count)
            };
            let api = ash::khr::swapchain::Device::new(&ctx.instance, device);
            let mut swap = Self {
                device: device.clone(),
                api,
                handle: vk::SwapchainKHR::null(),
                extent,
                views: vec![],
                pass: vk::RenderPass::null(),
                frames: vec![],
                finished: vec![],
            };
            swap.handle = swap.api.create_swapchain(
                &vk::SwapchainCreateInfoKHR::default()
                    .surface(ctx.surface)
                    .min_image_count(count)
                    .image_format(format.format)
                    .image_color_space(format.color_space)
                    .image_extent(extent)
                    .image_array_layers(1)
                    .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
                    .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
                    .pre_transform(caps.current_transform)
                    .composite_alpha(alpha)
                    .present_mode(vk::PresentModeKHR::FIFO)
                    .clipped(true),
                None,
            )?;
            let attachments = [vk::AttachmentDescription::default()
                .format(format.format)
                .samples(vk::SampleCountFlags::TYPE_1)
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::STORE)
                .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
                .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
                .initial_layout(vk::ImageLayout::UNDEFINED)
                .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)];
            let colors = [vk::AttachmentReference::default()
                .attachment(0)
                .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)];
            let subpasses = [vk::SubpassDescription::default()
                .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
                .color_attachments(&colors)];
            let dependencies = [vk::SubpassDependency::default()
                .src_subpass(vk::SUBPASS_EXTERNAL)
                .dst_subpass(0)
                .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
                .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
                .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)];
            swap.pass = device.create_render_pass(
                &vk::RenderPassCreateInfo::default()
                    .attachments(&attachments)
                    .subpasses(&subpasses)
                    .dependencies(&dependencies),
                None,
            )?;
            for image in swap.api.get_swapchain_images(swap.handle)? {
                let view = device.create_image_view(
                    &vk::ImageViewCreateInfo::default()
                        .image(image)
                        .view_type(vk::ImageViewType::TYPE_2D)
                        .format(format.format)
                        .subresource_range(
                            vk::ImageSubresourceRange::default()
                                .aspect_mask(vk::ImageAspectFlags::COLOR)
                                .level_count(1)
                                .layer_count(1),
                        ),
                    None,
                )?;
                swap.views.push(view);
                swap.frames.push(
                    device.create_framebuffer(
                        &vk::FramebufferCreateInfo::default()
                            .render_pass(swap.pass)
                            .attachments(&[view])
                            .width(extent.width)
                            .height(extent.height)
                            .layers(1),
                        None,
                    )?,
                );
                swap.finished
                    .push(device.create_semaphore(&vk::SemaphoreCreateInfo::default(), None)?);
            }
            Ok(swap)
        }
    }
}

impl Drop for Swapchain {
    fn drop(&mut self) {
        // SAFETY: caller waited for device idle; children precede parents, including partial initialization.
        unsafe {
            for semaphore in self.finished.drain(..) {
                self.device.destroy_semaphore(semaphore, None);
            }
            for frame in self.frames.drain(..) {
                self.device.destroy_framebuffer(frame, None);
            }
            for view in self.views.drain(..) {
                self.device.destroy_image_view(view, None);
            }
            self.device.destroy_render_pass(self.pass, None);
            self.api.destroy_swapchain(self.handle, None);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn respects_fixed_surface_extent() {
        let caps = vk::SurfaceCapabilitiesKHR {
            current_extent: vk::Extent2D {
                width: 640,
                height: 480,
            },
            ..Default::default()
        };
        assert_eq!(choose_extent(&caps, [1200, 900]), caps.current_extent);
    }
    #[test]
    fn clamps_variable_surface_extent() {
        let caps = vk::SurfaceCapabilitiesKHR {
            current_extent: vk::Extent2D {
                width: u32::MAX,
                height: u32::MAX,
            },
            min_image_extent: vk::Extent2D {
                width: 100,
                height: 100,
            },
            max_image_extent: vk::Extent2D {
                width: 1000,
                height: 800,
            },
            ..Default::default()
        };
        assert_eq!(
            choose_extent(&caps, [10, 900]),
            vk::Extent2D {
                width: 100,
                height: 800
            }
        );
    }
}
