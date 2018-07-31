#![feature(use_extern_macros)]
extern crate ash;
extern crate winit;
extern crate winapi;
extern crate ash_samples;

use std::ffi::CString;
use ash::vk;
use std::ptr;
use std::default::Default;
use ash::Entry;
use ash::Instance;
use ash::Device;
use ash::version::{DeviceV1_0, EntryV1_0, InstanceV1_0, V1_0};
use ash::extensions::{Swapchain, Surface, Win32Surface, XlibSurface, DebugReport};

// please look at ash-tutorial.pdf for further information!

fn main() {
    unsafe {
        let (entry, instance): (Entry<V1_0>, Instance<V1_0>) =
            init_instance_with_extensions("init-swap-chain-sample");

        let window_width: u32 = 500;
        let window_height: u32 = 500;

        let (events_loop, window) =
            create_events_loop_and_window(window_width, window_height);

        let surface: vk::SurfaceKHR = match create_surface(&entry, &instance, &window) {
            Ok(surface) => surface,
            Err(error) => ash_samples::clean_up_and_panic(&format!("Could not create surface: {:?}", error),
                                             instance, None, Vec::<vk::types::CommandPool>::new()),
        };

        let pdevices = match instance.enumerate_physical_devices() {
            Ok(pdevices) => pdevices,
            Err(error) => ash_samples::clean_up_and_panic(&format!("Physical device error: {:?}", error),
                                             instance, None, Vec::<vk::types::CommandPool>::new()),
        };

        let surface_extension_loader = match Surface::new(&entry, &instance) {
            Ok(surface_extension_loader) => surface_extension_loader,
            Err(error) => ash_samples::clean_up_and_panic(
                &format!("Could not load surface extension: {:?}", error),
                instance, None, Vec::<vk::types::CommandPool>::new()),
        };

        let (pdevice, graphics_qf_index, presentation_qf_index) = match
            find_pdevice_with_queue_family_supporting_graphics_and_presentation(
                &instance, pdevices, &surface_extension_loader, &surface) {
            Ok(pdev_and_qs) => pdev_and_qs,
            Err(error) => ash_samples::clean_up_and_panic(
                &format!("Could not find pdevice with gfx and presentation queues: {:?}", error),
                instance, None, Vec::<vk::types::CommandPool>::new()),
        };

        let ldevice: Device<V1_0> = match
            create_ldevice_and_setup_queues(&instance, pdevice,
                                            graphics_qf_index, presentation_qf_index) {
            Ok(ldevice) => ldevice,
            Err(_err) => ash_samples::clean_up_and_panic(
                "Could not create logical device!",
                instance, None, Vec::<vk::types::CommandPool>::new()),
        };

        let (graphics_command_pool, graphics_command_buffer) =
            match create_command_pool_and_buffer(&ldevice, graphics_qf_index, 1) {
            Ok(result) => result,
            Err(err) =>
                ash_samples::clean_up_and_panic(
                    &format!("Failed to set up graphics command pool and buffer: {:?}", err),
                   instance, Some(ldevice), Vec::<vk::types::CommandPool>::new()),
        };

        let (presentation_command_pool, presentation_command_buffer) =
            match create_command_pool_and_buffer(&ldevice, graphics_qf_index, 1) {
            Ok(result) => result,
            Err(err) => ash_samples::clean_up_and_panic(
                &format!("Failed to set up graphics command pool and buffer: {:?}", err),
                instance, Some(ldevice), vec![graphics_command_pool]),
        };

        let surface_capabilities = surface_extension_loader
            .get_physical_device_surface_capabilities_khr(pdevice, surface)
            .unwrap();

        let mut desired_image_count = surface_capabilities.min_image_count + 1;

        // if max_image_count == 0, then there is no software upper limit
        if surface_capabilities.max_image_count > 0
            && desired_image_count > surface_capabilities.max_image_count
            {
                desired_image_count = surface_capabilities.max_image_count;
            }

        let swapchain_loader = match Swapchain::new(&instance, &ldevice) {
            Ok(swapchain_loader) => swapchain_loader,
            Err(error) => ash_samples::clean_up_and_panic(
                &format!("Failed creating swapchain loader: {:?}", error),
                instance, Some(ldevice),
                vec![graphics_command_pool, presentation_command_pool]),
        };

        let surface_formats =
            match surface_extension_loader.
                get_physical_device_surface_formats_khr(pdevice, surface) {
                    Ok(surface_formats) => surface_formats,
                    Err(error) => ash_samples::clean_up_and_panic(
                                &format!("Failed to get surface's supported formats: {:?}", error),
                                instance, Some(ldevice),
                                vec![presentation_command_pool, graphics_command_pool]),
        };

        let surface_format = match surface_formats
            .iter()
            .map(|sfmt| match sfmt.format {
                vk::Format::Undefined => vk::SurfaceFormatKHR {
                    format: vk::Format::B8g8r8Unorm,
                    color_space: sfmt.color_space,
                },
                _ => sfmt.clone(),
            })
            .nth(0) {
            Some(surface_format) => surface_format,
            _ => ash_samples::clean_up_and_panic(
                &format!("Failed to extract surface format."),
                instance, Some(ldevice),
                vec![presentation_command_pool, graphics_command_pool]),
        };

        let surface_resolution = match surface_capabilities.current_extent.width {
            std::u32::MAX => vk::Extent2D {
                width: window_width,
                height: window_height,
            },
            _ => surface_capabilities.current_extent,
        };

        let pre_transform = if surface_capabilities
            .supported_transforms
            .subset(vk::SURFACE_TRANSFORM_IDENTITY_BIT_KHR)
            {
                vk::SURFACE_TRANSFORM_IDENTITY_BIT_KHR
            } else {
            surface_capabilities.current_transform
        };

        let present_modes = match surface_extension_loader
            .get_physical_device_surface_present_modes_khr(pdevice, surface) {
            Ok(present_modes) => present_modes,
            Err(error) => ash_samples::clean_up_and_panic(
                &format!("Failed to get surface's present modes: {:?}", error),
                instance, Some(ldevice),
                vec![presentation_command_pool, graphics_command_pool]),
        };

        let present_mode = present_modes
            .iter()
            .cloned()
            .find(|&mode| mode == vk::PresentModeKHR::Mailbox)
            .unwrap_or(vk::PresentModeKHR::Fifo);

        let swapchain_create_info = vk::SwapchainCreateInfoKHR {
            s_type: vk::StructureType::SwapchainCreateInfoKhr,
            p_next: ptr::null(),
            flags: Default::default(),
            surface: surface,
            min_image_count: desired_image_count,
            image_color_space: surface_format.color_space,
            image_format: surface_format.format,
            image_extent: surface_resolution.clone(),
            image_usage: vk::IMAGE_USAGE_COLOR_ATTACHMENT_BIT,
            image_sharing_mode: vk::SharingMode::Exclusive,
            pre_transform: pre_transform,
            composite_alpha: vk::COMPOSITE_ALPHA_OPAQUE_BIT_KHR,
            present_mode: present_mode,
            clipped: 1,
            old_swapchain: vk::SwapchainKHR::null(),
            image_array_layers: 1,
            p_queue_family_indices: ptr::null(),
            queue_family_index_count: 0,
        };

        let swapchain = match swapchain_loader
            .create_swapchain_khr(&swapchain_create_info, None) {
            Ok(swapchain) => swapchain,
            Err(error) => ash_samples::clean_up_and_panic(
                &format!("Failed to create swapchain: {:?}", error),
                instance, Some(ldevice),
                vec![presentation_command_pool, graphics_command_pool]),
        };

        let present_images = match swapchain_loader
            .get_swapchain_images_khr(swapchain) {
            Ok(present_images) => present_images,
            Err(error) => ash_samples::clean_up_and_panic(
                &format!("Failed to get presentable images from swapchain: {:?}", error),
                instance, Some(ldevice),
                vec![presentation_command_pool, graphics_command_pool]),
        };

        let present_image_views: Vec<vk::ImageView> = present_images
            .iter()
            .map(|&image| {
                let create_view_info = vk::ImageViewCreateInfo {
                    s_type: vk::StructureType::ImageViewCreateInfo,
                    p_next: ptr::null(),
                    flags: Default::default(),
                    view_type: vk::ImageViewType::Type2d,
                    format: surface_format.format,
                    components: vk::ComponentMapping {
                        r: vk::ComponentSwizzle::R,
                        g: vk::ComponentSwizzle::G,
                        b: vk::ComponentSwizzle::B,
                        a: vk::ComponentSwizzle::A,
                    },
                    subresource_range: vk::ImageSubresourceRange {
                        aspect_mask: vk::IMAGE_ASPECT_COLOR_BIT,
                        base_mip_level: 0,
                        level_count: 1,
                        base_array_layer: 0,
                        layer_count: 1,
                    },
                    image: image,
                };
                ldevice.create_image_view(&create_view_info, None).unwrap()
            })
            .collect();

        println!("Cleaning up...");
        ash_samples::clean_up(instance, Some(ldevice), vec![graphics_command_pool, presentation_command_pool]);
    }
}

