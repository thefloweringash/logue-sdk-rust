#![no_std]
#![no_main]

use logue_sdk::oscapi::{
    osc_white, Platform, UserOsc, UserOscHookTable, UserOscHooks as _, UserOscParam,
};

fn f32_to_q31(x: f32) -> i32 {
    unsafe {
        return (x * 0x7fffffff as f32).to_int_unchecked();
    }
}

struct Noise {}

impl UserOsc for Noise {
    const PLATFORM: Platform = Platform::MinilogueXD;

    fn cycle(_params: &UserOscParam, buf: &mut [i32]) {
        for i in 0..buf.len() {
            buf[i] = f32_to_q31(osc_white());
        }
    }

    fn value(_value: u16) {}
    fn param(_idx: u16, _value: u16) {}
}

#[link_section = ".hooks"]
#[no_mangle]
static s_hook_table: UserOscHookTable = Noise::HOOK_TABLE;
