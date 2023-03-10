#![no_std]

use core::iter::Iterator;
use core::marker::PhantomData;
use core::mem;
use core::slice;

use logue_sdk::dsp::f32_to_q31;
use logue_sdk::oscapi::{
    cubicsat_lut_f, osc_wave_scanf, schetzen_lut_f, wavesA, Platform, UserOsc, UserOscParam,
    SAMPLERATE, SAMPLERATE_RECIPF,
};

#[derive(Clone, Copy)]
#[repr(transparent)]
struct Phi(f32);

impl Phi {
    fn new() -> Self {
        Self(0.0)
    }

    #[inline(always)]
    fn advance(&mut self, w0: f32) {
        let mut next = self.0 + w0;
        next -= (next as u32) as f32;
        self.0 = next;
    }
}

// We need to be able to be interrupted at any point, and then continue at the
// next sample. So the easiest model is to just have a single wave position
// tracker, and a composite offset (byte,bit,sample) representing overall
// operation. When sample overflows, it increments bit. When bit overflows it
// increments byte. When byte overflows, we're done.

pub trait ModemParams {
    const SAMPLES_PER_BIT: usize;
    const ZERO_W0: f32;
    const ONE_W0: f32;
}

pub struct Bell103 {}

impl ModemParams for Bell103 {
    // 300bps, 48k sample rate
    const SAMPLES_PER_BIT: usize = SAMPLERATE as usize / 300;
    const ZERO_W0: f32 = 1070_f32 * SAMPLERATE_RECIPF;
    const ONE_W0: f32 = 1270_f32 * SAMPLERATE_RECIPF;
}

pub struct SampleIter<T: ModemParams> {
    buf: &'static [u8],
    byte_off: usize,
    bit_off: usize,
    sample_off: usize,
    params: PhantomData<T>,
}

impl<T: ModemParams> SampleIter<T> {
    pub fn new(buf: &'static [u8]) -> Self {
        SampleIter {
            buf,
            byte_off: 0,
            bit_off: 0,
            sample_off: 0,
            params: PhantomData,
        }
    }
}

impl<T: ModemParams> Iterator for SampleIter<T> {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        // Rollover from smallest to largest
        if self.sample_off == T::SAMPLES_PER_BIT {
            self.sample_off = 0;
            self.bit_off += 1;
        }

        if self.bit_off == 10 {
            self.bit_off = 0;
            self.byte_off += 1;
        }

        if self.byte_off >= self.buf.len() {
            return None;
        }

        let current_byte = self.buf[self.byte_off];

        let w0 = match self.bit_off {
            // Start bit
            0 => T::ZERO_W0,

            // Data bit
            1..=8 => {
                let current_bit = (current_byte >> (self.bit_off - 1)) & 0x1;
                if current_bit == 1 {
                    T::ONE_W0
                } else {
                    T::ZERO_W0
                }
            }

            // Stop bit
            _ => T::ONE_W0,
        };

        self.sample_off += 1;

        Some(w0)
    }
}

pub struct Modem<T: ModemParams> {
    carrier_samples: usize,
    current_iter: Option<SampleIter<T>>,
    phi: Phi,
    params: PhantomData<T>,
}

impl<T: ModemParams> Modem<T> {
    pub fn new() -> Self {
        Modem {
            carrier_samples: 0,
            current_iter: None,
            phi: Phi::new(),
            params: PhantomData,
        }
    }

    pub fn send(&mut self, buf: &'static [u8]) {
        self.carrier_samples = (SAMPLERATE / 20) as usize; // 50ms of carrier
        self.current_iter = Some(SampleIter::new(buf));
    }

    pub fn reset(&mut self) {
        self.carrier_samples = 0;
        self.current_iter = None;
    }
}

unsafe fn slice_as_bytes<'a, T>(slice: &'a [T]) -> &'a [u8] {
    let p: *const T = &slice[0];
    slice::from_raw_parts(p as *const u8, slice.len() * mem::size_of::<T>())
}

impl<T: ModemParams> Iterator for Modem<T> {
    type Item = f32;
    fn next(&mut self) -> Option<f32> {
        if self.carrier_samples != 0 {
            self.carrier_samples -= 1;
            return Some(T::ONE_W0);
        }

        self.current_iter.as_mut().and_then(|i| i.next())
    }
}

impl<T: ModemParams> UserOsc for Modem<T> {
    const PLATFORM: Platform = Platform::MinilogueXD;

    fn init(_platform: u32, _api: u32) -> Self {
        let mut modem = Modem::new();

        if cfg!(feature = "wasm_module") {
            // The WASM interface doesn't support note data in any form.
            // Default to sending something so we can test.
            modem.send(b"Hello, world!");
        }

        modem
    }

    fn cycle(&mut self, _params: &UserOscParam, buf: &mut [i32]) {
        for i in buf {
            let sig = osc_wave_scanf(unsafe { wavesA[0] }, self.phi.0);
            *i = f32_to_q31(sig);

            let w0 = self.next().unwrap_or(T::ONE_W0);
            self.phi.advance(w0);
        }
    }

    fn note_on(&mut self, params: &UserOscParam) {
        let note = params.pitch >> 8;

        let slice = unsafe {
            match note {
                60 => Some(slice_as_bytes(&schetzen_lut_f)),
                61 => Some(slice_as_bytes(&cubicsat_lut_f)),
                62 => Some(slice_as_bytes(wavesA[0])),
                63 => Some(slice_as_bytes(wavesA[1])),
                _ => None,
            }
        };

        if let Some(buf) = slice {
            self.send(buf);
        } else {
            self.reset();
        }
    }
}