#[cfg(all(windows))]
fn get_extension_names() -> Vec<*const i8> {
    vec![
        Surface::name().as_ptr(),
        Win32Surface::name().as_ptr(),
        DebugReport::name().as_ptr(),
    ]
}

#[cfg(all(unix, not(target_os = "android")))]
fn get_extension_names() -> Vec<*const i8> {
    vec![
        Surface::name().as_ptr(),
        XlibSurface::name().as_ptr(),
        DebugReport::name().as_ptr(),
    ]
}

pub unsafe fn init_instance_with_extensions(app_name: &str) -> (Entry<V1_0>, Instance<V1_0>) {
    let app_name_raw = CString::new(app_name).unwrap().as_ptr();

    println!("Creating ApplicationInfo...");
    let appinfo = vk::ApplicationInfo {
        p_application_name: app_name_raw,
        s_type: vk::StructureType::ApplicationInfo,
        p_next: ptr::null(),
        application_version: 0,
        p_engine_name: app_name_raw,
        engine_version: 0,
        api_version: ash::vk_make_version!(1, 0, 36),
    };

    let pp_extension_names = get_extension_names();

    println!("Creating InstanceCreateInfo...");
    let create_info = vk::InstanceCreateInfo {
        s_type: vk::StructureType::InstanceCreateInfo,
        p_next: ptr::null(),
        flags: Default::default(),
        p_application_info: &appinfo,
        pp_enabled_layer_names: ptr::null(),
        enabled_layer_count: 0 as u32,
        pp_enabled_extension_names: pp_extension_names.as_ptr(),
        enabled_extension_count: pp_extension_names.len() as u32,
    };

    println!("Creating instance...");
    let entry = Entry::new().unwrap();
    let instance: Instance<V1_0> = entry
            .create_instance(&create_info, None)
            .expect("Instance creation error");
    // definition of `entry` at: https://docs.rs/ash/0.20.2/src/ash/entry.rs.html#51-54

    (entry, instance)
}

