//! Instance/device/surface ownership. Created and destroyed on the window thread.
use super::Result;
use ash::{Entry, vk};
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use std::ffi::CStr;
use winit::window::Window;

pub(super) struct Context {
    _entry: Entry,
    pub instance: ash::Instance,
    pub surface_api: ash::khr::surface::Instance,
    pub surface: vk::SurfaceKHR,
    pub physical: vk::PhysicalDevice,
    device: Option<ash::Device>,
    pub queue: vk::Queue,
    pub family: u32,
    pub pool: vk::CommandPool,
}

impl Context {
    pub fn new(window: &Window) -> Result<Self> {
        // SAFETY: loader is retained until after all objects/function pointers are destroyed.
        let entry = unsafe { Entry::load()? };
        let display = window.display_handle()?.as_raw();
        let handle = window.window_handle()?.as_raw();
        let extensions = ash_window::enumerate_required_extensions(display)?;
        let info = vk::ApplicationInfo::default()
            .application_name(c"Astero GUI")
            .api_version(vk::API_VERSION_1_0);
        // SAFETY: borrowed names/extensions live through creation, with no dangling pointers.
        let instance = unsafe {
            entry.create_instance(
                &vk::InstanceCreateInfo::default()
                    .application_info(&info)
                    .enabled_extension_names(extensions),
                None,
            )?
        };
        let surface_api = ash::khr::surface::Instance::new(&entry, &instance);
        let mut ctx = Self {
            _entry: entry,
            instance,
            surface_api,
            surface: vk::SurfaceKHR::null(),
            physical: vk::PhysicalDevice::null(),
            device: None,
            queue: vk::Queue::null(),
            family: 0,
            pool: vk::CommandPool::null(),
        };
        // SAFETY: the owning LiveWindow keeps the window alive beyond this context.
        unsafe {
            ctx.surface =
                ash_window::create_surface(&ctx._entry, &ctx.instance, display, handle, None)?;
            let mut selected = None;
            for physical in ctx.instance.enumerate_physical_devices()? {
                let has_swapchain = ctx
                    .instance
                    .enumerate_device_extension_properties(physical)?
                    .iter()
                    .any(|ext| {
                        CStr::from_ptr(ext.extension_name.as_ptr()) == ash::khr::swapchain::NAME
                    });
                if !has_swapchain {
                    continue;
                }
                for (index, props) in ctx
                    .instance
                    .get_physical_device_queue_family_properties(physical)
                    .iter()
                    .enumerate()
                {
                    if props.queue_count > 0
                        && props.queue_flags.contains(vk::QueueFlags::GRAPHICS)
                        && ctx.surface_api.get_physical_device_surface_support(
                            physical,
                            index as u32,
                            ctx.surface,
                        )?
                    {
                        selected = Some((physical, index as u32));
                        break;
                    }
                }
                if selected.is_some() {
                    break;
                }
            }
            let (physical, family) = selected
                .ok_or("No Vulkan device with graphics/present queue and VK_KHR_swapchain")?;
            ctx.physical = physical;
            ctx.family = family;
            let priorities = [1.0];
            let queues = [vk::DeviceQueueCreateInfo::default()
                .queue_family_index(family)
                .queue_priorities(&priorities)];
            let device_extensions = [ash::khr::swapchain::NAME.as_ptr()];
            ctx.device = Some(
                ctx.instance.create_device(
                    physical,
                    &vk::DeviceCreateInfo::default()
                        .queue_create_infos(&queues)
                        .enabled_extension_names(&device_extensions),
                    None,
                )?,
            );
            ctx.queue = ctx.device().get_device_queue(family, 0);
            ctx.pool = ctx.device().create_command_pool(
                &vk::CommandPoolCreateInfo::default()
                    .queue_family_index(family)
                    .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER),
                None,
            )?;
            let props = ctx.instance.get_physical_device_properties(physical);
            eprintln!(
                "GUI Vulkan device: {}",
                CStr::from_ptr(props.device_name.as_ptr()).to_string_lossy()
            );
        }
        Ok(ctx)
    }

    pub fn device(&self) -> &ash::Device {
        self.device
            .as_ref()
            .expect("device exists after successful Context::new")
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        // SAFETY: child renderer/swapchain resources were dropped first; partial creation uses nulls/None.
        unsafe {
            if let Some(device) = self.device.take() {
                let _ = device.device_wait_idle();
                device.destroy_command_pool(self.pool, None);
                device.destroy_device(None);
            }
            self.surface_api.destroy_surface(self.surface, None);
            self.instance.destroy_instance(None);
        }
    }
}
