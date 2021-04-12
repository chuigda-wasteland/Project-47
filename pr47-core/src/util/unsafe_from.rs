pub trait UnsafeFrom<T> {
    unsafe fn unsafe_from(data: T) -> Self;
}

pub trait UnsafeInto<T> {
    unsafe fn unsafe_into(self) -> T;
}
