use crate::error::{Result, VulkanError};
use ash::extensions::ext::DebugUtils;
use ash::extensions::khr;
use ash::vk;
use ash::{Device, Entry, Instance};
use log::{debug, error};
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use std::ffi::CStr;
use std::sync::Arc;

pub struct VulkanContext {
    _entry: Entry,
    instance: Instance,
    debug_utils: Option<(DebugUtils, vk::DebugUtilsMessengerEXT)>,
    surface_loader: khr::Surface,
    surface: vk::SurfaceKHR,
    physical_device: vk::PhysicalDevice,
    device: Device,
    graphics_queue: vk::Queue,
    present_queue: vk::Queue,
}

impl VulkanContext {
    pub fn new<W: HasRawDisplayHandle + HasRawWindowHandle>(window: &W) -> Result<Self> {
        // Load Vulkan entry
        let entry =
            unsafe { Entry::load() }.map_err(|e| VulkanError::InstanceCreation(e.to_string()))?;

        // Create instance with validation layers
        let instance = Self::create_instance(&entry)?;

        // Setup debug messenger if validation layers are enabled
        let debug_utils = Self::setup_debug_messenger(&entry, &instance)?;

        // Create surface
        let (surface_loader, surface) = Self::create_surface(&entry, &instance, window)?;

        // Select physical device
        let physical_device = Self::pick_physical_device(&instance, &surface_loader, surface)?;

        // Create logical device and queues
        let (device, graphics_queue, present_queue) =
            Self::create_logical_device(&instance, physical_device, &surface_loader, surface)?;

        Ok(Self {
            _entry: entry,
            instance,
            debug_utils,
            surface_loader,
            surface,
            physical_device,
            device,
            graphics_queue,
            present_queue,
        })
    }

    fn create_instance(entry: &Entry) -> Result<Instance> {
        let app_info = vk::ApplicationInfo::builder()
            .application_name(CStr::from_bytes_with_nul(b"Vulkan Engine\0").unwrap())
            .application_version(vk::make_api_version(0, 1, 0, 0))
            .engine_name(CStr::from_bytes_with_nul(b"No Engine\0").unwrap())
            .engine_version(vk::make_api_version(0, 1, 0, 0))
            .api_version(vk::API_VERSION_1_0);

        let layer_names = [CStr::from_bytes_with_nul(b"VK_LAYER_KHRONOS_validation\0").unwrap()];
        let layers_names_raw: Vec<*const i8> = layer_names
            .iter()
            .map(|raw_name| raw_name.as_ptr())
            .collect();

        let extension_names_raw = vec![
            ash::extensions::ext::DebugUtils::name().as_ptr(),
            ash::extensions::khr::Surface::name().as_ptr(),
            #[cfg(target_os = "windows")]
            ash::extensions::khr::Win32Surface::name().as_ptr(),
            #[cfg(target_os = "linux")]
            ash::extensions::khr::XcbSurface::name().as_ptr(),
            #[cfg(target_os = "linux")]
            ash::extensions::khr::XlibSurface::name().as_ptr(),
            #[cfg(target_os = "linux")]
            ash::extensions::khr::WaylandSurface::name().as_ptr(),
        ];

        let create_info = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_layer_names(&layers_names_raw)
            .enabled_extension_names(&extension_names_raw);

        unsafe {
            entry
                .create_instance(&create_info, None)
                .map_err(|e| VulkanError::InstanceCreation(e.to_string()))
        }
    }

    fn create_surface<W: HasRawDisplayHandle + HasRawWindowHandle>(
        entry: &Entry,
        instance: &Instance,
        window: &W,
    ) -> Result<(khr::Surface, vk::SurfaceKHR)> {
        let surface = unsafe {
            ash_window::create_surface(
                entry,
                instance,
                window.raw_display_handle(),
                window.raw_window_handle(),
                None,
            )
            .map_err(|e| VulkanError::SurfaceCreation(e.to_string()))?
        };

        let surface_loader = khr::Surface::new(entry, instance);

        Ok((surface_loader, surface))
    }

    fn pick_physical_device(
        instance: &Instance,
        surface_loader: &khr::Surface,
        surface: vk::SurfaceKHR,
    ) -> Result<vk::PhysicalDevice> {
        let devices = unsafe {
            instance
                .enumerate_physical_devices()
                .map_err(|e| VulkanError::DeviceCreation(e.to_string()))?
        };

        let physical_device = devices
            .into_iter()
            .find(|&device| unsafe {
                Self::is_device_suitable(instance, surface_loader, surface, device)
            })
            .ok_or(VulkanError::NoSuitableGpu)?;

        Ok(physical_device)
    }

    unsafe fn is_device_suitable(
        instance: &Instance,
        surface_loader: &khr::Surface,
        surface: vk::SurfaceKHR,
        device: vk::PhysicalDevice,
    ) -> bool {
        let queue_families = instance.get_physical_device_queue_family_properties(device);

        let mut graphics_family = None;
        let mut present_family = None;

        for (i, queue_family) in queue_families.iter().enumerate() {
            if queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                graphics_family = Some(i as u32);
            }

            let present_support = surface_loader
                .get_physical_device_surface_support(device, i as u32, surface)
                .unwrap_or(false);

            if present_support {
                present_family = Some(i as u32);
            }

            if graphics_family.is_some() && present_family.is_some() {
                break;
            }
        }

