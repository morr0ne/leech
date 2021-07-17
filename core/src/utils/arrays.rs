pub trait ToArrayUnchecked<T, const N: usize> {
    unsafe fn to_array_unchecked(&mut self) -> [T; N];
}

/// see https://doc.rust-lang.org/src/alloc/vec/mod.rs.html#2881-2897
impl<T, const N: usize> ToArrayUnchecked<T, N> for Vec<T> {
    unsafe fn to_array_unchecked(&mut self) -> [T; N] {
        self.set_len(0);

        std::ptr::read(self.as_ptr() as *const [T; N])
    }
}

impl<T: Copy, const N: usize> ToArrayUnchecked<T, N> for &[T] {
    unsafe fn to_array_unchecked(&mut self) -> [T; N] {
        *(self.as_ptr() as *const [T; N])
    }
}

/// Builds an array of N size from bytes.
///
/// SAFETY: The caller must payload bytes must be exactly of lenght N
pub unsafe fn build_array<T: Clone, const P: usize, const N: usize>(payload: [&[T]; P]) -> [T; N] {
    let mut message = Vec::with_capacity(N);

    for p in payload {
        message.extend_from_slice(p)
    }

    message.to_array_unchecked()
}
