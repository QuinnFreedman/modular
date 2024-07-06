pub trait DebugUnwrap {
    type Ok;

    fn assert_ok(self) -> Self::Ok;
}

impl<T, E> DebugUnwrap for Result<T, E> {
    type Ok = T;

    /// if debug-assertions are enabled, will do a checked unwrap and panic
    /// if not ok. Otherwise, will do an unsafe unchecked unwrap
    fn assert_ok(self) -> T {
        debug_assert!(self.is_ok());
        unsafe { self.unwrap_unchecked() }
    }
}

impl<T> DebugUnwrap for Option<T> {
    type Ok = T;

    /// if debug-assertions are enabled, will do a checked unwrap and panic
    /// if None. Otherwise, will do an unsafe unchecked unwrap
    fn assert_ok(self) -> T {
        debug_assert!(self.is_some());
        unsafe { self.unwrap_unchecked() }
    }
}
