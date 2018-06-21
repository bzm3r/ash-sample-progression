extern crate ash;
extern crate ash_samples;

use ash::vk;
use ash::Entry;
use ash::Instance;
use ash::Device;
use ash::version::{DeviceV1_0, InstanceV1_0, V1_0};
use std::ptr;

// code is a Rust version of: https://github.com/LunarG/VulkanSamples/blob/master/API-Samples/02-enumerate_devices/02-enumerate_devices.cpp
// please look at ash-tutorial.pdf for further information!

fn main() {
    unsafe {
        let (_entry, instance): (Entry<V1_0>, Instance<V1_0>) = ash_samples::init_instance("init-command-buffer-sample");

        let pdevices = match instance.enumerate_physical_devices() {
            Ok(pdevices) => pdevices,
            Err(error) => {
                // we should destroy the instance we have created, before panicking
                ash_samples::destroy_instance_and_panic(
                    &format!("failed to get list of pdevices: {:?}", error),
                    instance,
                );
            }
        };

        println!("{} pdevices found.", pdevices.len());
        if pdevices.len() == 0 {
            clean_up_and_panic("No physical devices found!", instance, None, None);
        }

        let (pdevice, queue_family_index): (vk::types::PhysicalDevice, usize) =
            match ash_samples::find_relevant_pdevice_and_queue_family(
                &instance,
                pdevices,
                vec![vk::QUEUE_GRAPHICS_BIT],
            ) {
                Some(result) => result,
                None => {
                    clean_up_and_panic(
                        "Could not find a capable physical device!",
                        instance,
                        None,
                        None,
                    );
                }
            };

        let qfp_info =
            &(instance.get_physical_device_queue_family_properties(pdevice))[queue_family_index];
        println!("Found a pdevice with capable queue family: ");
        print!(
            "===========\n\
             queue family index: {}\n\
             num queues: {}\n\
             supported operations: {}\n\
             ===========\n",
            queue_family_index,
            qfp_info.queue_count as u32,
            ash_samples::get_queue_family_supported_ops(qfp_info.queue_flags)
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

        let device: Device<V1_0> = match instance.create_device(pdevice, &device_create_info, None)
        {
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

        println!("Cleaning up...");
        clean_up(instance, Some(device), Some(pool));
    }
}

unsafe fn clean_up_and_panic(
    message: &str,
    instance: Instance<V1_0>,
    some_device: Option<Device<V1_0>>,
    some_pool: Option<vk::CommandPool>,
) -> ! {
    clean_up(instance, some_device, some_pool);
    panic!("panic: {}", message);
}

unsafe fn clean_up(
    instance: Instance<V1_0>,
    some_device: Option<Device<V1_0>>,
    some_pool: Option<vk::CommandPool>,
) {
    match some_device {
        Some(device) => {
            match some_pool {
                Some(pool) => {
                    println!("Destroying pool...");
                    device.destroy_command_pool(pool, None);
                }
                None => {}
            };

            println!("Destroying device...");
            device.destroy_device(None);
        }
        None => {}
    };

    println!("Destroying instance...");
    instance.destroy_instance(None);

    println!("Clean up complete.");
}

