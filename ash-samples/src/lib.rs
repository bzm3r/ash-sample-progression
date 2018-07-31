#![feature(use_extern_macros)]

extern crate ash;

use std::ffi::CString;
use ash::vk;
use std::ptr;
use std::default::Default;
use ash::Entry;
use ash::Instance;
use ash::Device;
use ash::version::{EntryV1_0, InstanceV1_0, DeviceV1_0, V1_0};
use ash::extensions::{Swapchain, Surface, Win32Surface, XlibSurface, DebugReport};

pub unsafe fn init_instance_without_extensions(app_name: &str) -> (Entry<V1_0>, Instance<V1_0>) {
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

    let pp_extension_names = Vec::<*const i8>::new();

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

pub unsafe fn destroy_instance_and_panic(message: &str, instance: Instance<V1_0>) -> ! {
    instance.destroy_instance(None);
    panic!("panic: {}", message);
}

pub fn get_queue_family_supported_ops(queue_flags: vk::types::QueueFlags) -> String {
    let possible_ops: [(&str, vk::types::QueueFlags); 4] = [
        ("GRAPHICS", vk::QUEUE_GRAPHICS_BIT),
        ("COMPUTE", vk::QUEUE_COMPUTE_BIT),
        ("TRANSFER", vk::QUEUE_TRANSFER_BIT),
        ("SPARSE", vk::QUEUE_SPARSE_BINDING_BIT),
    ];

    possible_ops
        .iter()
        .filter_map(|&(op, bit)| {
            if queue_flags.subset(bit) == true {
                Some(op)
            } else {
                None
            }
        })
        .collect::<Vec<&str>>()
        .join(", ")
}

pub fn find_relevant_pdevice_and_queue_family(
    instance: &Instance<V1_0>,
    pdevices: Vec<vk::types::PhysicalDevice>,
    required_capabilities: Vec<vk::types::QueueFlags>,
) -> Option<(vk::types::PhysicalDevice, usize)> {
    pdevices
        .iter()
        .map(|pdevice| {
            instance
                .get_physical_device_queue_family_properties(*pdevice)
                .iter()
                .enumerate()
                .filter_map(|(index, qfp)| {
                    let has_required_capabilities: bool = required_capabilities
                        .iter()
                        .all(|&req_bit| qfp.queue_flags.subset(req_bit));

                    match has_required_capabilities {
                        true => Some((*pdevice, index)),
                        false => None,
                    }
                })
                .nth(0)
        })
        .filter_map(|r| r)
        .nth(0)
}

pub unsafe fn clean_up_and_panic(
    message: &str,
    instance: Instance<V1_0>,
    some_ldevice: Option<Device<V1_0>>,
    pools: Vec<vk::types::CommandPool>,
) -> ! {
    clean_up(instance, some_ldevice, pools);
    panic!("panic: {}", message);
}

pub unsafe fn clean_up(
    instance: Instance<V1_0>,
    some_ldevice: Option<Device<V1_0>>,
    pools: Vec<vk::CommandPool>) {
    match some_ldevice {
        Some(ldevice) => {
            for pool in pools {
                ldevice.destroy_command_pool(pool, None);
            }

            println!("Destroying ldevice...");
            ldevice.destroy_device(None);
        },
        None => {}
    };

    println!("Destroying instance...");
    instance.destroy_instance(None);

    println!("Clean up complete.");
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