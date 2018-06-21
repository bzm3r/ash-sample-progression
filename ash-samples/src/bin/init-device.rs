extern crate ash;
extern crate ash_samples;

use ash::vk;
use ash::Entry;
use ash::Instance;
use ash::version::{InstanceV1_0, V1_0};

// code is a Rust version of: https://github.com/LunarG/VulkanSamples/blob/master/API-Samples/02-enumerate_devices/02-enumerate_devices.cpp
// please look at ash-tutorial.pdf for further information!
fn main() {
    unsafe {
        let (_entry, instance): (Entry<V1_0>, Instance<V1_0>) = ash_samples::init_instance_without_extensions("init-device-sample");

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

        println!("{} physical devices found.", pdevices.len());
        if pdevices.len() == 0 {
            ash_samples::destroy_instance_and_panic("No physical devices found!", instance);
        }

        println!("Selecting the very first physical device...");
        let pdevice = pdevices[0];

        println!("Getting list of queue families available...");
        let queue_family_properties = instance.get_physical_device_queue_family_properties(pdevice);

        if queue_family_properties.len() == 0 {
            ash_samples::destroy_instance_and_panic("No queue families found!", instance);
        }

        for (index, qfp_info) in queue_family_properties.iter().enumerate() {
            println!("=========");
            print!(
                "index: {}\n\
                 num queues: {}\n\
                 supported operations: {}\n",
                index,
                qfp_info.queue_count as u32,
                get_queue_family_supported_ops(qfp_info.queue_flags)
            );
        }
        println!("=========");

        println!("Destroying instance...");
        instance.destroy_instance(None);
    }
}

fn get_queue_family_supported_ops(queue_flags: vk::types::QueueFlags) -> String {
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