fn create_events_loop_and_window(window_width: u32, window_height: u32)
    -> (winit::EventsLoop, winit::Window) {

    let window_size = winit::dpi::LogicalSize::from((window_width, window_height));

    let events_loop = winit::EventsLoop::new();
    let window = winit::WindowBuilder::new()
        .with_title("Tutorial")
        .with_dimensions(window_size)
        .build(&events_loop)
        .unwrap();

    (events_loop, window)
}

#[cfg(all(unix, not(target_os = "android")))]
unsafe fn create_surface<E: EntryV1_0, I: InstanceV1_0>(
    entry: &E,
    instance: &I,
    window: &winit::Window,
) -> Result<vk::SurfaceKHR, vk::Result> {
    use winit::os::unix::WindowExt;
    let x11_display = window.get_xlib_display().unwrap();
    let x11_window = window.get_xlib_window().unwrap();
    let x11_create_info = vk::XlibSurfaceCreateInfoKHR {
        s_type: vk::StructureType::XlibSurfaceCreateInfoKhr,
        p_next: ptr::null(),
        flags: Default::default(),
        window: x11_window as vk::Window,
        dpy: x11_display as *mut vk::Display,
    };
    let xlib_surface_loader =
        XlibSurface::new(entry, instance).expect("Unable to load xlib surface");
    xlib_surface_loader.create_xlib_surface_khr(&x11_create_info, None)
}

#[cfg(windows)]
unsafe fn create_surface<E: EntryV1_0, I: InstanceV1_0>(
    entry: &E,
    instance: &I,
    window: &winit::Window,
) -> Result<vk::SurfaceKHR, vk::Result> {

    use winapi::shared::windef::HWND;
    use winapi::um::winuser::GetWindow;
    use winit::os::windows::WindowExt;

    let hwnd = window.get_hwnd() as HWND;
    let hinstance = GetWindow(hwnd, 0) as *const vk::c_void;
    let win32_create_info = vk::Win32SurfaceCreateInfoKHR {
        s_type: vk::StructureType::Win32SurfaceCreateInfoKhr,
        p_next: ptr::null(),
        flags: Default::default(),
        hinstance: hinstance,
        hwnd: hwnd as *const vk::c_void,
    };
    let win32_surface_loader =
        Win32Surface::new(entry, instance).expect("Unable to load win32 surface");
    win32_surface_loader.create_win32_surface_khr(&win32_create_info, None)
}

