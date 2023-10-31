use crate::common::{cidr_parts, parse_prefix, IpNetworkError};
use std::{cmp, convert::TryFrom, fmt, net::Ipv6Addr, str::FromStr};

const IPV6_BITS: u8 = 128;
const IPV6_SEGMENT_BITS: u8 = 16;

/// Represents a network range where the IP addresses are of v6
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Ipv6Network {
    addr: Ipv6Addr,
    prefix: u8,
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Ipv6Network {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = <String>::deserialize(deserializer)?;
        Ipv6Network::from_str(&s).map_err(serde::de::Error::custom)
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Ipv6Network {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(self)
    }
}

#[cfg(feature = "schemars")]
impl schemars::JsonSchema for Ipv6Network {
    fn schema_name() -> String {
        "Ipv6Network".to_string()
    }

    fn json_schema(_: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        schemars::schema::SchemaObject {
            instance_type: Some(schemars::schema::InstanceType::String.into()),
            string: Some(Box::new(schemars::schema::StringValidation {
                pattern: Some(
                    concat!(
                        r#"^("#,
                        r#"([0-9a-fA-F]{1,4}:){7,7}[0-9a-fA-F]{1,4}"#,
                        r#"|([0-9a-fA-F]{1,4}:){1,7}:"#,
                        r#"|([0-9a-fA-F]{1,4}:){1,6}:[0-9a-fA-F]{1,4}"#,
                        r#"|([0-9a-fA-F]{1,4}:){1,5}(:[0-9a-fA-F]{1,4}){1,2}"#,
                        r#"|([0-9a-fA-F]{1,4}:){1,4}(:[0-9a-fA-F]{1,4}){1,3}"#,
                        r#"|([0-9a-fA-F]{1,4}:){1,3}(:[0-9a-fA-F]{1,4}){1,4}"#,
                        r#"|([0-9a-fA-F]{1,4}:){1,2}(:[0-9a-fA-F]{1,4}){1,5}"#,
                        r#"|[0-9a-fA-F]{1,4}:((:[0-9a-fA-F]{1,4}){1,6})"#,
                        r#"|:((:[0-9a-fA-F]{1,4}){1,7}|:)"#,
                        r#"|fe80:(:[0-9a-fA-F]{0,4}){0,4}%[0-9a-zA-Z]{1,}"#,
                        r#"|::(ffff(:0{1,4}){0,1}:){0,1}((25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])\.){3,3}(25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])"#,
                        r#"|([0-9a-fA-F]{1,4}:){1,4}:((25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])\.){3,3}(25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])"#,
                        r#"")[/](12[0-8]|1[0-1][0-9]|[0-9]?[0-9])$"#,
                    ).to_string(),
                ),
                ..Default::default()
            })),
            extensions: [("x-rust-type".to_string(), "ipnetwork::Ipv6Network".into())]
                .iter()
                .cloned()
                .collect(),
            ..Default::default()
        }
        .into()
    }
}

impl Ipv6Network {
    /// Constructs a new `Ipv6Network` from any `Ipv6Addr` and a prefix denoting the network size.
    ///
    /// If the prefix is larger than 128 this will return an `IpNetworkError::InvalidPrefix`.
    pub const fn new(addr: Ipv6Addr, prefix: u8) -> Result<Ipv6Network, IpNetworkError> {
        if prefix > IPV6_BITS {
            Err(IpNetworkError::InvalidPrefix)
        } else {
            Ok(Ipv6Network { addr, prefix })
        }
    }

    /// Constructs a new `Ipv6Network` from a network address and a network mask.
    ///
    /// If the netmask is not valid this will return an `IpNetworkError::InvalidPrefix`.
    pub fn with_netmask(netaddr: Ipv6Addr, netmask: Ipv6Addr) -> Result<Self, IpNetworkError> {
        let prefix = ipv6_mask_to_prefix(netmask)?;
        let net = Self {
            addr: netaddr,
            prefix,
        };
        Ok(net)
    }

    /// Returns an iterator over `Ipv6Network`. Each call to `next` will return the next
    /// `Ipv6Addr` in the given network. `None` will be returned when there are no more
    /// addresses.
    pub fn iter(&self) -> Ipv6NetworkIterator {
        let dec = u128::from(self.addr);
        let max = u128::max_value();
        let prefix = self.prefix;

        let mask = max.checked_shl(u32::from(IPV6_BITS - prefix)).unwrap_or(0);
        let start: u128 = dec & mask;

        let mask = max.checked_shr(u32::from(prefix)).unwrap_or(0);
        let end: u128 = dec | mask;

        Ipv6NetworkIterator {
            next: Some(start),
            end,
        }
    }

    /// Returns the address of the network denoted by this `Ipv6Network`.
    /// This means the lowest possible IPv6 address inside of the network.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::Ipv6Addr;
    /// use ipnetwork::Ipv6Network;
    ///
    /// let net: Ipv6Network = "2001:db8::/96".parse().unwrap();
    /// assert_eq!(net.network(), Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 0));
    /// ```
    pub fn network(&self) -> Ipv6Addr {
        let mask = u128::from(self.mask());
        let ip = u128::from(self.addr) & mask;
        Ipv6Addr::from(ip)
    }

    /// Returns the broadcast address of this `Ipv6Network`.
    /// This means the highest possible IPv4 address inside of the network.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::Ipv6Addr;
    /// use ipnetwork::Ipv6Network;
    ///
    /// let net: Ipv6Network = "2001:db8::/96".parse().unwrap();
    /// assert_eq!(net.broadcast(), Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0xffff, 0xffff));
    /// ```
    pub fn broadcast(&self) -> Ipv6Addr {
        let mask = u128::from(self.mask());
        let broadcast = u128::from(self.addr) | !mask;
        Ipv6Addr::from(broadcast)
    }

    pub fn ip(&self) -> Ipv6Addr {
        self.addr
    }

    pub fn prefix(&self) -> u8 {
        self.prefix
    }

    /// Checks if the given `Ipv6Network` is a subnet of the other.
    pub fn is_subnet_of(self, other: Ipv6Network) -> bool {
        other.ip() <= self.ip() && other.broadcast() >= self.broadcast()
    }

    /// Checks if the given `Ipv6Network` is a supernet of the other.
    pub fn is_supernet_of(self, other: Ipv6Network) -> bool {
        other.is_subnet_of(self)
    }

    /// Checks if the given `Ipv6Network` is partly contained in other.
    pub fn overlaps(self, other: Ipv6Network) -> bool {
        other.contains(self.ip())
            || (other.contains(self.broadcast())
                || (self.contains(other.ip()) || (self.contains(other.broadcast()))))
    }

    /// Returns the mask for this `Ipv6Network`.
    /// That means the `prefix` most significant bits will be 1 and the rest 0
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::Ipv6Addr;
    /// use ipnetwork::Ipv6Network;
    ///
    /// let net: Ipv6Network = "ff01::0".parse().unwrap();
    /// assert_eq!(net.mask(), Ipv6Addr::new(0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff));
    /// let net: Ipv6Network = "ff01::0/32".parse().unwrap();
    /// assert_eq!(net.mask(), Ipv6Addr::new(0xffff, 0xffff, 0, 0, 0, 0, 0, 0));
    /// ```
    pub fn mask(&self) -> Ipv6Addr {
        // Ipv6Addr::from is only implemented for [u8; 16]
        let mut segments = [0; 16];
        for (i, segment) in segments.iter_mut().enumerate() {
            let bits_remaining = self.prefix.saturating_sub(i as u8 * 8);
            let set_bits = cmp::min(bits_remaining, 8);
            *segment = !(0xff as u16 >> set_bits) as u8;
        }
        Ipv6Addr::from(segments)
    }

    /// Checks if a given `Ipv6Addr` is in this `Ipv6Network`
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::Ipv6Addr;
    /// use ipnetwork::Ipv6Network;
    ///
    /// let net: Ipv6Network = "ff01::0/32".parse().unwrap();
    /// assert!(net.contains(Ipv6Addr::new(0xff01, 0, 0, 0, 0, 0, 0, 0x1)));
    /// assert!(!net.contains(Ipv6Addr::new(0xffff, 0, 0, 0, 0, 0, 0, 0x1)));
    /// ```
    #[inline]
    pub fn contains(&self, ip: Ipv6Addr) -> bool {
        let a = self.addr.segments();
        let b = ip.segments();
        let addrs = Iterator::zip(a.iter(), b.iter());
        self.mask()
            .segments()
            .iter()
            .zip(addrs)
            .all(|(mask, (a, b))| a & mask == b & mask)
    }

    /// Returns number of possible host addresses in this `Ipv6Network`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::Ipv6Addr;
    /// use ipnetwork::Ipv6Network;
    ///
    /// let net: Ipv6Network = "ff01::0/32".parse().unwrap();
    /// assert_eq!(net.size(), 79228162514264337593543950336);
    ///
    /// let tinynet: Ipv6Network = "ff01::0/128".parse().unwrap();
    /// assert_eq!(tinynet.size(), 1);
    /// ```
    pub fn size(&self) -> u128 {
        let host_bits = u32::from(IPV6_BITS - self.prefix);
        (2 as u128).pow(host_bits)
    }
}

/// Creates an `Ipv6Network` from parsing a string in CIDR notation.
///
/// # Examples
///
/// ```
/// use std::net::Ipv6Addr;
/// use ipnetwork::Ipv6Network;
///
/// let new = Ipv6Network::new(Ipv6Addr::new(0xff01, 0, 0, 0x17, 0, 0, 0, 0x2), 65).unwrap();
/// let from_cidr: Ipv6Network = "FF01:0:0:17:0:0:0:2/65".parse().unwrap();
/// assert_eq!(new.ip(), from_cidr.ip());
/// assert_eq!(new.prefix(), from_cidr.prefix());
/// ```
impl FromStr for Ipv6Network {
    type Err = IpNetworkError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (addr_str, prefix_str) = cidr_parts(s)?;
        let addr = Ipv6Addr::from_str(addr_str)
            .map_err(|_| IpNetworkError::InvalidAddr(addr_str.to_string()))?;
        let prefix = match prefix_str {
            Some(v) => parse_prefix(v, IPV6_BITS)?,
            None => IPV6_BITS,
        };
        Ipv6Network::new(addr, prefix)
    }
}

