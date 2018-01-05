#![feature(use_extern_macros)]
extern crate ash;

use std::ffi::CString; 
use ash::vk;
use std::ptr;
use std::default::Default;
use ash::Entry;
use ash::Instance;
use ash::version::{EntryV1_0, InstanceV1_0, V1_0};

// code is a Rust version of: https://github.com/LunarG/VulkanSamples/blob/master/API-Samples/02-enumerate_devices/02-enumerate_devices.cpp
// please look at ash-tutorial.pdf for further information!
fn main() {
    unsafe {
        let instance: Instance<V1_0> = init_instance();

        let pdevices = instance
                    .enumerate_physical_devices()
                    .expect("Physical device error");

        println!("{} pdevices found.", pdevices.len());
        assert!(pdevices.len() > 0);

        let pdevice = pdevices[0];
        let q_family_properties = instance.get_physical_device_queue_family_properties(*pdevice);

        println!("Destroying instance...");
        instance.destroy_instance(None);
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

