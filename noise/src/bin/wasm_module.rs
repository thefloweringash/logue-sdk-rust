#![no_main]

use std::slice;

use logue_sdk::dsp::q31_to_f32;
use logue_sdk::oscapi::{UserOsc, UserOscParam};

use noise::Noise;

#[cfg(feature = "wasm_module")]
#[no_mangle]
pub extern "C" fn init() {
    Noise::init(0, 0); // TODO: what do upstream do?
}

#[cfg(feature = "wasm_module")]
#[no_mangle]
pub extern "C" fn cycle(buf: *mut f32, frames: i32) {
    let frames: usize = frames.try_into().unwrap();

    let mut params = UserOscParam::default(); // TODO: what do upstream do?
    params.pitch = 0x40_00;

    let mut isamples: Vec<i32> = vec![0; frames];
    Noise::cycle(&params, &mut isamples);

    let samples = unsafe { slice::from_raw_parts_mut(buf, frames) };
    for i in 0..frames {
        samples[i] = q31_to_f32(isamples[i])
    }
}

#[cfg(feature = "wasm_module")]
#[no_mangle]
pub extern "C" fn param(idx: u16, value: u16) {
    let param = idx.try_into().unwrap();
    Noise::param(param, value);
}

#[cfg(feature = "wasm_module")]
#[no_mangle]
pub extern "C" fn allocate_sample_buffer(capacity: usize) -> *mut f32 {
    let mut vec = Vec::with_capacity(capacity);
    let bytes = vec.as_mut_ptr();
    std::mem::forget(vec);
    bytes
}
