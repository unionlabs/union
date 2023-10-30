// default pin_project! is completely safe.

::pin_project_lite::pin_project! {
    #[derive(Debug)]
    pub struct DefaultStruct<T, U> {
        #[pin]
        pub pinned: T,
        pub unpinned: U,
    }
}

::pin_project_lite::pin_project! {
    #[project = DefaultStructProj]
    #[project_ref = DefaultStructProjRef]
    #[derive(Debug)]
    pub struct DefaultStructNamed<T, U> {
        #[pin]
        pub pinned: T,
        pub unpinned: U,
    }
}

::pin_project_lite::pin_project! {
    #[project = DefaultEnumProj]
    #[project_ref = DefaultEnumProjRef]
    #[derive(Debug)]
    pub enum DefaultEnum<T, U> {
        Struct {
            #[pin]
            pinned: T,
            unpinned: U,
        },
        Unit,
    }
}
