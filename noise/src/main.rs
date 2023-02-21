#![no_std]
#![no_main]

use logue_sdk::dsp::{f32_to_q31, param_val_to_f32, si_roundf};
use logue_sdk::oscapi::{
    osc_bitresf, osc_white, OscParam, Platform, UserOsc, UserOscHookTable, UserOscHooks as _,
    UserOscParam,
};

struct Param {
    bitcrush: f32,
}

impl Param {
    const fn default() -> Self {
        Self { bitcrush: 0.0 }
    }
}

struct State {
    level: f32,
    bitres: f32,
    bitresrcp: f32,
}

impl State {
    const fn default() -> Self {
        Self {
            level: 1.0,
            bitres: 1.0,
            bitresrcp: 1.0,
        }
    }
}

struct Noise {
    param: Param,
    state: State,
}

impl Noise {
    const fn default() -> Self {
        Self {
            param: Param::default(),
            state: State::default(),
        }
    }
}

static mut INSTANCE: Noise = Noise::default();

impl UserOsc for Noise {
    const PLATFORM: Platform = Platform::MinilogueXD;

    fn cycle(_params: &UserOscParam, buf: &mut [i32]) {
        let noise = unsafe { &INSTANCE };
        let state = &noise.state;

        for i in buf {
            let mut sig = osc_white();
            sig *= state.level;
            sig = si_roundf(sig * state.bitres) * state.bitresrcp;

            *i = f32_to_q31(sig);
        }
    }

    fn value(_value: u16) {}
    fn param(param: OscParam, value: u16) {
        let noise = unsafe { &mut INSTANCE };
        let mut state = &mut noise.state;
        let mut p = &mut noise.param;
        match param {
            OscParam::ParamShape => {
                state.level = param_val_to_f32(value);
            }
            OscParam::ParamShiftShape => {
                let value = param_val_to_f32(value);
                p.bitcrush = (value * 0.1).clamp(0.0, 1.0);

                // TODO: move to osc section like upstream?
                state.bitres = osc_bitresf(p.bitcrush);
                state.bitresrcp = 1.0 / state.bitres;
            }
            _ => (),
        }
    }
}

#[link_section = ".hooks"]
#[no_mangle]
static s_hook_table: UserOscHookTable = Noise::HOOK_TABLE;
