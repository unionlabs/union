use crate::common::{cidr_parts, parse_prefix, IpNetworkError};
use std::{convert::TryFrom, fmt, net::Ipv4Addr, str::FromStr};

const IPV4_BITS: u8 = 32;

/// Represents a network range where the IP addresses are of v4
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Ipv4Network {
    addr: Ipv4Addr,
    prefix: u8,
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Ipv4Network {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = <String>::deserialize(deserializer)?;
        Ipv4Network::from_str(&s).map_err(serde::de::Error::custom)
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Ipv4Network {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(self)
    }
}

#[cfg(feature = "schemars")]
impl schemars::JsonSchema for Ipv4Network {
    fn schema_name() -> String {
        "Ipv4Network".to_string()
    }

    fn json_schema(_: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        schemars::schema::SchemaObject {
            instance_type: Some(schemars::schema::InstanceType::String.into()),
            string: Some(Box::new(schemars::schema::StringValidation {
                pattern: Some(
                    concat!(
                        r#"^((25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}"#,
                        r#"(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)"#,
                        r#"\/(3[0-2]|[0-2]?[0-9])$"#,
                    )
                    .to_string(),
                ),
                ..Default::default()
            })),
            extensions: [("x-rust-type".to_string(), "ipnetwork::Ipv4Network".into())]
                .iter()
                .cloned()
                .collect(),
            ..Default::default()
        }
        .into()
    }
}

impl Ipv4Network {
    /// Constructs a new `Ipv4Network` from any `Ipv4Addr` and a prefix denoting the network size.
    ///
    /// If the prefix is larger than 32 this will return an `IpNetworkError::InvalidPrefix`.
    pub const fn new(addr: Ipv4Addr, prefix: u8) -> Result<Ipv4Network, IpNetworkError> {
        if prefix > IPV4_BITS {
            Err(IpNetworkError::InvalidPrefix)
        } else {
            Ok(Ipv4Network { addr, prefix })
        }
    }

    /// Constructs a new `Ipv4Network` from a network address and a network mask.
    ///
    /// If the netmask is not valid this will return an `IpNetworkError::InvalidPrefix`.
    pub fn with_netmask(
        netaddr: Ipv4Addr,
        netmask: Ipv4Addr,
    ) -> Result<Ipv4Network, IpNetworkError> {
        let prefix = ipv4_mask_to_prefix(netmask)?;
        let net = Self {
            addr: netaddr,
            prefix,
        };
        Ok(net)
    }

    /// Returns an iterator over `Ipv4Network`. Each call to `next` will return the next
    /// `Ipv4Addr` in the given network. `None` will be returned when there are no more
    /// addresses.
    pub fn iter(self) -> Ipv4NetworkIterator {
        let start = u32::from(self.network());
        let end = start + (self.size() - 1);
        Ipv4NetworkIterator {
            next: Some(start),
            end,
        }
    }

    pub fn ip(self) -> Ipv4Addr {
        self.addr
    }

    pub fn prefix(self) -> u8 {
        self.prefix
    }

    /// Checks if the given `Ipv4Network` is a subnet of the other.
    pub fn is_subnet_of(self, other: Ipv4Network) -> bool {
        other.ip() <= self.ip() && other.broadcast() >= self.broadcast()
    }

    /// Checks if the given `Ipv4Network` is a supernet of the other.
    pub fn is_supernet_of(self, other: Ipv4Network) -> bool {
        other.is_subnet_of(self)
    }

    /// Checks if the given `Ipv4Network` is partly contained in other.
    pub fn overlaps(self, other: Ipv4Network) -> bool {
        other.contains(self.ip())
            || (other.contains(self.broadcast())
                || (self.contains(other.ip()) || (self.contains(other.broadcast()))))
    }

    /// Returns the mask for this `Ipv4Network`.
    /// That means the `prefix` most significant bits will be 1 and the rest 0
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::Ipv4Addr;
    /// use ipnetwork::Ipv4Network;
    ///
    /// let net: Ipv4Network = "127.0.0.0".parse().unwrap();
    /// assert_eq!(net.mask(), Ipv4Addr::new(255, 255, 255, 255));
    /// let net: Ipv4Network = "127.0.0.0/16".parse().unwrap();
    /// assert_eq!(net.mask(), Ipv4Addr::new(255, 255, 0, 0));
    /// ```
    pub fn mask(self) -> Ipv4Addr {
        let prefix = self.prefix;
        let mask = !(0xffff_ffff as u64 >> prefix) as u32;
        Ipv4Addr::from(mask)
    }

    /// Returns the address of the network denoted by this `Ipv4Network`.
    /// This means the lowest possible IPv4 address inside of the network.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::Ipv4Addr;
    /// use ipnetwork::Ipv4Network;
    ///
    /// let net: Ipv4Network = "10.1.9.32/16".parse().unwrap();
    /// assert_eq!(net.network(), Ipv4Addr::new(10, 1, 0, 0));
    /// ```
    pub fn network(self) -> Ipv4Addr {
        let mask = u32::from(self.mask());
        let ip = u32::from(self.addr) & mask;
        Ipv4Addr::from(ip)
    }

    /// Returns the broadcasting address of this `Ipv4Network`.
    /// This means the highest possible IPv4 address inside of the network.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::Ipv4Addr;
    /// use ipnetwork::Ipv4Network;
    ///
    /// let net: Ipv4Network = "10.9.0.32/16".parse().unwrap();
    /// assert_eq!(net.broadcast(), Ipv4Addr::new(10, 9, 255, 255));
    /// ```
    pub fn broadcast(self) -> Ipv4Addr {
        let mask = u32::from(self.mask());
        let broadcast = u32::from(self.addr) | !mask;
        Ipv4Addr::from(broadcast)
    }

    /// Checks if a given `Ipv4Addr` is in this `Ipv4Network`
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::Ipv4Addr;
    /// use ipnetwork::Ipv4Network;
    ///
    /// let net: Ipv4Network = "127.0.0.0/24".parse().unwrap();
    /// assert!(net.contains(Ipv4Addr::new(127, 0, 0, 70)));
    /// assert!(!net.contains(Ipv4Addr::new(127, 0, 1, 70)));
    /// ```
    #[inline]
    pub fn contains(self, ip: Ipv4Addr) -> bool {
        let mask = !(0xffff_ffff as u64 >> self.prefix) as u32;
        let net = u32::from(self.addr) & mask;
        (u32::from(ip) & mask) == net
    }

    /// Returns number of possible host addresses in this `Ipv4Network`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::Ipv4Addr;
    /// use ipnetwork::Ipv4Network;
    ///
    /// let net: Ipv4Network = "10.1.0.0/16".parse().unwrap();
    /// assert_eq!(net.size(), 65536);
    ///
    /// let tinynet: Ipv4Network = "0.0.0.0/32".parse().unwrap();
    /// assert_eq!(tinynet.size(), 1);
    /// ```
    pub fn size(self) -> u32 {
        let host_bits = u32::from(IPV4_BITS - self.prefix);
        (2 as u32).pow(host_bits)
    }

    /// Returns the `n`:th address within this network.
    /// The adresses are indexed from 0 and `n` must be smaller than the size of the network.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::Ipv4Addr;
    /// use ipnetwork::Ipv4Network;
    ///
    /// let net: Ipv4Network = "192.168.0.0/24".parse().unwrap();
    /// assert_eq!(net.nth(0).unwrap(), Ipv4Addr::new(192, 168, 0, 0));
    /// assert_eq!(net.nth(15).unwrap(), Ipv4Addr::new(192, 168, 0, 15));
    /// assert!(net.nth(256).is_none());
    ///
    /// let net2: Ipv4Network = "10.0.0.0/16".parse().unwrap();
    /// assert_eq!(net2.nth(256).unwrap(), Ipv4Addr::new(10, 0, 1, 0));
    /// ```
    pub fn nth(self, n: u32) -> Option<Ipv4Addr> {
        if n < self.size() {
            let net = u32::from(self.network());
            Some(Ipv4Addr::from(net + n))
        } else {
            None
        }
    }
}

impl fmt::Display for Ipv4Network {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "{}/{}", self.ip(), self.prefix())
    }
}

