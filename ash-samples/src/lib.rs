#![feature(use_extern_macros)]

extern crate ash;

use std::ffi::CString;
use ash::vk;
use std::ptr;
use std::default::Default;
use ash::Entry;
use ash::Instance;
use ash::version::{EntryV1_0, InstanceV1_0, V1_0};

pub fn extension_names() -> Vec<*const i8> {
    Vec::<*const i8>::new()
}

pub fn init_instance(app_name: &str) -> (Entry<V1_0>, Instance<V1_0>) {
    unsafe {
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
        let instance: Instance<V1_0> = entry
                .create_instance(&create_info, None)
                .expect("Instance creation error");
        // definition of `entry` at: https://docs.rs/ash/0.20.2/src/ash/entry.rs.html#51-54

        (entry, instance)
    }
}

pub unsafe fn destroy_instance_and_panic(message: &str, instance: Instance<V1_0>) -> ! {
instance.destroy_instance(None);
panic!("panic: {}", message);
}
