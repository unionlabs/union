//! The `ipnetwork` crate provides a set of APIs to work with IP CIDRs in
//! Rust.
#![crate_type = "lib"]
#![deny(
    missing_debug_implementations,
    unsafe_code,
    unused_extern_crates,
    unused_import_braces
)]

use std::{convert::TryFrom, fmt, net::IpAddr, str::FromStr};

mod common;
mod ipv4;
mod ipv6;

pub use crate::common::IpNetworkError;
pub use crate::ipv4::Ipv4NetworkIterator;
pub use crate::ipv4::{ipv4_mask_to_prefix, Ipv4Network};
pub use crate::ipv6::Ipv6NetworkIterator;
pub use crate::ipv6::{ipv6_mask_to_prefix, Ipv6Network};

/// Represents a generic network range. This type can have two variants:
/// the v4 and the v6 case.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum IpNetwork {
    V4(Ipv4Network),
    V6(Ipv6Network),
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for IpNetwork {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = <String>::deserialize(deserializer)?;
        IpNetwork::from_str(&s).map_err(serde::de::Error::custom)
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for IpNetwork {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(self)
    }
}

#[cfg(feature = "schemars")]
impl schemars::JsonSchema for IpNetwork {
    fn schema_name() -> String {
        "IpNetwork".to_string()
    }

    fn json_schema(gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        schemars::schema::SchemaObject {
            metadata: Some(
                schemars::schema::Metadata {
                    ..Default::default()
                }
                .into(),
            ),
            subschemas: Some(
                schemars::schema::SubschemaValidation {
                    one_of: Some(vec![
                        schemars::schema::SchemaObject {
                            metadata: Some(
                                schemars::schema::Metadata {
                                    title: Some("v4".to_string()),
                                    ..Default::default()
                                }
                                .into(),
                            ),
                            subschemas: Some(
                                schemars::schema::SubschemaValidation {
                                    all_of: Some(vec![gen.subschema_for::<Ipv4Network>()]),
                                    ..Default::default()
                                }
                                .into(),
                            ),
                            ..Default::default()
                        }
                        .into(),
                        schemars::schema::SchemaObject {
                            metadata: Some(
                                schemars::schema::Metadata {
                                    title: Some("v6".to_string()),
                                    ..Default::default()
                                }
                                .into(),
                            ),
                            subschemas: Some(
                                schemars::schema::SubschemaValidation {
                                    all_of: Some(vec![gen.subschema_for::<Ipv6Network>()]),
                                    ..Default::default()
                                }
                                .into(),
                            ),
                            ..Default::default()
                        }
                        .into(),
                    ]),
                    ..Default::default()
                }
                .into(),
            ),
            extensions: [("x-rust-type".to_string(), "ipnetwork::IpNetwork".into())]
                .iter()
                .cloned()
                .collect(),
            ..Default::default()
        }
        .into()
    }
}

/// Represents a generic network size. For IPv4, the max size is a u32 and for IPv6, it is a u128
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum NetworkSize {
    V4(u32),
    V6(u128),
}

impl IpNetwork {
    /// Constructs a new `IpNetwork` from a given `IpAddr` and a prefix denoting the
    /// network size. If the prefix is larger than 32 (for IPv4) or 128 (for IPv6), this
    /// will raise an `IpNetworkError::InvalidPrefix` error. Support for IPv6 is not
    /// complete yet.
    pub fn new(ip: IpAddr, prefix: u8) -> Result<IpNetwork, IpNetworkError> {
        match ip {
            IpAddr::V4(a) => Ok(IpNetwork::V4(Ipv4Network::new(a, prefix)?)),
            IpAddr::V6(a) => Ok(IpNetwork::V6(Ipv6Network::new(a, prefix)?)),
        }
    }

