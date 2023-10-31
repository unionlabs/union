//! I/O functions

pub use self::dma::*;
pub use self::io::*;
pub use self::mmio::*;

#[cfg(target_arch = "x86_64")]
pub use self::pio::*;

mod dma;
mod io;
mod mmio;

#[cfg(target_arch = "x86_64")]
mod pio;
