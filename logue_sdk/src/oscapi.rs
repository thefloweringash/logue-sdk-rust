use core::intrinsics::transmute;
use core::slice;

use crate::dsp::linintf;

pub const SAMPLERATE: u32 = 48_000;
pub const SAMPLERATE_RECIPF: f32 = 2.083_333_3e-5_f32;

pub const NOTE_MOD_FSCALE: f32 = 0.00392156862745098f32;
pub const NOTE_MAX_HZ: f32 = 23679.643054f32;

extern "C" {
    // Vendor wavetables
    pub static wavesA: [&'static [f32; 129]; 16];
    pub static wavesB: [&'static [f32; 129]; 16];
    pub static wavesC: [&'static [f32; 129]; 14];
    pub static wavesD: [&'static [f32; 129]; 13];
    pub static wavesE: [&'static [f32; 129]; 15];
    pub static wavesF: [&'static [f32; 129]; 16];

    // Utility
    pub static bitres_lut_f: [f32; 129];
    pub static midi_to_hz_lut_f: [f32; 152];

    // Math
    pub static log_lut_f: [f32; 257];
    pub static sqrtm2log_lut_f: [f32; 257];
    pub static tanpi_lut_f: [f32; 257];

    // Vendor wavetables for standard waves
    pub static wt_par_lut_f: [f32; 903];
    pub static wt_par_notes: [u8; 7];
    pub static wt_saw_lut_f: [f32; 903];
    pub static wt_saw_notes: [u8; 7];
    pub static wt_sine_lut_f: [f32; 129];
    pub static wt_sqr_lut_f: [f32; 903];
    pub static wt_sqr_notes: [u8; 7];

    // Saturation
    pub static cubicsat_lut_f: [f32; 129];
    pub static schetzen_lut_f: [f32; 129];
}

mod internal {
    extern "C" {
        pub fn _osc_white() -> f32;
    }
}

#[inline(always)]
pub fn pick1<T: Copy>(opts: &[T], x: f32) -> Option<T> {
    let xf: f32 = x * ((opts.len() - 1) as f32);
    let xi: usize = xf as usize;

    if cfg!(feature = "no_panic") {
        if xi >= opts.len() {
            return None;
        }
    }

    Some(opts[xi])
}

pub fn osc_white() -> f32 {
    unsafe { internal::_osc_white() }
}

pub fn osc_bitresf(x: f32) -> f32 {
    unsafe {
        let xf = x * (bitres_lut_f.len() - 1) as f32;
        let xi: usize = xf.to_int_unchecked();

        if cfg!(feature = "no_panic") {
            if xi >= bitres_lut_f.len() || xi + 1 >= bitres_lut_f.len() {
                return *bitres_lut_f.last().unwrap_unchecked();
            }
        }

        let y0 = bitres_lut_f[xi];
        let y1 = bitres_lut_f[xi + 1];
        linintf(xf - xi as f32, y0, y1)
    }
}

#[inline(always)]
pub fn osc_wave_scanf(wave: &[f32; 129], x: f32) -> f32 {
    let p: f32 = x - ((x as u32) as f32);
    let x0f: f32 = p * (wave.len() - 1) as f32;
    let x0: u32 = (x0f as u32) & 127;
    let x1: u32 = (x0 + 1) & 127;
    linintf(
        x0f - (x0f as u32) as f32,
        wave[x0 as usize],
        wave[x1 as usize],
    )
}

#[inline(always)]
pub fn osc_notehz(note: u8) -> f32 {
    unsafe {
        let p = note.clamp(0, midi_to_hz_lut_f.len() as u8 - 1);
        midi_to_hz_lut_f[p as usize]
    }
}

#[inline(always)]
pub fn osc_w0f_for_note(note: u8, r#mod: u8) -> f32 {
    let f0: f32 = osc_notehz(note);
    let f1: f32 = osc_notehz(note + 1);

    let f: f32 = linintf(r#mod as f32 * NOTE_MOD_FSCALE, f0, f1).clamp(0.0, NOTE_MAX_HZ);

    f * SAMPLERATE_RECIPF
}

#[repr(C)]
#[derive(Default)]
pub struct UserOscParam {
    pub shape_lfo: i32,
    pub pitch: u16,
    pub cutoff: u16,
    pub resonance: u16,
    pub reserved0: [u16; 3],
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

impl TryFrom<u16> for OscParam {
    type Error = ();

    fn try_from(x: u16) -> Result<OscParam, Self::Error> {
        if x > OscParam::ParamShiftShape as u16 {
            Err(())
        } else {
            Ok(unsafe { transmute(x) })
        }
    }
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

extern "C" fn init_cb<T: UserOsc>(platform: u32, api: u32) {
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
    if let Ok(param) = idx.try_into() {
        T::param(param, value);
    }
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