    /// Constructs a new `IpNetwork` from a network address and a network mask.
    ///
    /// If the netmask is not valid this will return an `IpNetworkError::InvalidPrefix`.
    pub fn with_netmask(netaddr: IpAddr, netmask: IpAddr) -> Result<Self, IpNetworkError> {
        let prefix = ip_mask_to_prefix(netmask)?;
        Self::new(netaddr, prefix)
    }

    /// Returns the IP part of a given `IpNetwork`
    pub fn ip(&self) -> IpAddr {
        match *self {
            IpNetwork::V4(ref a) => IpAddr::V4(a.ip()),
            IpNetwork::V6(ref a) => IpAddr::V6(a.ip()),
        }
    }

    /// Returns the prefix of the given `IpNetwork`
    ///
    /// # Example
    /// ```
    /// use ipnetwork::IpNetwork;
    ///
    /// assert_eq!(IpNetwork::V4("10.9.0.1".parse().unwrap()).prefix(), 32u8);
    /// assert_eq!(IpNetwork::V4("10.9.0.32/16".parse().unwrap()).prefix(), 16u8);
    ///
    /// assert_eq!(IpNetwork::V6("ff01::0".parse().unwrap()).prefix(), 128u8);
    /// assert_eq!(IpNetwork::V6("ff01::0/32".parse().unwrap()).prefix(), 32u8);
    /// ```
    pub fn prefix(&self) -> u8 {
        match *self {
            IpNetwork::V4(ref a) => a.prefix(),
            IpNetwork::V6(ref a) => a.prefix(),
        }
    }

    /// Returns the address of the network denoted by this `IpNetwork`.
    /// This means the lowest possible IP address inside of the network.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::{Ipv4Addr, Ipv6Addr};
    /// use ipnetwork::IpNetwork;
    ///
    /// let net: IpNetwork = "10.1.9.32/16".parse().unwrap();
    /// assert_eq!(net.network(), Ipv4Addr::new(10, 1, 0, 0));
    /// let net: IpNetwork = "2001:db8::/96".parse().unwrap();
    /// assert_eq!(net.network(), Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 0));
    /// ```
    pub fn network(&self) -> IpAddr {
        match *self {
            IpNetwork::V4(ref a) => IpAddr::V4(a.network()),
            IpNetwork::V6(ref a) => IpAddr::V6(a.network()),
        }
    }

    /// Returns the broadcasting address of this `IpNetwork`.
    /// This means the highest possible IP address inside of the network.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::Ipv4Addr;
    /// use ipnetwork::{IpNetwork, Ipv4Network};
    ///
    /// let net: Ipv4Network = "10.9.0.32/16".parse().unwrap();
    /// assert_eq!(net.broadcast(), Ipv4Addr::new(10, 9, 255, 255));
    /// ```
    pub fn broadcast(&self) -> IpAddr {
        match *self {
            IpNetwork::V4(ref a) => IpAddr::V4(a.broadcast()),
            IpNetwork::V6(ref a) => IpAddr::V6(a.broadcast()),
        }
    }

    /// Returns the mask for this `IpNetwork`.
    /// That means the `prefix` most significant bits will be 1 and the rest 0
    ///
    /// # Example
    ///
    /// ```
    /// use ipnetwork::IpNetwork;
    /// use std::net::{Ipv4Addr, Ipv6Addr};
    ///
    /// let v4_net: IpNetwork = "10.9.0.1".parse().unwrap();
    /// assert_eq!(v4_net.mask(), Ipv4Addr::new(255, 255, 255, 255));
    /// let v4_net: IpNetwork = "10.9.0.32/16".parse().unwrap();
    /// assert_eq!(v4_net.mask(), Ipv4Addr::new(255, 255, 0, 0));
    ///
    /// let v6_net: IpNetwork = "ff01::0".parse().unwrap();
    /// assert_eq!(v6_net.mask(), Ipv6Addr::new(0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff));
    /// let v6_net: IpNetwork = "ff01::0/32".parse().unwrap();
    /// assert_eq!(v6_net.mask(), Ipv6Addr::new(0xffff, 0xffff, 0, 0, 0, 0, 0, 0));
    /// ```
    pub fn mask(&self) -> IpAddr {
        match *self {
            IpNetwork::V4(ref a) => IpAddr::V4(a.mask()),
            IpNetwork::V6(ref a) => IpAddr::V6(a.mask()),
        }
    }

