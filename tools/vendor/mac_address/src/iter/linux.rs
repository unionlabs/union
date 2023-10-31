use crate::{MacAddress, MacAddressError};
use nix::{ifaddrs, sys::socket::SockAddr};

/// An iterator over all available MAC addresses on the system.
pub struct MacAddressIterator {
    iter: std::iter::FilterMap<
        ifaddrs::InterfaceAddressIterator,
        fn(ifaddrs::InterfaceAddress) -> Option<MacAddress>,
    >,
}

impl MacAddressIterator {
    /// Creates a new `MacAddressIterator`.
    pub fn new() -> Result<MacAddressIterator, MacAddressError> {
        Ok(Self {
            iter: ifaddrs::getifaddrs()?.filter_map(filter_macs),
        })
    }
}

fn filter_macs(intf: ifaddrs::InterfaceAddress) -> Option<MacAddress> {
    if let SockAddr::Link(link) = intf.address? {
        Some(MacAddress::new(link.addr()))
    } else {
        None
    }
}

impl Iterator for MacAddressIterator {
    type Item = MacAddress;

    fn next(&mut self) -> Option<MacAddress> {
        self.iter.next()
    }
}
