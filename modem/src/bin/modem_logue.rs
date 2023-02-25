#![no_std]
#![no_main]

use no_panics_whatsoever as _;

use logue_sdk::oscapi::{UserOscHookTable, UserOscHooks};
use modem::{Bell103, Modem};

#[cfg(feature = "logue_plugin")]
#[link_section = ".hooks"]
#[no_mangle]
static s_hook_table: UserOscHookTable = Modem::<Bell103>::HOOK_TABLE;