/// Creates an `Ipv4Network` from parsing a string in CIDR notation.
///
/// # Examples
///
/// ```
/// use std::net::Ipv4Addr;
/// use ipnetwork::Ipv4Network;
///
/// let new = Ipv4Network::new(Ipv4Addr::new(10, 1, 9, 32), 16).unwrap();
/// let from_cidr: Ipv4Network = "10.1.9.32/16".parse().unwrap();
/// assert_eq!(new.ip(), from_cidr.ip());
/// assert_eq!(new.prefix(), from_cidr.prefix());
/// ```
impl FromStr for Ipv4Network {
    type Err = IpNetworkError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (addr_str, prefix_str) = cidr_parts(s)?;
        let addr = Ipv4Addr::from_str(addr_str)
            .map_err(|_| IpNetworkError::InvalidAddr(addr_str.to_string()))?;
        let prefix = match prefix_str {
            Some(v) => {
                if let Ok(netmask) = Ipv4Addr::from_str(v) {
                    ipv4_mask_to_prefix(netmask)?
                } else {
                    parse_prefix(v, IPV4_BITS)?
                }
            }
            None => IPV4_BITS,
        };
        Ipv4Network::new(addr, prefix)
    }
}

impl TryFrom<&str> for Ipv4Network {
    type Error = IpNetworkError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Ipv4Network::from_str(s)
    }
}

