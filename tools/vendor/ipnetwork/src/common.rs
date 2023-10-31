use std::{error::Error, fmt};

/// Represents a bunch of errors that can occur while working with a `IpNetwork`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IpNetworkError {
    InvalidAddr(String),
    InvalidPrefix,
    InvalidCidrFormat(String),
}

impl fmt::Display for IpNetworkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use crate::IpNetworkError::*;
        match *self {
            InvalidAddr(ref s) => write!(f, "invalid address: {}", s),
            InvalidPrefix => write!(f, "invalid prefix"),
            InvalidCidrFormat(ref s) => write!(f, "invalid cidr format: {}", s),
        }
    }
}

impl Error for IpNetworkError {
    fn description(&self) -> &str {
        use crate::IpNetworkError::*;
        match *self {
            InvalidAddr(_) => "address is invalid",
            InvalidPrefix => "prefix is invalid",
            InvalidCidrFormat(_) => "cidr is invalid",
        }
    }
}

pub fn cidr_parts(cidr: &str) -> Result<(&str, Option<&str>), IpNetworkError> {
    // Try to find a single slash
    if let Some(sep) = cidr.find('/') {
        let (ip, prefix) = cidr.split_at(sep);
        // Error if cidr has multiple slashes
        if prefix[1..].find('/').is_some() {
            Err(IpNetworkError::InvalidCidrFormat(format!(
                "CIDR must contain a single '/': {}",
                cidr
            )))
        } else {
            // Handle the case when cidr has exactly one slash
            Ok((ip, Some(&prefix[1..])))
        }
    } else {
        // Handle the case when cidr does not have a slash
        Ok((cidr, None))
    }
}

pub fn parse_prefix(prefix: &str, max: u8) -> Result<u8, IpNetworkError> {
    let mask = prefix
        .parse::<u8>()
        .map_err(|_| IpNetworkError::InvalidPrefix)?;
    if mask > max {
        Err(IpNetworkError::InvalidPrefix)
    } else {
        Ok(mask)
    }
}
