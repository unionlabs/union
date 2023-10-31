#[cfg(all(loom, test))]
macro_rules! thread_local {
    ($(#[$attrs:meta])* $vis:vis static $name:ident: $ty:ty = const { $expr:expr } $(;)?) => {
        loom::thread_local! {
            $(#[$attrs])*
            $vis static $name: $ty = $expr;
        }
    };

    ($($tts:tt)+) => { loom::thread_local!{ $($tts)+ } }
}

#[cfg(not(tokio_no_const_thread_local))]
#[cfg(not(all(loom, test)))]
macro_rules! thread_local {
    ($($tts:tt)+) => { ::std::thread_local!{ $($tts)+ } }
}

#[cfg(tokio_no_const_thread_local)]
#[cfg(not(all(loom, test)))]
macro_rules! thread_local {
    ($(#[$attrs:meta])* $vis:vis static $name:ident: $ty:ty = const { $expr:expr } $(;)?) => {
        ::std::thread_local! {
            $(#[$attrs])*
            $vis static $name: $ty = $expr;
        }
    };

    ($($tts:tt)+) => { ::std::thread_local!{ $($tts)+ } }
}
