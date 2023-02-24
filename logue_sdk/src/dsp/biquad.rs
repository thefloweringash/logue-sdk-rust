struct Coeffs {
    ff0: f32,
    ff1: f32,
    ff2: f32,
    fb1: f32,
    fb2: f32,
}

impl Default for Coeffs {
    fn default() -> Self {
        Self {
            ff0: 0.0,
            ff1: 0.0,
            ff2: 0.0,
            fb1: 0.0,
            fb2: 0.0,
        }
    }
}

impl Coeffs {
    #[inline(always)]
    fn wc(fc: f32, fsrecip: f32) -> f32 {
        fc * fsrecip
    }

    fn set_pole_lp(&mut self, pole: f32) {
        self.ff0 = 1.0 - pole;
        self.fb1 = -pole;
        self.fb2 = 0.0;
        self.ff2 = 0.0;
        self.ff1 = 0.0;
    }

    fn set_pole_hp(&mut self, pole: f32) {
        self.ff0 = 1.0;
        self.ff1 = -1.0;
        self.fb1 = -pole;
        self.fb2 = 0.0;
        self.ff2 = 0.0;
    }
}

pub struct BiQuad {
    coeffs: Coeffs,
    z1: f32,
    z2: f32,
}

impl BiQuad {
    #[inline(always)]
    fn process_so(&mut self, xn: f32) -> f32 {
        let acc: f32 = self.coeffs.ff0 * xn + self.z1;
        self.z1 = self.coeffs.ff1 * xn + self.z2;
        self.z2 = self.coeffs.ff2 * xn;
        self.z1 -= self.coeffs.fb1 * acc;
        self.z2 -= self.coeffs.fb2 * acc;
        acc
    }

    #[inline(always)]
    fn process_fo(&mut self, xn: f32) -> f32 {
        let acc: f32 = self.coeffs.ff0 * xn + self.z1;
        self.z1 = self.coeffs.ff1 * xn;
        self.z1 -= self.coeffs.fb1 * acc;
        acc
    }

    #[inline(always)]
    fn process(&mut self, xn: f32) -> f32 {
        return self.process_so(xn);
    }
}
