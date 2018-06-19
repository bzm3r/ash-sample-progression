#![feature(use_extern_macros)]
extern crate ash;

use std::ffi::CString;
use ash::vk;
use std::ptr;
use std::default::Default;
use ash::Entry;
use ash::Instance;
use ash::version::{EntryV1_0, InstanceV1_0, V1_0};

// please look at ash-tutorial.pdf for further information!
fn main() {
    unsafe {
        let (entry, instance): (Entry<V1_0>, Instance<V1_0>) = init_instance();

        let pdevices = match instance.enumerate_physical_devices() {
            Ok(pdevices) => pdevices,
            Err(error) => {
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

#[cfg(all(unix, not(target_os = "android")))]
fn extension_names() -> Vec<*const i8> {
    Vec::<*const i8>::new()
}

#[cfg(all(windows))]
fn extension_names() -> Vec<*const i8> {
    Vec::<*const i8>::new()
}

fn init_instance() -> (Entry<V1_0>, Instance<V1_0>) {
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

        let pp_extension_names = extension_names();

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
        let instance: Instance<V1_0> = unsafe {
            entry
                .create_instance(&create_info, None)
                .expect("Instance creation error")
        };
        // definition of `entry` at: https://docs.rs/ash/0.20.2/src/ash/entry.rs.html#51-54

        (entry, instance)
    }
}

unsafe fn destroy_instance_and_panic(message: &str, instance: Instance<V1_0>) -> ! {
    instance.destroy_instance(None);
    panic!("panic: {}", message);
}
