#![no_std]
#![no_main]

use logue_sdk::oscapi::UserOsc;
use logue_sdk::oscapi::UserOscParam;
use noise::Noise;

#[cfg(feature = "wasm_module")]
#[no_mangle]
pub extern "C" fn init() {
    Noise::init(0, 0); // TODO: what do upstream do?
}

#[cfg(feature = "wasm_module")]
#[no_mangle]
pub extern "C" fn cycle() {
    let params = UserOscParam::default();
    let mut samples: [i32; 32] = [0; 32];
    Noise::cycle(&params, &mut samples); // TODO: what do upstream do?
}
