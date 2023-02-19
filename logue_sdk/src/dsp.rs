use core::mem::transmute;

pub fn f32_to_q31(x: f32) -> i32 {
    unsafe { (x * 0x7fffffff as f32).to_int_unchecked() }
}

pub fn param_val_to_f32(val: u16) -> f32 {
    val as f32 * 9.77517106549365e-004f32
}

pub fn linintf(fr: f32, x0: f32, x1: f32) -> f32 {
    x0 + fr * (x1 - x0)
}

pub fn si_roundf(x: f32) -> f32 {
    unsafe { (x + si_copysign(0.5, x)).to_int_unchecked::<i32>() as f32 }
}

pub fn si_copysign(x: f32, y: f32) -> f32 {
    let mut xi: u32 = unsafe { transmute(x) };
    let yi: u32 = unsafe { transmute(y) };

    xi &= 0x7fffffff;
    xi |= yi & 0x80000000;

    unsafe { transmute(xi) }
}