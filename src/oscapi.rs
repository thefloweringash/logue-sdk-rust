use core::slice;

pub const SAMPLERATE: u32 = 48_000;
pub const SAMPLERATE_RECIPF: f32 = 2.08333333333333e-005_f32;

extern "C" {
    pub static wavesA: [*const f32; 16];
    pub static wavesB: [*const f32; 16];
    pub static wavesC: [*const f32; 14];
    pub static wavesD: [*const f32; 13];
    pub static wavesE: [*const f32; 15];
    pub static wavesF: [*const f32; 16];
}

mod internal {
    extern "C" {
        pub fn osc_white() -> f32;
    }
}

pub fn osc_white() -> f32 {
    unsafe { internal::osc_white() }
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

extern "C" fn init_trampoline<T: UserOsc>(platform: u32, api: u32) -> () {
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

extern "C" fn cycle_trampoline<T: UserOsc>(
    params: *const UserOscParam,
    buf: *mut i32,
    frames: i32,
) {
    unsafe {
        let params_ref: &UserOscParam = &*params;
        let frames = slice::from_raw_parts_mut(buf, frames as usize);
        T::cycle(params_ref, frames);
    }
}

extern "C" fn on_trampoline<T: UserOsc>(params: *const UserOscParam) {
    unsafe {
        T::note_on(&*params);
    }
}
extern "C" fn off_trampoline<T: UserOsc>(params: *const UserOscParam) {
    unsafe {
        T::note_off(&*params);
    }
}
extern "C" fn mute_trampoline<T: UserOsc>(params: *const UserOscParam) {
    unsafe {
        T::mute(&*params);
    }
}
extern "C" fn value_trampoline<T: UserOsc>(value: u16) {
    T::value(value);
}

extern "C" fn param_trampoline<T: UserOsc>(idx: u16, value: u16) {
    T::param(idx, value);
}

pub trait UserOsc {
    const PLATFORM: Platform;

    fn init(_platform: u32, _api: u32) {}
    fn cycle(_params: &UserOscParam, _buf: &mut [i32]) {}
    fn note_on(_params: &UserOscParam) {}
    fn note_off(_params: &UserOscParam) {}
    fn mute(_params: &UserOscParam) {}
    fn value(_value: u16) {}
    fn param(_idx: u16, _value: u16) {}
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
        func_entry: init_trampoline::<T>,
        func_cycle: cycle_trampoline::<T>,
        func_on: on_trampoline::<T>,
        func_off: off_trampoline::<T>,
        func_mute: mute_trampoline::<T>,
        func_value: value_trampoline::<T>,
        func_param: param_trampoline::<T>,
    };
}
