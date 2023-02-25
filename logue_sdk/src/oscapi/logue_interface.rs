#[macro_export]
macro_rules! user_osc_hooks {
    ($osc:ty) => {
        static mut INSTANCE: core::mem::MaybeUninit<$osc> = core::mem::MaybeUninit::uninit();

        extern "C" fn func_entry(platform: u32, api: u32) {
            unsafe {
                $crate::oscapi::init_cb(&mut INSTANCE, platform, api);
            }
        }

        extern "C" fn func_cycle(
            params: *const $crate::oscapi::UserOscParam,
            buf: *mut i32,
            frames: i32,
        ) {
            unsafe {
                $crate::oscapi::cycle_cb(&mut INSTANCE, params, buf, frames);
            }
        }

        extern "C" fn func_on(params: *const $crate::oscapi::UserOscParam) {
            unsafe {
                $crate::oscapi::on_cb(&mut INSTANCE, params);
            }
        }

        extern "C" fn func_off(params: *const $crate::oscapi::UserOscParam) {
            unsafe {
                $crate::oscapi::off_cb(&mut INSTANCE, params);
            }
        }

        extern "C" fn func_mute(params: *const $crate::oscapi::UserOscParam) {
            unsafe {
                $crate::oscapi::mute_cb(&mut INSTANCE, params);
            }
        }

        extern "C" fn func_value(value: u16) {
            unsafe {
                $crate::oscapi::value_cb(&mut INSTANCE, value);
            }
        }

        extern "C" fn func_param(idx: u16, value: u16) {
            unsafe {
                $crate::oscapi::param_cb(&mut INSTANCE, idx, value);
            }
        }

        #[link_section = ".hooks"]
        #[no_mangle]
        static hook_table: $crate::oscapi::UserOscHookTable = $crate::oscapi::UserOscHookTable {
            magic: [b'U', b'O', b'S', b'C'],
            api: 0x01_01_00,
            platform: <$osc as $crate::oscapi::UserOsc>::PLATFORM as u8,
            reserved0: [0, 0, 0, 0, 0, 0, 0],
            func_entry,
            func_cycle,
            func_on,
            func_off,
            func_mute,
            func_value,
            func_param,
        };
    };
}