impl From<Ipv4Addr> for Ipv4Network {
    fn from(a: Ipv4Addr) -> Ipv4Network {
        Ipv4Network {
            addr: a,
            prefix: 32,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Ipv4NetworkIterator {
    next: Option<u32>,
    end: u32,
}

impl Iterator for Ipv4NetworkIterator {
    type Item = Ipv4Addr;

    fn next(&mut self) -> Option<Ipv4Addr> {
        let next = self.next?;
        self.next = if next == self.end {
            None
        } else {
            Some(next + 1)
        };
        Some(next.into())
    }
}

impl IntoIterator for &'_ Ipv4Network {
    type IntoIter = Ipv4NetworkIterator;
    type Item = Ipv4Addr;
    fn into_iter(self) -> Ipv4NetworkIterator {
        self.iter()
    }
}

/// Converts a `Ipv4Addr` network mask into a prefix.
///
/// If the mask is invalid this will return an `IpNetworkError::InvalidPrefix`.
pub fn ipv4_mask_to_prefix(mask: Ipv4Addr) -> Result<u8, IpNetworkError> {
    let mask = u32::from(mask);

    let prefix = (!mask).leading_zeros() as u8;
    if (u64::from(mask) << prefix) & 0xffff_ffff != 0 {
        Err(IpNetworkError::InvalidPrefix)
    } else {
        Ok(prefix)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::collections::HashMap;
    use std::mem;
    use std::net::Ipv4Addr;

    #[test]
    fn create_v4() {
        let cidr = Ipv4Network::new(Ipv4Addr::new(77, 88, 21, 11), 24).unwrap();
        assert_eq!(cidr.prefix(), 24);
    }

    #[test]
    fn create_v4_invalid_prefix() {
        let net = Ipv4Network::new(Ipv4Addr::new(0, 0, 0, 0), 33);
        assert!(net.is_err());
    }

    #[test]
    fn parse_v4_24bit() {
        let cidr: Ipv4Network = "127.1.0.0/24".parse().unwrap();
        assert_eq!(cidr.ip(), Ipv4Addr::new(127, 1, 0, 0));
        assert_eq!(cidr.prefix(), 24);
    }

    #[test]
    fn parse_v4_32bit() {
        let cidr: Ipv4Network = "127.0.0.0/32".parse().unwrap();
        assert_eq!(cidr.ip(), Ipv4Addr::new(127, 0, 0, 0));
        assert_eq!(cidr.prefix(), 32);
    }

    #[test]
    fn parse_v4_noprefix() {
        let cidr: Ipv4Network = "127.0.0.0".parse().unwrap();
        assert_eq!(cidr.ip(), Ipv4Addr::new(127, 0, 0, 0));
        assert_eq!(cidr.prefix(), 32);
    }

    #[test]
    fn parse_v4_fail_addr() {
        let cidr: Option<Ipv4Network> = "10.a.b/8".parse().ok();
        assert_eq!(None, cidr);
    }

    #[test]
    fn parse_v4_fail_addr2() {
        let cidr: Option<Ipv4Network> = "10.1.1.1.0/8".parse().ok();
        assert_eq!(None, cidr);
    }

    #[test]
    fn parse_v4_fail_addr3() {
        let cidr: Option<Ipv4Network> = "256/8".parse().ok();
        assert_eq!(None, cidr);
    }

    #[test]
    fn parse_v4_non_zero_host_bits() {
        let cidr: Ipv4Network = "10.1.1.1/24".parse().unwrap();
        assert_eq!(cidr.ip(), Ipv4Addr::new(10, 1, 1, 1));
        assert_eq!(cidr.prefix(), 24);
    }

    #[test]
    fn parse_v4_fail_prefix() {
        let cidr: Option<Ipv4Network> = "0/39".parse().ok();
        assert_eq!(None, cidr);
    }

    #[test]
    fn parse_v4_fail_two_slashes() {
        let cidr: Option<Ipv4Network> = "10.1.1.1/24/".parse().ok();
        assert_eq!(None, cidr);
    }

    #[test]
    fn nth_v4() {
        let net = Ipv4Network::new(Ipv4Addr::new(127, 0, 0, 0), 24).unwrap();
        assert_eq!(net.nth(0).unwrap(), Ipv4Addr::new(127, 0, 0, 0));
        assert_eq!(net.nth(1).unwrap(), Ipv4Addr::new(127, 0, 0, 1));
        assert_eq!(net.nth(255).unwrap(), Ipv4Addr::new(127, 0, 0, 255));
        assert!(net.nth(256).is_none());
    }

    #[test]
    fn nth_v4_fail() {
        let net = Ipv4Network::new(Ipv4Addr::new(10, 0, 0, 0), 32).unwrap();
        assert!(net.nth(1).is_none());
    }

    #[test]
    fn hash_eq_compatibility_v4() {
        let mut map = HashMap::new();
        let net = Ipv4Network::new(Ipv4Addr::new(127, 0, 0, 1), 16).unwrap();
        map.insert(net, 137);
        assert_eq!(137, map[&net]);
    }

    #[test]
    fn copy_compatibility_v4() {
        let net = Ipv4Network::new(Ipv4Addr::new(127, 0, 0, 1), 16).unwrap();
        mem::drop(net);
        assert_eq!(16, net.prefix());
    }

    #[test]
    fn mask_v4() {
        let cidr = Ipv4Network::new(Ipv4Addr::new(74, 125, 227, 0), 29).unwrap();
        let mask = cidr.mask();
        assert_eq!(mask, Ipv4Addr::new(255, 255, 255, 248));
    }

    #[test]
    fn network_v4() {
        let cidr = Ipv4Network::new(Ipv4Addr::new(10, 10, 1, 97), 23).unwrap();
        let net = cidr.network();
        assert_eq!(net, Ipv4Addr::new(10, 10, 0, 0));
    }

    #[test]
    fn broadcast_v4() {
        let cidr = Ipv4Network::new(Ipv4Addr::new(10, 10, 1, 97), 23).unwrap();
        let bcast = cidr.broadcast();
        assert_eq!(bcast, Ipv4Addr::new(10, 10, 1, 255));
    }

    #[test]
    fn contains_v4() {
        let cidr = Ipv4Network::new(Ipv4Addr::new(74, 125, 227, 0), 25).unwrap();
        let ip = Ipv4Addr::new(74, 125, 227, 4);
        assert!(cidr.contains(ip));
    }

    #[test]
    fn not_contains_v4() {
        let cidr = Ipv4Network::new(Ipv4Addr::new(10, 0, 0, 50), 24).unwrap();
        let ip = Ipv4Addr::new(10, 1, 0, 1);
        assert!(!cidr.contains(ip));
    }

    #[test]
    fn iterator_v4() {
        let cidr: Ipv4Network = "192.168.122.0/30".parse().unwrap();
        let mut iter = cidr.iter();
        assert_eq!(Ipv4Addr::new(192, 168, 122, 0), iter.next().unwrap());
        assert_eq!(Ipv4Addr::new(192, 168, 122, 1), iter.next().unwrap());
        assert_eq!(Ipv4Addr::new(192, 168, 122, 2), iter.next().unwrap());
        assert_eq!(Ipv4Addr::new(192, 168, 122, 3), iter.next().unwrap());
        assert_eq!(None, iter.next());
    }

    // Tests the entire IPv4 space to see if the iterator will stop at the correct place
    // and not overflow or wrap around. Ignored since it takes a long time to run.
    #[test]
    #[ignore]
    fn iterator_v4_huge() {
        let cidr: Ipv4Network = "0/0".parse().unwrap();
        let mut iter = cidr.iter();
        for i in 0..(u32::max_value() as u64 + 1) {
            assert_eq!(i as u32, u32::from(iter.next().unwrap()));
        }
        assert_eq!(None, iter.next());
    }

    #[test]
    fn v4_mask_to_prefix() {
        let mask = Ipv4Addr::new(255, 255, 255, 128);
        let prefix = ipv4_mask_to_prefix(mask).unwrap();
        assert_eq!(prefix, 25);
    }

    /// Parse netmask as well as prefix
    #[test]
    fn parse_netmask() {
        let from_netmask: Ipv4Network = "192.168.1.0/255.255.255.0".parse().unwrap();
        let from_prefix: Ipv4Network = "192.168.1.0/24".parse().unwrap();
        assert_eq!(from_netmask, from_prefix);
    }

    #[test]
    fn parse_netmask_broken_v4() {
        assert_eq!(
            "192.168.1.0/255.0.255.0".parse::<Ipv4Network>(),
            Err(IpNetworkError::InvalidPrefix)
        );
    }

    #[test]
    fn invalid_v4_mask_to_prefix() {
        let mask = Ipv4Addr::new(255, 0, 255, 0);
        let prefix = ipv4_mask_to_prefix(mask);
        assert!(prefix.is_err());
    }

    #[test]
    fn ipv4network_with_netmask() {
        {
            // Positive test-case.
            let addr = Ipv4Addr::new(127, 0, 0, 1);
            let mask = Ipv4Addr::new(255, 0, 0, 0);
            let net = Ipv4Network::with_netmask(addr, mask).unwrap();
            let expected = Ipv4Network::new(Ipv4Addr::new(127, 0, 0, 1), 8).unwrap();
            assert_eq!(net, expected);
        }
        {
            // Negative test-case.
            let addr = Ipv4Addr::new(127, 0, 0, 1);
            let mask = Ipv4Addr::new(255, 0, 255, 0);
            Ipv4Network::with_netmask(addr, mask).unwrap_err();
        }
    }

    #[test]
    fn ipv4network_from_ipv4addr() {
        let net = Ipv4Network::from(Ipv4Addr::new(127, 0, 0, 1));
        let expected = Ipv4Network::new(Ipv4Addr::new(127, 0, 0, 1), 32).unwrap();
        assert_eq!(net, expected);
    }

    #[test]
    fn test_send() {
        fn assert_send<T: Send>() {}
        assert_send::<Ipv4Network>();
    }

    #[test]
    fn test_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<Ipv4Network>();
    }

    // Tests from cpython https://github.com/python/cpython/blob/e9bc4172d18db9c182d8e04dd7b033097a994c06/Lib/test/test_ipaddress.py
    #[test]
    fn test_is_subnet_of() {
        let mut test_cases: HashMap<(Ipv4Network, Ipv4Network), bool> = HashMap::new();

        test_cases.insert(
            (
                "10.0.0.0/30".parse().unwrap(),
                "10.0.1.0/24".parse().unwrap(),
            ),
            false,
        );
        test_cases.insert(
            (
                "10.0.0.0/30".parse().unwrap(),
                "10.0.0.0/24".parse().unwrap(),
            ),
            true,
        );
        test_cases.insert(
            (
                "10.0.0.0/30".parse().unwrap(),
                "10.0.1.0/24".parse().unwrap(),
            ),
            false,
        );
        test_cases.insert(
            (
                "10.0.1.0/24".parse().unwrap(),
                "10.0.0.0/30".parse().unwrap(),
            ),
            false,
        );

        for (key, val) in test_cases.iter() {
            let (src, dest) = (key.0, key.1);
            assert_eq!(
                src.is_subnet_of(dest),
                *val,
                "testing with {} and {}",
                src,
                dest
            );
        }
    }

    #[test]
    fn test_is_supernet_of() {
        let mut test_cases: HashMap<(Ipv4Network, Ipv4Network), bool> = HashMap::new();

        test_cases.insert(
            (
                "10.0.0.0/30".parse().unwrap(),
                "10.0.1.0/24".parse().unwrap(),
            ),
            false,
        );
        test_cases.insert(
            (
                "10.0.0.0/30".parse().unwrap(),
                "10.0.0.0/24".parse().unwrap(),
            ),
            false,
        );
        test_cases.insert(
            (
                "10.0.0.0/30".parse().unwrap(),
                "10.0.1.0/24".parse().unwrap(),
            ),
            false,
        );
        test_cases.insert(
            (
                "10.0.0.0/24".parse().unwrap(),
                "10.0.0.0/30".parse().unwrap(),
            ),
            true,
        );

        for (key, val) in test_cases.iter() {
            let (src, dest) = (key.0, key.1);
            assert_eq!(
                src.is_supernet_of(dest),
                *val,
                "testing with {} and {}",
                src,
                dest
            );
        }
    }

    #[test]
    fn test_overlaps() {
        let other: Ipv4Network = "1.2.3.0/30".parse().unwrap();
        let other2: Ipv4Network = "1.2.2.0/24".parse().unwrap();
        let other3: Ipv4Network = "1.2.2.64/26".parse().unwrap();

        let skynet: Ipv4Network = "1.2.3.0/24".parse().unwrap();
        assert_eq!(skynet.overlaps(other), true);
        assert_eq!(skynet.overlaps(other2), false);
        assert_eq!(other2.overlaps(other3), true);
    }

    #[test]
    fn edges() {
        let low: Ipv4Network = "0.0.0.0/24".parse().unwrap();
        let low_addrs: Vec<Ipv4Addr> = low.iter().collect();
        assert_eq!(256, low_addrs.len());
        assert_eq!("0.0.0.0".parse::<Ipv4Addr>().unwrap(), low_addrs[0]);
        assert_eq!("0.0.0.255".parse::<Ipv4Addr>().unwrap(), low_addrs[255]);

        let high: Ipv4Network = "255.255.255.0/24".parse().unwrap();
        let high_addrs: Vec<Ipv4Addr> = high.iter().collect();
        assert_eq!(256, high_addrs.len());
        assert_eq!("255.255.255.0".parse::<Ipv4Addr>().unwrap(), high_addrs[0]);
        assert_eq!(
            "255.255.255.255".parse::<Ipv4Addr>().unwrap(),
            high_addrs[255]
        );
    }
}
