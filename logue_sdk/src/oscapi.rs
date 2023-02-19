use core::intrinsics::transmute;
use core::slice;

use crate::dsp::linintf;

pub const SAMPLERATE: u32 = 48_000;
pub const SAMPLERATE_RECIPF: f32 = 2.08333333333333e-005_f32;

extern "C" {
    pub static wavesA: [*const f32; 16];
    pub static wavesB: [*const f32; 16];
    pub static wavesC: [*const f32; 14];
    pub static wavesD: [*const f32; 13];
    pub static wavesE: [*const f32; 15];
    pub static wavesF: [*const f32; 16];

    pub static bitres_lut_f: [f32; (1 << 7) + 1];
}

mod internal {
    extern "C" {
        pub fn _osc_white() -> f32;
    }
}

pub fn osc_white() -> f32 {
    unsafe { internal::_osc_white() }
}

pub fn osc_bitresf(x: f32) -> f32 {
    unsafe {
        let xf = x * (bitres_lut_f.len() - 1) as f32;
        let xi: usize = xf.to_int_unchecked();

        // TODO: Can't panic
        if xi >= bitres_lut_f.len() || xi + 1 >= bitres_lut_f.len() {
            return 0.0;
        }

        let y0 = bitres_lut_f[xi];
        let y1 = bitres_lut_f[xi + 1];
        linintf(xf - xi as f32, y0, y1)
    }
}

#[repr(C)]
pub struct UserOscParam {
    shape_lfo: i32,
    pitch: u16,
    cutoff: u16,
    resonance: u16,
    reserved0: [u16; 3],
}

pub enum Platform {
    Prologue,
    MinilogueXD,
    NutektDigital,
}

#[repr(u16)]
pub enum OscParam {
    Param1 = 0,
    Param2,
    Param3,
    Param4,
    Param5,
    Param6,
    ParamShape,
    ParamShiftShape,
}

impl Platform {
    const fn to_byte(&self) -> u8 {
        match *self {
            Self::Prologue => 1,
            Self::MinilogueXD => 2,
            Self::NutektDigital => 3,
        }
    }
}

type UserOscFuncInit = extern "C" fn(platform: u32, api: u32) -> ();
type UserOscFuncCycle = extern "C" fn(params: *const UserOscParam, buf: *mut i32, frames: i32);
type UserOscFuncOn = extern "C" fn(params: *const UserOscParam);
type UserOscFuncOff = extern "C" fn(params: *const UserOscParam);
type UserOscFuncMute = extern "C" fn(params: *const UserOscParam);
type UserOscFuncValue = extern "C" fn(value: u16);
type UserOscFuncParam = extern "C" fn(idx: u16, value: u16);

#[repr(C, packed(1))]
pub struct UserOscHookTable {
    magic: [u8; 4],
    api: u32,
    platform: u8,
    reserved0: [u8; 7],
    func_entry: UserOscFuncInit,
    func_cycle: UserOscFuncCycle,
    func_on: UserOscFuncOn,
    func_off: UserOscFuncOff,
    func_mute: UserOscFuncMute,
    func_value: UserOscFuncValue,
    func_param: UserOscFuncParam,
}

type InitFn = extern "C" fn() -> ();

extern "C" fn init_cb<T: UserOsc>(platform: u32, api: u32) -> () {
    unsafe {
        let mut bss_p: *mut u8 = &mut _bss_start;
        let bss_e: *mut u8 = &mut _bss_end;
        while bss_p != bss_e {
            *bss_p = 0;
            bss_p = bss_p.offset(1);
        }

        let mut init_p: *const InitFn = __init_array_start;
        let init_e: *const InitFn = __init_array_end;
        while init_p != init_e {
            if !init_p.is_null() {
                (*init_p)()
            }
            init_p = init_p.offset(1);
        }
    }

    T::init(platform, api);
}

extern "C" {
    static mut _bss_start: u8;
    static mut _bss_end: u8;

    static mut __init_array_start: *const InitFn;
    static mut __init_array_end: *const InitFn;
}

extern "C" fn cycle_cb<T: UserOsc>(params: *const UserOscParam, buf: *mut i32, frames: i32) {
    unsafe {
        let params_ref: &UserOscParam = &*params;
        let frames = slice::from_raw_parts_mut(buf, frames as usize);
        T::cycle(params_ref, frames);
    }
}

extern "C" fn on_cb<T: UserOsc>(params: *const UserOscParam) {
    unsafe {
        T::note_on(&*params);
    }
}
extern "C" fn off_cb<T: UserOsc>(params: *const UserOscParam) {
    unsafe {
        T::note_off(&*params);
    }
}
extern "C" fn mute_cb<T: UserOsc>(params: *const UserOscParam) {
    unsafe {
        T::mute(&*params);
    }
}
extern "C" fn value_cb<T: UserOsc>(value: u16) {
    T::value(value);
}

extern "C" fn param_cb<T: UserOsc>(idx: u16, value: u16) {
    let param: OscParam = unsafe { transmute(idx) };
    T::param(param, value);
}

pub trait UserOsc {
    const PLATFORM: Platform;

    fn init(_platform: u32, _api: u32) {}
    fn cycle(_params: &UserOscParam, _buf: &mut [i32]) {}
    fn note_on(_params: &UserOscParam) {}
    fn note_off(_params: &UserOscParam) {}
    fn mute(_params: &UserOscParam) {}
    fn value(_value: u16) {}
    fn param(_param: OscParam, _value: u16) {}
}

pub trait UserOscHooks {
    const HOOK_TABLE: UserOscHookTable;
}

impl<T: UserOsc> UserOscHooks for T {
    const HOOK_TABLE: UserOscHookTable = UserOscHookTable {
        magic: [b'U', b'O', b'S', b'C'],
        api: 0x01_01_00,
        platform: T::PLATFORM.to_byte(),
        reserved0: [0, 0, 0, 0, 0, 0, 0],
        func_entry: init_cb::<T>,
        func_cycle: cycle_cb::<T>,
        func_on: on_cb::<T>,
        func_off: off_cb::<T>,
        func_mute: mute_cb::<T>,
        func_value: value_cb::<T>,
        func_param: param_cb::<T>,
    };
}