        graphics_family.is_some() && present_family.is_some()
    }

    fn create_logical_device(
        instance: &Instance,
        physical_device: vk::PhysicalDevice,
        surface_loader: &khr::Surface,
        surface: vk::SurfaceKHR,
    ) -> Result<(Device, vk::Queue, vk::Queue)> {
        let queue_families =
            unsafe { instance.get_physical_device_queue_family_properties(physical_device) };

        let mut graphics_family = None;
        let mut present_family = None;

        for (i, queue_family) in queue_families.iter().enumerate() {
            if queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                graphics_family = Some(i as u32);
            }

            let present_support = unsafe {
                surface_loader
                    .get_physical_device_surface_support(physical_device, i as u32, surface)
                    .unwrap_or(false)
            };

            if present_support {
                present_family = Some(i as u32);
            }

            if graphics_family.is_some() && present_family.is_some() {
                break;
            }
        }

        let graphics_family = graphics_family.ok_or(VulkanError::NoSuitableGpu)?;
        let present_family = present_family.ok_or(VulkanError::NoSuitableGpu)?;

        let queue_priorities = [1.0];
        let queue_create_info = vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(graphics_family)
            .queue_priorities(&queue_priorities);

        let device_extensions = [ash::extensions::khr::Swapchain::name().as_ptr()];
        let features = vk::PhysicalDeviceFeatures::builder().build();

        let create_info = vk::DeviceCreateInfo::builder()
            .queue_create_infos(std::slice::from_ref(&queue_create_info))
            .enabled_extension_names(&device_extensions)
            .enabled_features(&features);

        let device = unsafe {
            instance
                .create_device(physical_device, &create_info, None)
                .map_err(|e| VulkanError::DeviceCreation(e.to_string()))?
        };

        let graphics_queue = unsafe { device.get_device_queue(graphics_family, 0) };
        let present_queue = unsafe { device.get_device_queue(present_family, 0) };

        Ok((device, graphics_queue, present_queue))
    }

    fn setup_debug_messenger(
        entry: &Entry,
        instance: &Instance,
    ) -> Result<Option<(DebugUtils, vk::DebugUtilsMessengerEXT)>> {
        let debug_utils = DebugUtils::new(entry, instance);

        let messenger_ci = vk::DebugUtilsMessengerCreateInfoEXT::builder()
            .message_severity(
                vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                    | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
            )
            .message_type(
                vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                    | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                    | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
            )
            .pfn_user_callback(Some(Self::debug_callback));

        let messenger = unsafe {
            debug_utils
                .create_debug_utils_messenger(&messenger_ci, None)
                .map_err(|e| VulkanError::InstanceCreation(e.to_string()))?
        };

        Ok(Some((debug_utils, messenger)))
    }

    unsafe extern "system" fn debug_callback(
        message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
        message_type: vk::DebugUtilsMessageTypeFlagsEXT,
        p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
        _user_data: *mut std::ffi::c_void,
    ) -> vk::Bool32 {
        let callback_data = *p_callback_data;
        let message = match CStr::from_ptr(callback_data.p_message).to_str() {
            Ok(message) => message.to_string(),
            Err(_) => String::from("[Invalid UTF-8 in validation message]"),
        };

        match message_severity {
            vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => {
                error!("{:?}: {}", message_type, message)
            }
            _ => debug!("{:?}: {}", message_type, message),
        };

        vk::FALSE
    }

    pub fn instance(&self) -> &Instance {
        &self.instance
    }

    pub fn physical_device(&self) -> vk::PhysicalDevice {
        self.physical_device
    }

    pub fn device(&self) -> &Device {
        &self.device
    }

    pub fn surface_loader(&self) -> &khr::Surface {
        &self.surface_loader
    }

    pub fn surface(&self) -> vk::SurfaceKHR {
        self.surface
    }

    pub fn graphics_queue(&self) -> vk::Queue {
        self.graphics_queue
    }

    pub fn present_queue(&self) -> vk::Queue {
        self.present_queue
    }

    pub fn wait_idle(&self) -> Result<()> {
        unsafe {
            self.device
                .device_wait_idle()
                .map_err(|e| VulkanError::SyncError(e.to_string()))?;
        }
        Ok(())
    }

    fn cleanup(&mut self) {
        if let Ok(_) = self.wait_idle() {
            unsafe {
                if let Some((debug_utils, messenger)) = self.debug_utils.take() {
                    debug_utils.destroy_debug_utils_messenger(messenger, None);
                }
                self.device.destroy_device(None);
                self.surface_loader.destroy_surface(self.surface, None);
                self.instance.destroy_instance(None);
            }
        }
    }
}

impl Drop for VulkanContext {
    fn drop(&mut self) {
        self.cleanup();
    }
}
