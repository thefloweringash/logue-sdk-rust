use modem::{ModemParams, SampleIter};

pub struct DebugModem {}

impl ModemParams for DebugModem {
    const SAMPLES_PER_BIT: usize = 1;
    const ZERO_W0: f32 = 0.0;
    const ONE_W0: f32 = 1.0;
}

pub fn main() {
    let mut iter = SampleIter::<DebugModem>::new(&[0xf0u8, 0x0f]);

    for x in iter {
        println!("{}", x);
    }
}
