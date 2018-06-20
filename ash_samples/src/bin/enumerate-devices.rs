extern crate ashtutorial;

// please look at ash-tutorial.pdf for further information!
fn main() {
    unsafe {
        let (entry, instance): (Entry<V1_0>, Instance<V1_0>) = init_instance();

        let pdevices = match instance.enumerate_physical_devices() {
            Ok(pdevices) => pdevices,
            Err(error) => {
                // we should destroy the instance we have created, before panicking
                destroy_instance_and_panic(
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