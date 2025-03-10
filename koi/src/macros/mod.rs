pub trait Convert<To> {
    fn convert(self) -> To;
}

macro_rules! convert {
    ($a:ty, $b:ty) => {
        impl Convert<$b> for $a {
            #[inline(always)]
            fn convert(self) -> $b {
                unsafe {
                    let mut result: $b = core::mem::zeroed();
                    core::ptr::copy_nonoverlapping(
                        &self as *const $a as *const u8,
                        &mut result as *mut $b as *mut u8,
                        core::mem::size_of::<$b>(),
                    );
                    return result;
                }
            }
        }
        impl Convert<$a> for $b {
            #[inline(always)]
            fn convert(self) -> $a {
                unsafe {
                    let mut result: $a = core::mem::zeroed();
                    core::ptr::copy_nonoverlapping(
                        &self as *const $b as *const u8,
                        &mut result as *mut $a as *mut u8,
                        core::mem::size_of::<$a>(),
                    );
                    return result;
                }
            }
        }
    };
}
convert!([f32; 2], [u8; 8]);
convert!([f32; 4], [u8; 16]);