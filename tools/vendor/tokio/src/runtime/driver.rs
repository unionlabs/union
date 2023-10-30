//! Abstracts out the entire chain of runtime sub-drivers into common types.
use crate::park::thread::ParkThread;
use crate::park::Park;

use std::io;
use std::time::Duration;

// ===== io driver =====

cfg_io_driver! {
    type IoDriver = crate::io::driver::Driver;
    type IoStack = crate::park::either::Either<ProcessDriver, ParkThread>;
    pub(crate) type IoHandle = Option<crate::io::driver::Handle>;

    fn create_io_stack(enabled: bool) -> io::Result<(IoStack, IoHandle, SignalHandle)> {
        use crate::park::either::Either;

        #[cfg(loom)]
        assert!(!enabled);

        let ret = if enabled {
            let io_driver = crate::io::driver::Driver::new()?;
            let io_handle = io_driver.handle();

            let (signal_driver, signal_handle) = create_signal_driver(io_driver)?;
            let process_driver = create_process_driver(signal_driver);

            (Either::A(process_driver), Some(io_handle), signal_handle)
        } else {
            (Either::B(ParkThread::new()), Default::default(), Default::default())
        };

        Ok(ret)
    }
}

cfg_not_io_driver! {
    pub(crate) type IoHandle = ();
    type IoStack = ParkThread;

    fn create_io_stack(_enabled: bool) -> io::Result<(IoStack, IoHandle, SignalHandle)> {
        Ok((ParkThread::new(), Default::default(), Default::default()))
    }
}

// ===== signal driver =====

macro_rules! cfg_signal_internal_and_unix {
    ($($item:item)*) => {
        #[cfg(unix)]
        cfg_signal_internal! { $($item)* }
    }
}

cfg_signal_internal_and_unix! {
    type SignalDriver = crate::signal::unix::driver::Driver;
    pub(crate) type SignalHandle = Option<crate::signal::unix::driver::Handle>;

    fn create_signal_driver(io_driver: IoDriver) -> io::Result<(SignalDriver, SignalHandle)> {
        let driver = crate::signal::unix::driver::Driver::new(io_driver)?;
        let handle = driver.handle();
        Ok((driver, Some(handle)))
    }
}

cfg_not_signal_internal! {
    pub(crate) type SignalHandle = ();

    cfg_io_driver! {
        type SignalDriver = IoDriver;

        fn create_signal_driver(io_driver: IoDriver) -> io::Result<(SignalDriver, SignalHandle)> {
            Ok((io_driver, ()))
        }
    }
}

// ===== process driver =====

cfg_process_driver! {
    type ProcessDriver = crate::process::unix::driver::Driver;

    fn create_process_driver(signal_driver: SignalDriver) -> ProcessDriver {
        crate::process::unix::driver::Driver::new(signal_driver)
    }
}

cfg_not_process_driver! {
    cfg_io_driver! {
        type ProcessDriver = SignalDriver;

        fn create_process_driver(signal_driver: SignalDriver) -> ProcessDriver {
            signal_driver
        }
    }
}

// ===== time driver =====

cfg_time! {
    type TimeDriver = crate::park::either::Either<crate::time::driver::Driver<IoStack>, IoStack>;

    pub(crate) type Clock = crate::time::Clock;
    pub(crate) type TimeHandle = Option<crate::time::driver::Handle>;

    fn create_clock(enable_pausing: bool, start_paused: bool) -> Clock {
        crate::time::Clock::new(enable_pausing, start_paused)
    }

    fn create_time_driver(
        enable: bool,
        io_stack: IoStack,
        clock: Clock,
    ) -> (TimeDriver, TimeHandle) {
        use crate::park::either::Either;

        if enable {
            let driver = crate::time::driver::Driver::new(io_stack, clock);
            let handle = driver.handle();

            (Either::A(driver), Some(handle))
        } else {
            (Either::B(io_stack), None)
        }
    }
}

cfg_not_time! {
    type TimeDriver = IoStack;

    pub(crate) type Clock = ();
    pub(crate) type TimeHandle = ();

    fn create_clock(_enable_pausing: bool, _start_paused: bool) -> Clock {
        ()
    }

    fn create_time_driver(
        _enable: bool,
        io_stack: IoStack,
        _clock: Clock,
    ) -> (TimeDriver, TimeHandle) {
        (io_stack, ())
    }
}

// ===== runtime driver =====

#[derive(Debug)]
pub(crate) struct Driver {
    inner: TimeDriver,
}

pub(crate) struct Resources {
    pub(crate) io_handle: IoHandle,
    pub(crate) signal_handle: SignalHandle,
    pub(crate) time_handle: TimeHandle,
    pub(crate) clock: Clock,
}

pub(crate) struct Cfg {
    pub(crate) enable_io: bool,
    pub(crate) enable_time: bool,
    pub(crate) enable_pause_time: bool,
    pub(crate) start_paused: bool,
}

impl Driver {
    pub(crate) fn new(cfg: Cfg) -> io::Result<(Self, Resources)> {
        let (io_stack, io_handle, signal_handle) = create_io_stack(cfg.enable_io)?;

        let clock = create_clock(cfg.enable_pause_time, cfg.start_paused);

        let (time_driver, time_handle) =
            create_time_driver(cfg.enable_time, io_stack, clock.clone());

        Ok((
            Self { inner: time_driver },
            Resources {
                io_handle,
                signal_handle,
                time_handle,
                clock,
            },
        ))
    }
}

impl Park for Driver {
    type Unpark = <TimeDriver as Park>::Unpark;
    type Error = <TimeDriver as Park>::Error;

    fn unpark(&self) -> Self::Unpark {
        self.inner.unpark()
    }

    fn park(&mut self) -> Result<(), Self::Error> {
        self.inner.park()
    }

    fn park_timeout(&mut self, duration: Duration) -> Result<(), Self::Error> {
        self.inner.park_timeout(duration)
    }

    fn shutdown(&mut self) {
        self.inner.shutdown()
    }
}
