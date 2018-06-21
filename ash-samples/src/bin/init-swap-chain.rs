#![feature(use_extern_macros)]
extern crate ash;
extern crate winit;

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
        let window_width = 500;
        let window_height = 500;
        
        let events_loop = winit::EventsLoop::new();
        let window = winit::WindowBuilder::new()
            .with_title("Ash - Example")
            .with_dimensions(window_width, window_height)
            .build(&events_loop)
            .unwrap();

        let (entry, instance): (Entry<V1_0>, Instance<V1_0>) = ash_samples::init_instance("init-swap-chain-sample");
        let pdevices = match instance.enumerate_physical_devices() {
            Ok(pdevices) => pdevices,
            Err(error) => clean_up_and_panic(format!("Physical device error: {:?}", error),
                                             instance, None, None);
        };

        println!("{} pdevices found.", pdevices.len());
        if pdevices.len() == 0 {
            clean_up_and_panic("No physical devices found!", instance, None, None);
        }

        let (pdevice, queue_family_index): (vk::types::PhysicalDevice, usize) =
            match find_relevant_pdevice_and_queue_family(
                &instance,
                pdevices,
                vec![vk::QUEUE_GRAPHICS_BIT],
            ) {
                Some(result) => result,
                None => {
                    clean_up_and_panic("Could not find a capable physical device!",
                        instance, None, None);
                }
            };

        let qfp_info = &(instance.get_physical_device_queue_family_properties(pdevice))
            [queue_family_index];
        println!("Found a pdevice with capable queue family: ");
        print!(
            "===========\n\
                queue family index: {}\n\
                num queues: {}\n\
                supported operations: {}\n\
                ===========\n",
            queue_family_index,
            qfp_info.queue_count as u32,
            get_queue_family_supported_ops(qfp_info.queue_flags)
        );

        let priorities: [f32; 1] = [1.0];
        let queue_info = vk::DeviceQueueCreateInfo {
            s_type: vk::StructureType::DeviceQueueCreateInfo,
            p_next: ptr::null(),
            flags: Default::default(),
            queue_family_index: queue_family_index as u32,
            p_queue_priorities: priorities.as_ptr(),
            queue_count: priorities.len() as u32,
        };

        let device_extension_names_pointers = [Swapchain::name().as_ptr()];
        let device_create_info = vk::DeviceCreateInfo {
            s_type: vk::StructureType::DeviceCreateInfo,
            p_next: ptr::null(),
            flags: Default::default(),
            queue_create_info_count: 1,
            p_queue_create_infos: &queue_info,
            enabled_layer_count: 0,
            pp_enabled_layer_names: ptr::null(),
            enabled_extension_count: device_extension_names_pointers.len() as u32,
            pp_enabled_extension_names: device_extension_names_pointers,
            p_enabled_features: ptr::null(),
        };

        let device: Device<V1_0> =
            match instance.create_device(pdevice, &device_create_info, None) {
                Ok(device) => {
                    println!("Successfully created logical device.");
                    device
                }
                Err(error) => {
                    clean_up_and_panic(
                        &format!("failed to create logical device: {:?}", error),
                        instance,
                        None,
                        None,
                    );
                }
            };

        let pool_create_info = vk::CommandPoolCreateInfo {
            s_type: vk::StructureType::CommandPoolCreateInfo,
            p_next: ptr::null(),
            flags: vk::CommandPoolCreateFlags::empty(),
            queue_family_index: queue_family_index as u32,
        };

        let pool = match device.create_command_pool(&pool_create_info, None) {
            Ok(pool) => {
                println!("Successfully created command pool!");
                pool
            }
            Err(error) => {
                clean_up_and_panic(
                    &format!("failed to create command pool: {:?}", error),
                    instance,
                    Some(device),
                    None,
                );
            }
        };

        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo {
            s_type: vk::StructureType::CommandBufferAllocateInfo,
            p_next: ptr::null(),
            command_buffer_count: 1,
            command_pool: pool,
            level: vk::CommandBufferLevel::Primary,
        };

        let command_buffer = match device.allocate_command_buffers(&command_buffer_allocate_info) {
            Ok(command_buffer) => {
                println!("Successfully allocated command buffer!");
                command_buffer
            }
            Err(error) => {
                clean_up_and_panic(
                    &format!("failed to allocate command buffer: {:?}", error),
                    instance,
                    Some(device),
                    Some(pool),
                );
            }
        };

        let surface: vk::SurfaceKHR = match create_surface(&entry, &instance, &window){
            Ok(surface) => surface,
            Err(error) => clean_up_and_panic(format!("Could not create surface: {:?}", error),
                                             instance, None, None);
        }

        println!("Cleaning up...");
        clean_up(instance, Some(device), Some(pool));
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