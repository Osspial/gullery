macro_rules! impl_slice_conversions {
    ($ty:ty) => {
        #[inline(always)]
        fn size() -> usize {
            use std::mem;
            let size = mem::size_of::<Self>() / mem::size_of::<$ty>();
            assert_eq!(0, mem::size_of::<Self>() % mem::size_of::<$ty>());
            size
        }

        #[inline(always)]
        pub fn from_raw_slice(raw: &[$ty]) -> &[Self] {
            let size = Self::size();
            assert_eq!(
                0,
                raw.len() % size,
                "raw slice length not multiple of {}",
                size
            );
            unsafe { ::std::slice::from_raw_parts(raw.as_ptr() as *const Self, raw.len() / size) }
        }

        #[inline(always)]
        pub fn from_raw_slice_mut(raw: &mut [$ty]) -> &mut [Self] {
            let size = Self::size();
            assert_eq!(
                0,
                raw.len() % size,
                "raw slice length not multiple of {}",
                size
            );
            unsafe {
                ::std::slice::from_raw_parts_mut(raw.as_mut_ptr() as *mut Self, raw.len() / size)
            }
        }

        #[inline(always)]
        pub fn to_raw_slice(slice: &[Self]) -> &[$ty] {
            let size = Self::size();
            unsafe {
                ::std::slice::from_raw_parts(slice.as_ptr() as *const $ty, slice.len() * size)
            }
        }

        #[inline(always)]
        pub fn to_raw_slice_mut(slice: &mut [Self]) -> &mut [$ty] {
            let size = Self::size();
            unsafe {
                ::std::slice::from_raw_parts_mut(slice.as_mut_ptr() as *mut $ty, slice.len() * size)
            }
        }
    };
}
