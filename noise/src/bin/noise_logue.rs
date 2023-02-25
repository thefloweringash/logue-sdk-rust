#![no_std]
#![no_main]

use no_panics_whatsoever as _;

#[cfg(feature = "logue_plugin")]
logue_sdk::user_osc_hooks!(noise::Noise);
