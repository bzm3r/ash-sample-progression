#![feature(use_extern_macros)]
extern crate ash;

use std::ffi::CString; 
use ash::vk;
use std::ptr;
use std::default::Default;
use ash::Entry;
use ash::Instance;
use ash::Device;
use ash::version::{DeviceV1_0, EntryV1_0, InstanceV1_0, V1_0};

// code is a Rust version of: https://github.com/LunarG/VulkanSamples/blob/master/API-Samples/02-enumerate_devices/02-enumerate_devices.cpp
// please look at ash-tutorial.pdf for further information!

fn main() {
    unsafe {
        let instance: Instance<V1_0> = init_instance();

        let pdevices = instance
                    .enumerate_physical_devices()
                    .expect("Physical device error");

        println!("{} pdevices found.", pdevices.len());
        if pdevices.len() == 0 {
            clean_up_and_panic("No physical devices found!", 
                instance, None, None);
        }

        let (pdevice, queue_family_index): (vk::types::PhysicalDevice, usize) = 
        match find_relevant_pdevice_and_queue_family(&instance, pdevices, vec![vk::QUEUE_GRAPHICS_BIT]) {
            Some(result) => result,
            None => {
                clean_up_and_panic("Could not find a capable physical device!", 
                    instance, None, None);
            },
        };

        let qfp_info = &(instance.get_physical_device_queue_family_properties(pdevice))[queue_family_index];
        println!("Found a pdevice with capable queue family: ");
        print!("===========\n\
                queue family index: {}\n\
                num queues: {}\n\
                supported operations: {}\n\
                ===========\n", 
        queue_family_index, 
        qfp_info.queue_count as u32, 
        get_queue_family_supported_ops(qfp_info.queue_flags));

        let priorities: [f32; 1] = [1.0];
        let queue_info = vk::DeviceQueueCreateInfo {
            s_type: vk::StructureType::DeviceQueueCreateInfo,
            p_next: ptr::null(),
            flags: Default::default(),
            queue_family_index: queue_family_index as u32,
            p_queue_priorities: priorities.as_ptr(),
            queue_count: priorities.len() as u32,
        };

        let device_create_info = vk::DeviceCreateInfo {
            s_type: vk::StructureType::DeviceCreateInfo,
            p_next: ptr::null(),
            flags: Default::default(),
            queue_create_info_count: 1,
            p_queue_create_infos: &queue_info,
            enabled_layer_count: 0,
            pp_enabled_layer_names: ptr::null(),
            enabled_extension_count: 0,
            pp_enabled_extension_names: ptr::null(),
            p_enabled_features: ptr::null(),
        };

        let device: Device<V1_0> = match instance.create_device(pdevice, &device_create_info, None) {
            Ok(device) => {
                println!("Successfully created logical device.");
                device
            },
            Err(error) => {
                clean_up_and_panic(&format!("failed to create logical device: {:?}", error), 
                    instance, None, None);
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
            },
            Err(error) => {
                clean_up_and_panic(&format!("failed to create command pool: {:?}", error), 
                    instance, Some(device), None);
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
            },
            Err(error) => {
                clean_up_and_panic(&format!("failed to allocate command buffer: {:?}", error), 
                    instance, Some(device), Some(pool));
            }
        };

        println!("Cleaning up...");
        clean_up(instance, Some(device), Some(pool));
    }
}


fn init_instance() -> Instance<V1_0> {
        unsafe {
        let app_name = CString::new("vulkansamples_instance").unwrap();
        let app_name_raw = app_name.as_ptr();

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

        println!("Creating InstanceCreateInfo...");
        let create_info = vk::InstanceCreateInfo {
            s_type: vk::StructureType::InstanceCreateInfo,
            p_next: ptr::null(),
            flags: Default::default(),
            p_application_info: &appinfo,
            pp_enabled_layer_names: ptr::null(),
            enabled_layer_count: 0 as u32,
            pp_enabled_extension_names: ptr::null(),
            enabled_extension_count: 0 as u32,
        };

        println!("Creating instance...");
        let entry = Entry::new().unwrap();
        let instance: Instance<V1_0> = entry
                    .create_instance(&create_info, None)
                    .expect("Instance creation error");
        // https://docs.rs/ash/0.20.2/src/ash/entry.rs.html#51-54

        instance
    }
}

unsafe fn clean_up_and_panic(message: &str, instance: Instance<V1_0>, 
    some_device: Option<Device<V1_0>>, 
    some_pool: Option<vk::CommandPool>) -> ! {
    
    clean_up(instance, some_device, some_pool);
    panic!("panic: {}", message);
}

unsafe fn clean_up(instance: Instance<V1_0>, 
                   some_device: Option<Device<V1_0>>, 
                   some_pool: Option<vk::CommandPool>) {
    match some_device {
        Some(device) => {
            match some_pool {
                Some(pool) => {
                    println!("Destroying pool...");
                    device.destroy_command_pool(pool, None);
                },
                None => {},
            };

            println!("Destroying device...");
            device.destroy_device(None);
        },
        None => {},
    };

    println!("Destroying instance...");
    instance.destroy_instance(None);

    println!("Clean up complete.");
}

fn get_queue_family_supported_ops(queue_flags: vk::types::QueueFlags) -> String {
    let possible_ops: [(&str, vk::types::QueueFlags); 4] = [("GRAPHICS", vk::QUEUE_GRAPHICS_BIT), 
                                                            ("COMPUTE", vk::QUEUE_COMPUTE_BIT), 
                                                            ("TRANSFER", vk::QUEUE_TRANSFER_BIT),
                                                            ("SPARSE", vk::QUEUE_SPARSE_BINDING_BIT)];
    possible_ops
        .iter()
        .filter_map(
            |&(op, bit)|

            if queue_flags.subset(bit) == true {
                Some(op)
            } else {
                None
            })
        .collect::<Vec<&str>>()
        .join(", ")
}

fn find_relevant_pdevice_and_queue_family(instance: &Instance<V1_0>, 
    pdevices: Vec<vk::types::PhysicalDevice>, 
    required_capabilities: Vec<vk::types::QueueFlags>) 
    -> Option<(vk::types::PhysicalDevice, usize)> {

    pdevices
    .iter()
    .map(|pdevice|
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
            .nth(0))
    .filter_map(|r| r)
    .nth(0)
}