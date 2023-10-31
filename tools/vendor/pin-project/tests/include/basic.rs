include!("basic-safe-part.rs");

#[allow(clippy::undocumented_unsafe_blocks)]
unsafe impl<T: ::pin_project::__private::Unpin, U: ::pin_project::__private::Unpin>
    ::pin_project::UnsafeUnpin for UnsafeUnpinStruct<T, U>
{
}
#[allow(clippy::undocumented_unsafe_blocks)]
unsafe impl<T: ::pin_project::__private::Unpin, U: ::pin_project::__private::Unpin>
    ::pin_project::UnsafeUnpin for UnsafeUnpinTupleStruct<T, U>
{
}
#[allow(clippy::undocumented_unsafe_blocks)]
unsafe impl<T: ::pin_project::__private::Unpin, U: ::pin_project::__private::Unpin>
    ::pin_project::UnsafeUnpin for UnsafeUnpinEnum<T, U>
{
}