    /// Returns true if the IP in this `IpNetwork` is a valid IPv4 address,
    /// false if it's a valid IPv6 address.
    ///
    /// # Example
    ///
    ///```
    /// use ipnetwork::IpNetwork;
    ///
    /// let v4: IpNetwork = IpNetwork::V4("10.9.0.32/16".parse().unwrap());
    /// assert_eq!(v4.is_ipv4(), true);
    /// assert_eq!(v4.is_ipv6(), false);
    ///```
    pub fn is_ipv4(&self) -> bool {
        match *self {
            IpNetwork::V4(_) => true,
            IpNetwork::V6(_) => false,
        }
    }

    /// Returns true if the IP in this `IpNetwork` is a valid IPv6 address,
    /// false if it's a valid IPv4 address.
    ///
    /// # Example
    ///
    ///```
    /// use ipnetwork::IpNetwork;
    ///
    /// let v6: IpNetwork = IpNetwork::V6("ff01::0/32".parse().unwrap());
    /// assert_eq!(v6.is_ipv6(), true);
    /// assert_eq!(v6.is_ipv4(), false);
    ///```
    pub fn is_ipv6(&self) -> bool {
        match *self {
            IpNetwork::V4(_) => false,
            IpNetwork::V6(_) => true,
        }
    }

    // TODO(abhishek) when TryFrom is stable, implement it for IpNetwork to
    // variant conversions. Then use that to implement a generic is_subnet_of
    // is_supernet_of, overlaps

    /// Checks if a given `IpAddr` is in this `IpNetwork`
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::IpAddr;
    /// use ipnetwork::IpNetwork;
    ///
    /// let net: IpNetwork = "127.0.0.0/24".parse().unwrap();
    /// let ip1: IpAddr = "127.0.0.1".parse().unwrap();
    /// let ip2: IpAddr = "172.0.0.1".parse().unwrap();
    /// let ip4: IpAddr = "::1".parse().unwrap();
    /// assert!(net.contains(ip1));
    /// assert!(!net.contains(ip2));
    /// assert!(!net.contains(ip4));
    /// ```
    #[inline]
    pub fn contains(&self, ip: IpAddr) -> bool {
        match (*self, ip) {
            (IpNetwork::V4(net), IpAddr::V4(ip)) => net.contains(ip),
            (IpNetwork::V6(net), IpAddr::V6(ip)) => net.contains(ip),
            _ => false,
        }
    }

    /// Returns the number of possible host addresses in this `IpAddr`
    ///
    /// # Examples
    ///
    /// ```
    /// use ipnetwork::{IpNetwork, NetworkSize};
    ///
    ///
    /// let net: IpNetwork = "127.0.0.0/24".parse().unwrap();
    /// assert_eq!(net.size(), NetworkSize::V4(256))
    /// ```
    pub fn size(&self) -> NetworkSize {
        match *self {
            IpNetwork::V4(ref ip) => NetworkSize::V4(ip.size()),
            IpNetwork::V6(ref ip) => NetworkSize::V6(ip.size()),
        }
    }

    /// Returns an iterator over the addresses contained in the network.
    ///
    /// This lists all the addresses in the network range, in ascending order.
    pub fn iter(&self) -> IpNetworkIterator {
        let inner = match self {
            IpNetwork::V4(ip) => IpNetworkIteratorInner::V4(ip.iter()),
            IpNetwork::V6(ip) => IpNetworkIteratorInner::V6(ip.iter()),
        };
        IpNetworkIterator { inner }
    }
}

