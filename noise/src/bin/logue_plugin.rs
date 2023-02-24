#![no_std]
#![no_main]

use logue_sdk::oscapi::{UserOscHookTable, UserOscHooks};
use noise::Noise;

#[cfg(feature = "logue_plugin")]
#[link_section = ".hooks"]
#[no_mangle]
static s_hook_table: UserOscHookTable = Noise::HOOK_TABLE;
