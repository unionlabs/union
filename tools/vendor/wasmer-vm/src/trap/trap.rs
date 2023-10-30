use backtrace::Backtrace;
use std::error::Error;
use wasmer_types::TrapCode;

/// Stores trace message with backtrace.
#[derive(Debug)]
pub enum Trap {
    /// A user-raised trap through `raise_user_trap`.
    User(Box<dyn Error + Send + Sync>),

    /// A trap raised from the Wasm generated code
    ///
    /// Note: this trap is deterministic (assuming a deterministic host implementation)
    Wasm {
        /// The program counter in generated code where this trap happened.
        pc: usize,
        /// Native stack backtrace at the time the trap occurred
        backtrace: Backtrace,
        /// Optional trapcode associated to the signal that caused the trap
        signal_trap: Option<TrapCode>,
    },

    /// A trap raised from a wasm libcall
    ///
    /// Note: this trap is deterministic (assuming a deterministic host implementation)
    Lib {
        /// Code of the trap.
        trap_code: TrapCode,
        /// Native stack backtrace at the time the trap occurred
        backtrace: Backtrace,
    },

    /// A trap indicating that the runtime was unable to allocate sufficient memory.
    ///
    /// Note: this trap is nondeterministic, since it depends on the host system.
    OOM {
        /// Native stack backtrace at the time the OOM occurred
        backtrace: Backtrace,
    },
}

impl Trap {
    /// Construct a new Wasm trap with the given source location and backtrace.
    ///
    /// Internally saves a backtrace when constructed.
    pub fn wasm(pc: usize, backtrace: Backtrace, signal_trap: Option<TrapCode>) -> Self {
        Trap::Wasm {
            pc,
            backtrace,
            signal_trap,
        }
    }

    /// Construct a new Wasm trap with the given trap code.
    ///
    /// Internally saves a backtrace when constructed.
    pub fn lib(trap_code: TrapCode) -> Self {
        let backtrace = Backtrace::new_unresolved();
        Trap::Lib {
            trap_code,
            backtrace,
        }
    }

    /// Construct a new OOM trap with the given source location and trap code.
    ///
    /// Internally saves a backtrace when constructed.
    pub fn oom() -> Self {
        let backtrace = Backtrace::new_unresolved();
        Trap::OOM { backtrace }
    }
}
