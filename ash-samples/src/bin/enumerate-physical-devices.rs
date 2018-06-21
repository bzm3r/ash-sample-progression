extern crate ash_samples;
extern crate ash;

use ash::Entry;
use ash::Instance;
use ash::version::{InstanceV1_0, V1_0};

// please look at ash-tutorial.pdf for further information!
fn main() {
    unsafe {
        let (_entry, instance): (Entry<V1_0>, Instance<V1_0>) = ash_samples::init_instance_without_extensions("enumerate-devices-sample");

        let pdevices = match instance.enumerate_physical_devices() {
            Ok(pdevices) => pdevices,
            Err(error) => {
                // we should destroy the instance we have created, before panicking
                ash_samples::destroy_instance_and_panic(
                    &format!("failed to create pdevices: {:?}", error),
                    instance,
                );
            }
        };

        println!("pdevices found: {}.", pdevices.len());

        println!("Destroying instance...");
        instance.destroy_instance(None);
    }
}