pub struct Debuggable<T: ?Sized> {
    pub text: &'static str,
    pub value: T,
}

impl<T: ?Sized> core::fmt::Debug for Debuggable<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.text)
    }
}

impl<T: ?Sized> core::ops::Deref for Debuggable<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

macro_rules! create_debuggable {
    ($($body:tt)+) => {
        crate::utils::debuggable::Debuggable {
            text: stringify!($($body)+),
            value: $($body)+,
        }
    };
}

pub(crate) use create_debuggable;
