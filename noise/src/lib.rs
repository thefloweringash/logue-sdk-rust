#![no_std]

use core::mem::MaybeUninit;

use logue_sdk::dsp::{f32_to_q31, param_val_to_f32, si_roundf};
use logue_sdk::oscapi::{
    osc_bitresf, osc_w0f_for_note, osc_wave_scanf, pick1, wavesA, OscParam, Platform, UserOsc,
    UserOscParam,
};

#[derive(Clone, Copy)]
#[repr(transparent)]
struct W0(f32);

impl W0 {
    #[inline(always)]
    fn for_note(note: u8, r#mod: u8) -> Self {
        Self(osc_w0f_for_note(note, r#mod))
    }

    #[inline(always)]
    fn for_params(params: &UserOscParam) -> Self {
        Self::for_note((params.pitch >> 8) as u8, (params.pitch & 0xFF) as u8)
    }
}

#[derive(Clone, Copy)]
#[repr(transparent)]
struct Phi(f32);

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

struct Bitcrush {
    res: f32,
    res_recip: f32,
}

impl Bitcrush {
    fn new(amount: f32) -> Self {
        let res = osc_bitresf(amount);
        let res_recip = res.recip();
        Self { res, res_recip }
    }

    fn apply(&self, sig: f32) -> f32 {
        si_roundf(sig * self.res) * self.res_recip
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
    bitcrush: Bitcrush,
}

impl Default for State {
    fn default() -> Self {
        Self {
            wave: unsafe { wavesA[0] },
            phi: Phi::default(),
            bitcrush: Bitcrush::new(0.0),
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

            sig = state.bitcrush.apply(sig);

            *i = f32_to_q31(sig);

            state.phi.advance(w0);
        }
    }

    fn param(param: OscParam, value: u16) {
        let noise = unsafe { INSTANCE.assume_init_mut() };
        let mut state = &mut noise.state;
        let mut p = &mut noise.param;
        match param {
            OscParam::ParamShape => {
                let x = param_val_to_f32(value);
                if let Some(wave) = pick1(unsafe { &wavesA }, x) {
                    state.wave = wave;
                }
            }
            OscParam::ParamShiftShape => {
                let value = param_val_to_f32(value);
                p.bitcrush = (value * 0.1).clamp(0.0, 1.0);

                // TODO: move to osc section like upstream?
                state.bitcrush = Bitcrush::new(p.bitcrush);
            }
            _ => (),
        }
    }
}
