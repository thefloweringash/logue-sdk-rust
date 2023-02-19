#![no_std]
#![no_main]

mod oscapi;

use bitflags::bitflags;
use core::mem::MaybeUninit;
use no_panics_whatsoever as _;
use oscapi::{
    osc_white, Platform, UserOsc, UserOscHookTable, UserOscHooks as _, UserOscParam,
    SAMPLERATE_RECIPF,
};

bitflags! {
    struct Flags: u8  {
        const WAVE0 = 1<<1;
        const WAVE1 = 1<<2;
        const SUBWAVE = 1<<3;
        const RINGMIX = 1<<4;
        const BITCRUSH = 1<<5;
        const RESET = 1<<6;
    }
}

struct Waves {
    params: Params,
    state: State,
}

struct Params {
    submix: f32,
    ringmix: f32,
    bitcrush: f32,
    shape: f32,
    shiftshape: f32,
    wave0: u8,
    wave1: u8,
    subwave: u8,
    padding: u8,
}

impl Params {
    const fn default() -> Self {
        Self {
            submix: 0.05,
            ringmix: 0.0,
            bitcrush: 0.0,
            shape: 0.0,
            shiftshape: 0.0,
            wave0: 0,
            wave1: 0,
            subwave: 0,
            padding: 0,
        }
    }
}

struct State {
    wave0: *const f32,
    wave1: *const f32,
    subwave: *const f32,
    phi0: f32,
    phi1: f32,
    phisub: f32,
    w00: f32,
    w01: f32,
    w0sub: f32,
    lfo: f32,
    lfoz: f32,
    dither: f32,
    bitres: f32,
    bitresrcp: f32,
    imperfection: f32,
    flags: Flags,
}

impl State {
    fn new() -> Self {
        Self {
            wave0: unsafe { oscapi::wavesA[0] },
            wave1: unsafe { oscapi::wavesD[0] },
            subwave: unsafe { oscapi::wavesA[0] },
            phi0: 0.0,
            phi1: 0.0,
            phisub: 0.0,
            w00: 440.0 * SAMPLERATE_RECIPF,
            w01: 440.0 * SAMPLERATE_RECIPF,
            w0sub: 220.0 * SAMPLERATE_RECIPF,
            lfo: 0.0,
            lfoz: 0.0,
            dither: 0.0,
            bitres: 1.0,
            bitresrcp: 1.0,
            imperfection: osc_white() * 1.0417e-006f32, // +/- 0.05Hz@48KHz,
            flags: Flags::empty(),
        }
    }

    fn reset(&mut self) {
        self.phi0 = 0.0;
        self.phi1 = 0.0;
        self.phisub = 0.0;
        self.lfo = self.lfoz;
    }
}

static mut INSTANCE: MaybeUninit<Waves> = MaybeUninit::uninit();

impl UserOsc for Waves {
    const PLATFORM: Platform = Platform::NutektDigital;

    fn init(_platform: u32, _api: u32) {
        unsafe {
            INSTANCE.write(Waves {
                params: Params::default(),
                state: State::new(),
            });
        }
    }

    fn cycle(_params: &UserOscParam, _buf: &mut [i32]) {
        let waves = unsafe { &mut INSTANCE.assume_init_mut() };
        let mut _params = &mut waves.params;
        let mut state = &mut waves.state;

        {
            let flags = state.flags;
            state.flags = Flags::empty();

            // state.updatePitch();
            // state.updateWaves();

            if flags.contains(Flags::RESET) {
                state.reset();
            }
        }
    }

    fn value(_value: u16) {}
    fn param(_idx: u16, _value: u16) {}
}

#[link_section = ".hooks"]
#[no_mangle]
static s_hook_table: UserOscHookTable = Waves::HOOK_TABLE;
