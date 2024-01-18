#[const_trait]
pub trait ConstInto<T>: Sized {
    #[must_use]
    fn const_into(self) -> T;
}

#[const_trait]
pub trait ConstFrom<T>: Sized {
    #[must_use]
    fn const_from(value: T) -> Self;
}
