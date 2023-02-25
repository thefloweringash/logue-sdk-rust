#![no_std]

use core::mem::MaybeUninit;

use logue_sdk::dsp::{f32_to_q31, param_val_to_f32, si_roundf};
use logue_sdk::oscapi::{
    osc_bitresf, osc_notehz, osc_w0f_for_note, osc_wave_scanf, osc_white, wavesA, wavesB, OscParam,
    Platform, UserOsc, UserOscHookTable, UserOscHooks as _, UserOscParam,
};

#[derive(Clone, Copy)]
#[repr(transparent)]
struct Phi(f32);

#[derive(Clone, Copy)]
#[repr(transparent)]
struct W0(f32);

impl W0 {
    const fn default() -> Self {
        Self(0.0)
    }

    #[inline(always)]
    fn for_note(note: u8, r#mod: u8) -> Self {
        Self(osc_w0f_for_note(note, r#mod))
    }

    #[inline(always)]
    fn for_params(params: &UserOscParam) -> Self {
        Self::for_note((params.pitch >> 8) as u8, (params.pitch & 0xFF) as u8)
    }
}

impl Phi {
    const fn default() -> Self {
        Self(0.0)
    }

    #[inline(always)]
    fn advance(&mut self, w0: W0) {
        let mut next = self.0 + w0.0;
        next -= (next as u32) as f32;
        self.0 = next;
    }
}

#[derive(Default)]
struct Param {
    bitcrush: f32,
}

type WaveTable = [f32; 129];

struct State {
    wave: &'static WaveTable,
    phi: Phi,
    bitres: f32,
    bitresrcp: f32,
}

impl Default for State {
    fn default() -> Self {
        Self {
            wave: unsafe { &wavesA[0] },
            phi: Phi::default(),
            bitres: 1.0,
            bitresrcp: 1.0,
        }
    }
}

#[derive(Default)]
pub struct Noise {
    param: Param,
    state: State,
}

static mut INSTANCE: MaybeUninit<Noise> = MaybeUninit::uninit();

impl UserOsc for Noise {
    const PLATFORM: Platform = Platform::MinilogueXD;

    fn init(_platform: u32, _api: u32) {
        unsafe {
            INSTANCE.write(Noise::default());
        }
    }

    fn cycle(params: &UserOscParam, buf: &mut [i32]) {
        let noise = unsafe { INSTANCE.assume_init_mut() };
        let state = &mut noise.state;

        let w0 = W0::for_params(params);

        for i in buf {
            let mut sig = osc_wave_scanf(state.wave, state.phi.0);

            sig = si_roundf(sig * state.bitres) * state.bitresrcp;

            *i = f32_to_q31(sig);

            state.phi.advance(w0);
        }
    }

    fn note_on(_params: &UserOscParam) {}

    fn value(_value: u16) {}
    fn param(param: OscParam, value: u16) {
        let noise = unsafe { INSTANCE.assume_init_mut() };
        let mut state = &mut noise.state;
        let mut p = &mut noise.param;
        match param {
            OscParam::ParamShape => {
                let xf = param_val_to_f32(value);
                unsafe {
                    let x: usize = (xf * (wavesA.len() - 1) as f32) as usize;
                    // TODO: can't panic, so ignore on bounds check failure
                    if x < wavesA.len() {
                        let wave = wavesA[x];
                        state.wave = wave;
                    }
                }
            }
            OscParam::ParamShiftShape => {
                let value = param_val_to_f32(value);
                p.bitcrush = (value * 0.1).clamp(0.0, 1.0);

                // TODO: move to osc section like upstream?
                state.bitres = osc_bitresf(p.bitcrush);
                state.bitresrcp = state.bitres.recip();
            }
            _ => (),
        }
    }
}
