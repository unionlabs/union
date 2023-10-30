// default #[pin_project], PinnedDrop, project_replace, !Unpin, and UnsafeUnpin without UnsafeUnpin impl are completely safe.

#[::pin_project::pin_project]
#[derive(Debug)]
pub struct DefaultStruct<T, U> {
    #[pin]
    pub pinned: T,
    pub unpinned: U,
}

#[::pin_project::pin_project(
    project = DefaultStructNamedProj,
    project_ref = DefaultStructNamedProjRef,
)]
#[derive(Debug)]
pub struct DefaultStructNamed<T, U> {
    #[pin]
    pub pinned: T,
    pub unpinned: U,
}

#[::pin_project::pin_project]
#[derive(Debug)]
pub struct DefaultTupleStruct<T, U>(#[pin] pub T, pub U);

#[::pin_project::pin_project(
    project = DefaultTupleStructNamedProj,
    project_ref = DefaultTupleStructNamedProjRef,
)]
#[derive(Debug)]
pub struct DefaultTupleStructNamed<T, U>(#[pin] pub T, pub U);

#[::pin_project::pin_project(
    project = DefaultEnumProj,
    project_ref = DefaultEnumProjRef,
)]
#[derive(Debug)]
pub enum DefaultEnum<T, U> {
    Struct {
        #[pin]
        pinned: T,
        unpinned: U,
    },
    Tuple(#[pin] T, U),
    Unit,
}

#[::pin_project::pin_project(PinnedDrop)]
#[derive(Debug)]
pub struct PinnedDropStruct<T, U> {
    #[pin]
    pub pinned: T,
    pub unpinned: U,
}

#[::pin_project::pinned_drop]
impl<T, U> PinnedDrop for PinnedDropStruct<T, U> {
    fn drop(self: ::pin_project::__private::Pin<&mut Self>) {}
}

#[::pin_project::pin_project(PinnedDrop)]
#[derive(Debug)]
pub struct PinnedDropTupleStruct<T, U>(#[pin] pub T, pub U);

#[::pin_project::pinned_drop]
impl<T, U> PinnedDrop for PinnedDropTupleStruct<T, U> {
    fn drop(self: ::pin_project::__private::Pin<&mut Self>) {}
}

#[::pin_project::pin_project(
    PinnedDrop,
    project = PinnedDropEnumProj,
    project_ref = PinnedDropEnumProjRef,
)]
#[derive(Debug)]
pub enum PinnedDropEnum<T, U> {
    Struct {
        #[pin]
        pinned: T,
        unpinned: U,
    },
    Tuple(#[pin] T, U),
    Unit,
}

#[::pin_project::pinned_drop]
impl<T, U> PinnedDrop for PinnedDropEnum<T, U> {
    fn drop(self: ::pin_project::__private::Pin<&mut Self>) {}
}

#[::pin_project::pin_project(project_replace)]
#[derive(Debug)]
pub struct ReplaceStruct<T, U> {
    #[pin]
    pub pinned: T,
    pub unpinned: U,
}

#[::pin_project::pin_project(
    project = ReplaceStructNamedProj,
    project_ref = ReplaceStructNamedProjRef,
    project_replace = ReplaceStructNamedProjOwn,
)]
#[derive(Debug)]
pub struct ReplaceStructNamed<T, U> {
    #[pin]
    pub pinned: T,
    pub unpinned: U,
}

#[::pin_project::pin_project(project_replace)]
#[derive(Debug)]
pub struct ReplaceTupleStruct<T, U>(#[pin] pub T, pub U);

#[::pin_project::pin_project(
    project = ReplaceTupleStructNamedProj,
    project_ref = ReplaceTupleStructNamedProjRef,
    project_replace = ReplaceTupleStructNamedProjOwn,
)]
#[derive(Debug)]
pub struct ReplaceTupleStructNamed<T, U>(#[pin] pub T, pub U);

#[::pin_project::pin_project(
    project = ReplaceEnumProj,
    project_ref = ReplaceEnumProjRef,
    project_replace = ReplaceEnumProjOwn,
)]
#[derive(Debug)]
pub enum ReplaceEnum<T, U> {
    Struct {
        #[pin]
        pinned: T,
        unpinned: U,
    },
    Tuple(#[pin] T, U),
    Unit,
}

#[::pin_project::pin_project(UnsafeUnpin)]
#[derive(Debug)]
pub struct UnsafeUnpinStruct<T, U> {
    #[pin]
    pub pinned: T,
    pub unpinned: U,
}

#[::pin_project::pin_project(UnsafeUnpin)]
#[derive(Debug)]
pub struct UnsafeUnpinTupleStruct<T, U>(#[pin] pub T, pub U);

#[::pin_project::pin_project(
    UnsafeUnpin,
    project = UnsafeUnpinEnumProj,
    project_ref = UnsafeUnpinEnumProjRef,
)]
#[derive(Debug)]
pub enum UnsafeUnpinEnum<T, U> {
    Struct {
        #[pin]
        pinned: T,
        unpinned: U,
    },
    Tuple(#[pin] T, U),
    Unit,
}

#[::pin_project::pin_project(!Unpin)]
#[derive(Debug)]
pub struct NotUnpinStruct<T, U> {
    #[pin]
    pub pinned: T,
    pub unpinned: U,
}

#[::pin_project::pin_project(!Unpin)]
#[derive(Debug)]
pub struct NotUnpinTupleStruct<T, U>(#[pin] pub T, pub U);

#[::pin_project::pin_project(
    !Unpin,
    project = NotUnpinEnumProj,
    project_ref = NotUnpinEnumProjRef,
)]
#[derive(Debug)]
pub enum NotUnpinEnum<T, U> {
    Struct {
        #[pin]
        pinned: T,
        unpinned: U,
    },
    Tuple(#[pin] T, U),
    Unit,
}