fn find_pdevice_with_queue_family_supporting_graphics_and_presentation(
    instance: &Instance<V1_0>,
    pdevices: Vec<vk::PhysicalDevice>,
    surface_extension_loader: &Surface,
    surface: &vk::types::SurfaceKHR,
) -> Result<(vk::PhysicalDevice, u32, u32), String> {

    let mut graphics_qf_index: Option<u32> = None;
    let mut presentation_qf_index: Option<u32> = None;
    let mut pdevice: Option<vk::PhysicalDevice> = None;

    'pdevice_loop: for pd in pdevices.iter() {
        for (index, qfp) in instance
                    .get_physical_device_queue_family_properties(*pd)
                    .iter()
                    .enumerate() {

            let index: u32 = index as u32;

            if graphics_qf_index.is_none() && qfp.queue_flags.subset(vk::QUEUE_GRAPHICS_BIT) {
                graphics_qf_index = Some(index);
            }


            if presentation_qf_index.is_none() &&
            surface_extension_loader.get_physical_device_surface_support_khr(*pd, index, *surface) {
                presentation_qf_index = Some(index);
            }

            if graphics_qf_index.is_some() && presentation_qf_index.is_some() {
                pdevice = Some(*pd);
                break 'pdevice_loop;
            }
        }
    }

    match pdevice {
        Some(pd) => Ok((pd, graphics_qf_index.unwrap(), presentation_qf_index.unwrap())),
        None => Err(String::from("Could not find suitable pdevice!")),
    }
}

unsafe fn create_ldevice_and_setup_queues(instance: &Instance<V1_0>, pdevice: vk::PhysicalDevice,
                                   graphics_qf_index: u32, presentation_qf_index: u32)
    -> Result<Device<V1_0>, ash::DeviceError>
{
    let priorities: [f32; 1] = [1.0];

    let graphics_queue_info = vk::DeviceQueueCreateInfo {
        s_type: vk::StructureType::DeviceQueueCreateInfo,
        p_next: ptr::null(),
        flags: Default::default(),
        queue_family_index: graphics_qf_index,
        p_queue_priorities: priorities.as_ptr(),
        queue_count: priorities.len() as u32,
    };

    let presentation_queue_info = vk::DeviceQueueCreateInfo {
        s_type: vk::StructureType::DeviceQueueCreateInfo,
        p_next: ptr::null(),
        flags: Default::default(),
        queue_family_index: presentation_qf_index,
        p_queue_priorities: priorities.as_ptr(),
        queue_count: priorities.len() as u32,
    };

    let num_queue_info_structs: u32 = 2;
    let queue_infos: Vec<vk::DeviceQueueCreateInfo> = vec![graphics_queue_info,
                                                               presentation_queue_info];

    let device_extension_names_pointers = [Swapchain::name().as_ptr()];

    let ldevice_create_info = vk::DeviceCreateInfo {
        s_type: vk::StructureType::DeviceCreateInfo,
        p_next: ptr::null(),
        flags: Default::default(),
        queue_create_info_count: num_queue_info_structs,
        p_queue_create_infos: queue_infos.as_ptr(),
        enabled_layer_count: 0,
        pp_enabled_layer_names: ptr::null(),
        enabled_extension_count: device_extension_names_pointers.len() as u32,
        pp_enabled_extension_names: device_extension_names_pointers.as_ptr(),
        p_enabled_features: ptr::null(),
    };

    instance.create_device(pdevice, &ldevice_create_info, None)
}

unsafe fn create_command_pool_and_buffer(ldevice: &Device<V1_0>,
                                         qf_index: u32,
                                         command_buffer_count: u32)
    -> Result<(vk::CommandPool, Vec<vk::CommandBuffer>), vk::Result>
{
    let pool_create_info = vk::CommandPoolCreateInfo {
        s_type: vk::StructureType::CommandPoolCreateInfo,
        p_next: ptr::null(),
        flags: vk::CommandPoolCreateFlags::empty(),
        queue_family_index: qf_index,
    };

    let command_pool = match ldevice.create_command_pool(&pool_create_info,
                                                         None)
    {
        Ok(command_pool) => command_pool,
        Err(err) => { return Err(err); }
    };

    let command_buffer_allocate_info = vk::CommandBufferAllocateInfo {
        s_type: vk::StructureType::CommandBufferAllocateInfo,
        p_next: ptr::null(),
        command_buffer_count: command_buffer_count,
        command_pool: command_pool,
        level: vk::CommandBufferLevel::Primary,
    };

    match ldevice.allocate_command_buffers(&command_buffer_allocate_info) {
        Ok(command_buffer) => {
            Ok((command_pool, command_buffer))
        }
        Err(error) => {
            Err(error)
        }
    }
}

