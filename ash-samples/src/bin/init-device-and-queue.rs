extern crate ash;
extern crate ash_samples;

use ash::vk;
use ash::Entry;
use ash::Instance;
use ash::Device;
use ash::version::{InstanceV1_0, DeviceV1_0, V1_0};
use std::ptr;

// code is a Rust version of: https://github.com/LunarG/VulkanSamples/blob/master/API-Samples/02-enumerate_devices/02-enumerate_devices.cpp
// please look at ash-tutorial.pdf for further information!

fn main() {
    unsafe {
        let (_entry, instance): (Entry<V1_0>, Instance<V1_0>) = ash_samples::init_instance_without_extensions("init-ldevice-and-queues-sample");

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
            ash_samples::destroy_instance_and_panic("No physical devices found!", instance);
        }

        let (pdevice, queue_family_index): (vk::types::PhysicalDevice, usize) =
            match find_relevant_pdevice_and_queue_family(
                &instance,
                pdevices,
                vec![vk::QUEUE_GRAPHICS_BIT],
            ) {
                Some(result) => result,
                None => {
                    ash_samples::destroy_instance_and_panic(
                        "Could not find a capable physical ldevice!",
                        instance,
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

        let ldevice: Device<V1_0> = match instance.create_device(pdevice, &device_create_info, None)
        {
            Ok(device) => {
                println!("Successfully created logical ldevice.");
                device
            }
            Err(error) => {
                ash_samples::destroy_instance_and_panic(
                    &format!("failed to create logical ldevice: {:?}", error),
                    instance,
                );
            }
        };

        println!("Destroying ldevice...");
        ldevice.destroy_device(None);

        println!("Destroying instance...");
        instance.destroy_instance(None);
    }
}

fn find_relevant_pdevice_and_queue_family(
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
