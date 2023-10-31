use std::future::Future;

cfg_rt! {
    #[track_caller]
    pub(crate) fn block_on<F: Future>(f: F) -> F::Output {
        let mut e = crate::runtime::enter::enter(false);
        e.block_on(f).unwrap()
    }
}

cfg_not_rt! {
    #[track_caller]
    pub(crate) fn block_on<F: Future>(f: F) -> F::Output {
        let mut park = crate::park::thread::CachedParkThread::new();
        park.block_on(f).unwrap()
    }
}
