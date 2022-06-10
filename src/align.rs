#[repr(C, align(4096))]
pub struct A4096;

pub struct Aligned<A, T>
where
    T: ?Sized,
{
    _alignment: [A; 0],
    value: T,
}

#[allow(non_snake_case)]
pub const fn Aligned<A, T>(value: T) -> Aligned<A, T> {
    Aligned {
        _alignment: [],
        value,
    }
}

impl<A, T> core::ops::Deref for Aligned<A, T>
where
    T: ?Sized,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
