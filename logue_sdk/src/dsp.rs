#[inline(always)]
pub fn f32_to_q31(x: f32) -> i32 {
    unsafe { (x * 0x7fffffff as f32).to_int_unchecked() }
}

#[inline(always)]
pub fn param_val_to_f32(val: u16) -> f32 {
    val as f32 * 9.775_171e-4_f32
}

#[inline(always)]
pub fn linintf(fr: f32, x0: f32, x1: f32) -> f32 {
    x0 + fr * (x1 - x0)
}

#[inline(always)]
pub fn si_roundf(x: f32) -> f32 {
    unsafe { (x + si_copysign(0.5, x)).to_int_unchecked::<i32>() as f32 }
}

#[inline(always)]
pub fn si_copysign(x: f32, y: f32) -> f32 {
    let mut xi = x.to_bits();
    let yi: u32 = y.to_bits();

    xi &= 0x7fffffff;
    xi |= yi & 0x80000000;

    f32::from_bits(xi)
}
