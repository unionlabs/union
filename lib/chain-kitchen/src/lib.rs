#![warn(clippy::pedantic)]
#![no_std]

use alloc::vec::Vec;
use core::{
    fmt::{self, Display},
    ptr,
};

use ucs04::{Family, Id, UniversalChainId};

extern crate alloc;

#[derive(Debug)]
#[must_use]
pub struct Endpoint<'a> {
    id: &'a Id,
    family: Family,
    protocol: &'a Protocol,
    tags: Vec<&'a Tag>,
    // maybe validate this is a valid domain, probably we don't care
    // default to chain.kitchen
    domain: &'a str,
}

impl<'a> Endpoint<'a> {
    pub const DEFAULT_DOMAIN: &'static str = "chain.kitchen";

    pub const fn new(id: &'a Id, family: Family, protocol: &'a Protocol) -> Self {
        Self {
            id,
            family,
            protocol,
            tags: Vec::new(),
            domain: Self::DEFAULT_DOMAIN,
        }
    }

    pub fn from_ucs04(ucs04: &'a UniversalChainId<'a>, protocol: &'a Protocol) -> Self {
        Self {
            id: ucs04.id(),
            family: ucs04.family(),
            protocol,
            tags: Vec::new(),
            domain: Self::DEFAULT_DOMAIN,
        }
    }

    pub const fn with_domain(mut self, domain: &'a str) -> Self {
        self.domain = domain;
        self
    }

    pub fn with_tags(mut self, tags: impl IntoIterator<Item = &'a Tag>) -> Self {
        self.tags = tags.into_iter().collect();
        self
    }
}

impl fmt::Display for Endpoint<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            id,
            family,
            protocol,
            tags,
            domain,
        } = self;

        write!(f, "https://")?;
        // intersperse is still unstable :(
        for (idx, tag) in tags.iter().enumerate() {
            write!(f, "{tag}")?;
            // don't write - after the last tag
            if idx + 1 < tags.len() {
                write!(f, "-")?;
            } else {
                write!(f, ".")?;
            }
        }

        write!(f, "{protocol}.{id}.{family}.{domain}")?;

        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq)]
#[repr(transparent)]
pub struct Protocol(str);

impl Protocol {
    pub const RPC: &'static Self = Self::new("rpc").unwrap();
    pub const REST: &'static Self = Self::new("rest").unwrap();
    pub const BEACON: &'static Self = Self::new("beacon").unwrap();

    #[must_use]
    pub const fn new(s: &str) -> Option<&Self> {
        if s.is_empty() {
            return None;
        }

        let mut i = 0;
        let bz = s.as_bytes();

        while i < bz.len() {
            if !(bz[i].is_ascii_lowercase() || bz[i] == b'-') {
                return None;
            }

            i += 1;
        }

        Some(unsafe { &*(ptr::from_ref::<str>(s) as *const Self) })
    }
}

impl Display for Protocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, PartialEq, Eq)]
#[repr(transparent)]
pub struct Tag(str);

impl Tag {
    #[must_use]
    pub const fn new(s: &str) -> Option<&Self> {
        if s.is_empty() {
            return None;
        }

        let mut i = 0;
        let bz = s.as_bytes();

        while i < bz.len() {
            if !bz[i].is_ascii_lowercase() {
                return None;
            }

            i += 1;
        }

        Some(unsafe { &*(ptr::from_ref::<str>(s) as *const Self) })
    }
}

impl Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;

    use super::*;

    #[test]
    fn display() {
        assert_eq!(
            Endpoint::new(
                Id::new("1").unwrap(),
                Family::Ethereum,
                Protocol::new("rpc").unwrap(),
            )
            .to_string(),
            "https://rpc.1.ethereum.chain.kitchen"
        );

        assert_eq!(
            Endpoint::new(
                Id::new("1").unwrap(),
                Family::Ethereum,
                Protocol::new("rpc").unwrap(),
            )
            .with_tags([])
            .to_string(),
            "https://rpc.1.ethereum.chain.kitchen"
        );

        assert_eq!(
            Endpoint::new(
                Id::new("1").unwrap(),
                Family::Ethereum,
                Protocol::new("rpc").unwrap(),
            )
            .with_tags([Tag::new("tag").unwrap()])
            .to_string(),
            "https://tag.rpc.1.ethereum.chain.kitchen"
        );

        assert_eq!(
            Endpoint::new(
                Id::new("1").unwrap(),
                Family::Ethereum,
                Protocol::new("rpc").unwrap(),
            )
            .with_tags([Tag::new("tag").unwrap(), Tag::new("anothertag").unwrap()])
            .to_string(),
            "https://tag-anothertag.rpc.1.ethereum.chain.kitchen"
        );
    }

    #[test]
    fn tag() {
        assert_eq!(Tag::new(""), None);
        assert_eq!(Tag::new("-"), None);
        assert_eq!(Tag::new(" "), None);
        assert_eq!(Tag::new("tag-"), None);
        assert_eq!(Tag::new("-tag"), None);
        assert_eq!(Tag::new("-t"), None);
        assert_eq!(Tag::new("t-"), None);
        assert_eq!(Tag::new("_"), None);
        assert_eq!(Tag::new("1"), None);
        assert_eq!(Tag::new("9"), None);

        assert_eq!(&Tag::new("tag").unwrap().0, "tag");
        assert_eq!(&Tag::new("validtag").unwrap().0, "validtag");
        assert_eq!(&Tag::new("anothervalidtag").unwrap().0, "anothervalidtag");
    }

    #[test]
    fn protocol() {
        assert_eq!(Protocol::new(""), None);
        assert_eq!(Protocol::new(" "), None);
        assert_eq!(Protocol::new("_"), None);
        assert_eq!(Protocol::new("1"), None);
        assert_eq!(Protocol::new("9"), None);

        // notably invalid tags, but valid protocols
        assert_eq!(&Protocol::new("-").unwrap().0, "-");
        assert_eq!(&Protocol::new("tag-").unwrap().0, "tag-");
        assert_eq!(&Protocol::new("-tag").unwrap().0, "-tag");
        assert_eq!(&Protocol::new("-t").unwrap().0, "-t");
        assert_eq!(&Protocol::new("t-").unwrap().0, "t-");

        assert_eq!(&Protocol::new("protocol").unwrap().0, "protocol");
        assert_eq!(
            &Protocol::new("valid-protocol").unwrap().0,
            "valid-protocol"
        );
        assert_eq!(
            &Protocol::new("another-valid-protocol").unwrap().0,
            "another-valid-protocol"
        );
    }
}
