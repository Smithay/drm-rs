/// Creates a structure that wraps around a raw DRM type.
macro_rules! ffi_struct {
    (
        struct $name:ident($rawty:ty) from $ioctl:expr;
    ) => (
        #[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq, From, Into)]
        pub(crate) struct $name {
            raw: $rawty
        }

        impl TryFromDevice {
            fn try_from_device(fd: c_int) -> Result<Self> {
                let mut t = Self::default();
                unsafe {
                    
                }
            }
        }
    ),
    (
        struct $name:ident {
            raw: $rawty:ty,
            $(
                $bname:ident : [$bty:ty;$blen:expr] => [$rptr:ident;$rlen:ident]
            ),*
        }
    ) => (
        #[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
        pub(crate) struct $name {
            raw: $rawty,
            $($bname: [$bty; $blen]),*
        }

        impl Default for $name {
            fn default() -> $name {
                let mut t: $name = unsafe {
                    mem::zeroed()
                };

                $(
                    t.raw.$rptr = (&mut t.$bname).as_mut_ptr();
                    t.raw.$rlen = $blen;
                )*

                t
            }
        }

        impl $name {
            $(
                pub(crate) fn $bname(&self) -> &[$bty] {
                    unsafe {
                        slice::from_raw_parts(self.raw.$rptr, self.raw.$rlen as usize)
                    }
                }
            )*
        }
    )
}

