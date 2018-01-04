// we want to use macros defined by the ash crate
#![feature(use_extern_macros)]
extern crate ash;

// a CString is a data structure compatable with C/C++ strings
use std::ffi::CString; 
// ash module which contains basic structs and functions reminiscent from Vulkan
use ash::vk;
// unsafe Rust feature which allows us to pass NULL pointers to C functions/interpret NULL points from C functions
use std::ptr;
// see std::default documentation
use std::default::Default;
// ash module which contains functions that provide syntactic sugar around the steps needed to interface with C Vulkan 
use ash::Entry;
// ash definition of the Instance handle
use ash::Instance;
// ash stuff to help us choose the right Vulkan version
use ash::version::{EntryV1_0, InstanceV1_0, V1_0};

// code is a Rust version of: https://github.com/LunarG/VulkanSamples/blob/master/API-Samples/01-init_instance/01-init_instance.cpp
fn main() {
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

        print!("Destroying instance...");
        instance.destroy_instance(None);
    }
}