impl TryFrom<&str> for Ipv6Network {
    type Error = IpNetworkError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Ipv6Network::from_str(s)
    }
}

impl From<Ipv6Addr> for Ipv6Network {
    fn from(a: Ipv6Addr) -> Ipv6Network {
        Ipv6Network {
            addr: a,
            prefix: 128,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Ipv6NetworkIterator {
    next: Option<u128>,
    end: u128,
}

impl Iterator for Ipv6NetworkIterator {
    type Item = Ipv6Addr;

    fn next(&mut self) -> Option<Ipv6Addr> {
        let next = self.next?;
        self.next = if next == self.end {
            None
        } else {
            Some(next + 1)
        };
        Some(next.into())
    }
}

impl IntoIterator for &'_ Ipv6Network {
    type IntoIter = Ipv6NetworkIterator;
    type Item = Ipv6Addr;
    fn into_iter(self) -> Ipv6NetworkIterator {
        self.iter()
    }
}

impl fmt::Display for Ipv6Network {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "{}/{}", self.ip(), self.prefix())
    }
}

/// Converts a `Ipv6Addr` network mask into a prefix.
/// If the mask is invalid this will return an `IpNetworkError::InvalidPrefix`.
pub fn ipv6_mask_to_prefix(mask: Ipv6Addr) -> Result<u8, IpNetworkError> {
    let mask = mask.segments();
    let mut mask_iter = mask.iter();

    // Count the number of set bits from the start of the address
    let mut prefix = 0;
    for &segment in &mut mask_iter {
        if segment == 0xffff {
            prefix += IPV6_SEGMENT_BITS;
        } else if segment == 0 {
            // Prefix finishes on a segment boundary
            break;
        } else {
            let prefix_bits = (!segment).leading_zeros() as u8;
            // Check that the remainder of the bits are all unset
            if segment << prefix_bits != 0 {
                return Err(IpNetworkError::InvalidPrefix);
            }
            prefix += prefix_bits;
            break;
        }
    }

    // Now check all the remaining bits are unset
    for &segment in mask_iter {
        if segment != 0 {
            return Err(IpNetworkError::InvalidPrefix);
        }
    }

    Ok(prefix)
}