/// Tries to parse the given string into a `IpNetwork`. Will first try to parse
/// it as an `Ipv4Network` and if that fails as an `Ipv6Network`. If both
/// fails it will return an `InvalidAddr` error.
///
/// # Examples
///
/// ```
/// use std::net::Ipv4Addr;
/// use ipnetwork::{IpNetwork, Ipv4Network};
///
/// let expected = IpNetwork::V4(Ipv4Network::new(Ipv4Addr::new(10, 1, 9, 32), 16).unwrap());
/// let from_cidr: IpNetwork = "10.1.9.32/16".parse().unwrap();
/// assert_eq!(expected, from_cidr);
/// ```
impl FromStr for IpNetwork {
    type Err = IpNetworkError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(net) = Ipv4Network::from_str(s) {
            Ok(IpNetwork::V4(net))
        } else if let Ok(net) = Ipv6Network::from_str(s) {
            Ok(IpNetwork::V6(net))
        } else {
            Err(IpNetworkError::InvalidAddr(s.to_string()))
        }
    }
}

impl TryFrom<&str> for IpNetwork {
    type Error = IpNetworkError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        IpNetwork::from_str(s)
    }
}

impl From<Ipv4Network> for IpNetwork {
    fn from(v4: Ipv4Network) -> IpNetwork {
        IpNetwork::V4(v4)
    }
}

impl From<Ipv6Network> for IpNetwork {
    fn from(v6: Ipv6Network) -> IpNetwork {
        IpNetwork::V6(v6)
    }
}

impl From<IpAddr> for IpNetwork {
    fn from(addr: IpAddr) -> IpNetwork {
        match addr {
            IpAddr::V4(a) => IpNetwork::V4(Ipv4Network::from(a)),
            IpAddr::V6(a) => IpNetwork::V6(Ipv6Network::from(a)),
        }
    }
}

impl fmt::Display for IpNetwork {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            IpNetwork::V4(net) => net.fmt(f),
            IpNetwork::V6(net) => net.fmt(f),
        }
    }
}

#[derive(Clone, Debug)]
enum IpNetworkIteratorInner {
    V4(Ipv4NetworkIterator),
    V6(Ipv6NetworkIterator),
}

#[derive(Clone, Debug)]
pub struct IpNetworkIterator {
    inner: IpNetworkIteratorInner,
}

impl Iterator for IpNetworkIterator {
    type Item = IpAddr;
    fn next(&mut self) -> Option<IpAddr> {
        match &mut self.inner {
            IpNetworkIteratorInner::V4(iter) => iter.next().map(IpAddr::V4),
            IpNetworkIteratorInner::V6(iter) => iter.next().map(IpAddr::V6),
        }
    }
}

impl IntoIterator for &'_ IpNetwork {
    type IntoIter = IpNetworkIterator;
    type Item = IpAddr;
    fn into_iter(self) -> IpNetworkIterator {
        self.iter()
    }
}

/// Converts a `IpAddr` network mask into a prefix.
/// If the mask is invalid this will return an `IpNetworkError::InvalidPrefix`.
pub fn ip_mask_to_prefix(mask: IpAddr) -> Result<u8, IpNetworkError> {
    match mask {
        IpAddr::V4(mask) => ipv4_mask_to_prefix(mask),
        IpAddr::V6(mask) => ipv6_mask_to_prefix(mask),
    }
}

#[cfg(test)]
mod test {
    #[test]
    #[cfg(feature = "serde")]
    fn deserialize_from_serde_json_value() {
        use super::*;
        let network = IpNetwork::from_str("0.0.0.0/0").unwrap();
        let val: serde_json::value::Value =
            serde_json::from_str(&serde_json::to_string(&network).unwrap()).unwrap();
        let _deser: IpNetwork = serde_json::from_value(val)
            .expect("Fails to deserialize from json_value::value::Value");
    }
}
