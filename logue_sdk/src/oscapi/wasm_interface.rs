#[macro_export]
macro_rules! user_osc_wasm_functions {
    ($osc:ty) => {
        static mut INSTANCE: core::mem::MaybeUninit<$osc> = core::mem::MaybeUninit::uninit();

        #[no_mangle]
        extern "C" fn init(platform: u32, api: u32) {
            unsafe {
                $crate::oscapi::init_cb(&mut INSTANCE, platform, api);
            }
        }

        // This crate is no_std unless targeting wasm, so any code using std
        // has to be snuck through with a macro.
        #[no_mangle]
        extern "C" fn cycle(
            // params: *const $crate::oscapi::UserOscParam,
            buf: *mut f32,
            frames: i32,
        ) {
            let osc = unsafe { INSTANCE.assume_init_mut() };
            let frames: usize = frames.try_into().unwrap();

            // TODO: maybe use bindgen for creating osc params?
            // TODO: what do upstream do?
            let mut params = $crate::oscapi::UserOscParam::default();
            params.pitch = 0x40_00;

            let mut isamples: Vec<i32> = vec![0; frames];
            <$osc as $crate::oscapi::UserOsc>::cycle(osc, &params, &mut isamples);

            let samples = unsafe { std::slice::from_raw_parts_mut(buf, frames) };
            for i in 0..frames {
                samples[i] = $crate::dsp::q31_to_f32(isamples[i])
            }
        }

        #[no_mangle]
        extern "C" fn on(params: *const $crate::oscapi::UserOscParam) {
            unsafe {
                $crate::oscapi::on_cb(&mut INSTANCE, params);
            }
        }

        #[no_mangle]
        extern "C" fn off(params: *const $crate::oscapi::UserOscParam) {
            unsafe {
                $crate::oscapi::off_cb(&mut INSTANCE, params);
            }
        }

        #[no_mangle]
        extern "C" fn mute(params: *const $crate::oscapi::UserOscParam) {
            unsafe {
                $crate::oscapi::mute_cb(&mut INSTANCE, params);
            }
        }

        #[no_mangle]
        extern "C" fn value(value: u16) {
            unsafe {
                $crate::oscapi::value_cb(&mut INSTANCE, value);
            }
        }

        #[no_mangle]
        extern "C" fn param(idx: u16, value: u16) {
            unsafe {
                $crate::oscapi::param_cb(&mut INSTANCE, idx, value);
            }
        }

        #[no_mangle]
        pub extern "C" fn allocate_sample_buffer(capacity: usize) -> *mut f32 {
            let mut vec = Vec::with_capacity(capacity);
            let bytes = vec.as_mut_ptr();
            std::mem::forget(vec);
            bytes
        }
    };
}