#[cfg(test)]
mod test {
    use super::*;
    use std::collections::HashMap;
    use std::net::Ipv6Addr;

    #[test]
    fn create_v6() {
        let cidr = Ipv6Network::new(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1), 24).unwrap();
        assert_eq!(cidr.prefix(), 24);
    }

    #[test]
    fn parse_netmask_broken_v6() {
        assert_eq!(
            "FF01:0:0:17:0:0:0:2/255.255.255.0".parse::<Ipv6Network>(),
            Err(IpNetworkError::InvalidPrefix)
        );
    }

    #[test]
    fn create_v6_invalid_prefix() {
        let cidr = Ipv6Network::new(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1), 129);
        assert!(cidr.is_err());
    }

    #[test]
    fn parse_v6() {
        let cidr: Ipv6Network = "::1/0".parse().unwrap();
        assert_eq!(cidr.ip(), Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1));
        assert_eq!(cidr.prefix(), 0);
    }

    #[test]
    fn parse_v6_2() {
        let cidr: Ipv6Network = "FF01:0:0:17:0:0:0:2/64".parse().unwrap();
        assert_eq!(cidr.ip(), Ipv6Addr::new(0xff01, 0, 0, 0x17, 0, 0, 0, 0x2));
        assert_eq!(cidr.prefix(), 64);
    }

    #[test]
    fn parse_v6_noprefix() {
        let cidr: Ipv6Network = "::1".parse().unwrap();
        assert_eq!(cidr.ip(), Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1));
        assert_eq!(cidr.prefix(), 128);
    }

    #[test]
    fn parse_v6_fail_addr() {
        let cidr: Option<Ipv6Network> = "2001::1::/8".parse().ok();
        assert_eq!(None, cidr);
    }

    #[test]
    fn parse_v6_fail_prefix() {
        let cidr: Option<Ipv6Network> = "::1/129".parse().ok();
        assert_eq!(None, cidr);
    }

    #[test]
    fn parse_v6_fail_two_slashes() {
        let cidr: Option<Ipv6Network> = "::1/24/".parse().ok();
        assert_eq!(None, cidr);
    }

    #[test]
    fn mask_v6() {
        let cidr = Ipv6Network::new(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0), 40).unwrap();
        let mask = cidr.mask();
        assert_eq!(mask, Ipv6Addr::new(0xffff, 0xffff, 0xff00, 0, 0, 0, 0, 0));
    }

    #[test]
    fn contains_v6() {
        let cidr = Ipv6Network::new(Ipv6Addr::new(0xff01, 0, 0, 0x17, 0, 0, 0, 0x2), 65).unwrap();
        let ip = Ipv6Addr::new(0xff01, 0, 0, 0x17, 0x7fff, 0, 0, 0x2);
        assert!(cidr.contains(ip));
    }

    #[test]
    fn not_contains_v6() {
        let cidr = Ipv6Network::new(Ipv6Addr::new(0xff01, 0, 0, 0x17, 0, 0, 0, 0x2), 65).unwrap();
        let ip = Ipv6Addr::new(0xff01, 0, 0, 0x17, 0xffff, 0, 0, 0x2);
        assert!(!cidr.contains(ip));
    }

    #[test]
    fn v6_mask_to_prefix() {
        let mask = Ipv6Addr::new(0xffff, 0xffff, 0xffff, 0, 0, 0, 0, 0);
        let prefix = ipv6_mask_to_prefix(mask).unwrap();
        assert_eq!(prefix, 48);
    }

    #[test]
    fn invalid_v6_mask_to_prefix() {
        let mask = Ipv6Addr::new(0, 0, 0xffff, 0xffff, 0, 0, 0, 0);
        let prefix = ipv6_mask_to_prefix(mask);
        assert!(prefix.is_err());
    }

    #[test]
    fn ipv6network_with_netmask() {
        {
            // Positive test-case.
            let addr = Ipv6Addr::new(0xff01, 0, 0, 0x17, 0, 0, 0, 0x2);
            let mask = Ipv6Addr::new(0xffff, 0xffff, 0xffff, 0, 0, 0, 0, 0);
            let net = Ipv6Network::with_netmask(addr, mask).unwrap();
            let expected =
                Ipv6Network::new(Ipv6Addr::new(0xff01, 0, 0, 0x17, 0, 0, 0, 0x2), 48).unwrap();
            assert_eq!(net, expected);
        }
        {
            // Negative test-case.
            let addr = Ipv6Addr::new(0xff01, 0, 0, 0x17, 0, 0, 0, 0x2);
            let mask = Ipv6Addr::new(0, 0, 0xffff, 0xffff, 0, 0, 0, 0);
            Ipv6Network::with_netmask(addr, mask).unwrap_err();
        }
    }

    #[test]
    fn iterator_v6() {
        let cidr: Ipv6Network = "2001:db8::/126".parse().unwrap();
        let mut iter = cidr.iter();
        assert_eq!(
            Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 0),
            iter.next().unwrap()
        );
        assert_eq!(
            Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 1),
            iter.next().unwrap()
        );
        assert_eq!(
            Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 2),
            iter.next().unwrap()
        );
        assert_eq!(
            Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 3),
            iter.next().unwrap()
        );
        assert_eq!(None, iter.next());
    }

    #[test]
    fn iterator_v6_tiny() {
        let cidr: Ipv6Network = "2001:db8::/128".parse().unwrap();
        let mut iter = cidr.iter();
        assert_eq!(
            Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 0),
            iter.next().unwrap()
        );
        assert_eq!(None, iter.next());
    }

    #[test]
    fn iterator_v6_huge() {
        let cidr: Ipv6Network = "2001:db8::/0".parse().unwrap();
        let mut iter = cidr.iter();
        assert_eq!(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0), iter.next().unwrap());
        assert_eq!(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1), iter.next().unwrap());
        assert_eq!(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 2), iter.next().unwrap());
    }

    #[test]
    fn network_v6() {
        let cidr: Ipv6Network = "2001:db8::0/96".parse().unwrap();
        let net = cidr.network();
        let expected: Ipv6Addr = "2001:db8::".parse().unwrap();
        assert_eq!(net, expected);
    }

    #[test]
    fn broadcast_v6() {
        let cidr: Ipv6Network = "2001:db8::0/96".parse().unwrap();
        let net = cidr.broadcast();
        let expected: Ipv6Addr = "2001:db8::ffff:ffff".parse().unwrap();
        assert_eq!(net, expected);
    }

    #[test]
    fn size_v6() {
        let cidr: Ipv6Network = "2001:db8::0/96".parse().unwrap();
        assert_eq!(cidr.size(), 4294967296);
    }

    #[test]
    fn ipv6network_from_ipv6addr() {
        let net = Ipv6Network::from(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1));
        let expected = Ipv6Network::new(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1), 128).unwrap();
        assert_eq!(net, expected);
    }

    #[test]
    fn test_send() {
        fn assert_send<T: Send>() {}
        assert_send::<Ipv6Network>();
    }

    #[test]
    fn test_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<Ipv6Network>();
    }

    // Tests from cpython https://github.com/python/cpython/blob/e9bc4172d18db9c182d8e04dd7b033097a994c06/Lib/test/test_ipaddress.py
    #[test]
    fn test_is_subnet_of() {
        let mut test_cases: HashMap<(Ipv6Network, Ipv6Network), bool> = HashMap::new();

        test_cases.insert(
            (
                "2000:999::/56".parse().unwrap(),
                "2000:aaa::/48".parse().unwrap(),
            ),
            false,
        );
        test_cases.insert(
            (
                "2000:aaa::/56".parse().unwrap(),
                "2000:aaa::/48".parse().unwrap(),
            ),
            true,
        );
        test_cases.insert(
            (
                "2000:bbb::/56".parse().unwrap(),
                "2000:aaa::/48".parse().unwrap(),
            ),
            false,
        );
        test_cases.insert(
            (
                "2000:aaa::/48".parse().unwrap(),
                "2000:aaa::/56".parse().unwrap(),
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
        let mut test_cases: HashMap<(Ipv6Network, Ipv6Network), bool> = HashMap::new();

        test_cases.insert(
            (
                "2000:999::/56".parse().unwrap(),
                "2000:aaa::/48".parse().unwrap(),
            ),
            false,
        );
        test_cases.insert(
            (
                "2000:aaa::/56".parse().unwrap(),
                "2000:aaa::/48".parse().unwrap(),
            ),
            false,
        );
        test_cases.insert(
            (
                "2000:bbb::/56".parse().unwrap(),
                "2000:aaa::/48".parse().unwrap(),
            ),
            false,
        );
        test_cases.insert(
            (
                "2000:aaa::/48".parse().unwrap(),
                "2000:aaa::/56".parse().unwrap(),
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
        let other: Ipv6Network = "2001:DB8:ACAD::1/64".parse().unwrap();
        let other2: Ipv6Network = "2001:DB8:ACAD::20:2/64".parse().unwrap();

        assert_eq!(other2.overlaps(other), true);
    }

    #[test]
    fn edges() {
        let low: Ipv6Network = "::0/120".parse().unwrap();
        let low_addrs: Vec<Ipv6Addr> = low.iter().collect();
        assert_eq!(256, low_addrs.len());

        let high: Ipv6Network = "ffff:ffff:ffff:ffff:ffff:ffff:ffff:ff00/120"
            .parse()
            .unwrap();
        let high_addrs: Vec<Ipv6Addr> = high.iter().collect();
        assert_eq!(256, high_addrs.len());
    }
}
