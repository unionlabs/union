use crate::tree::Node;

use std::fmt;

/// Represents errors that can occur when inserting a new route.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum InsertError {
    /// Attempted to insert a path that conflicts with an existing route.
    Conflict {
        /// The existing route that the insertion is conflicting with.
        with: String,
    },
    /// Only one parameter per route segment is allowed.
    TooManyParams,
    /// Parameters must be registered with a name.
    UnnamedParam,
    /// Catch-all parameters are only allowed at the end of a path.
    InvalidCatchAll,
    /// Invalid tokens in the inserted path.
    MalformedPath,
}

impl fmt::Display for InsertError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Conflict { with } => {
                write!(
                    f,
                    "insertion failed due to conflict with previously registered route: {}",
                    with
                )
            }
            Self::TooManyParams => write!(f, "only one parameter is allowed per path segment"),
            Self::UnnamedParam => write!(f, "parameters must be registered with a name"),
            Self::InvalidCatchAll => write!(
                f,
                "catch-all parameters are only allowed at the end of a route"
            ),
            Self::MalformedPath => write!(f, "malformed path"),
        }
    }
}

impl std::error::Error for InsertError {}

impl InsertError {
    pub(crate) fn conflict<T>(route: &[u8], prefix: &[u8], current: &Node<T>) -> Self {
        let mut route = route[..route.len() - prefix.len()].to_owned();

        if !route.ends_with(&current.prefix) {
            route.extend_from_slice(&current.prefix);
        }

        let mut current = current.children.first();
        while let Some(node) = current {
            route.extend_from_slice(&node.prefix);
            current = node.children.first();
        }

        InsertError::Conflict {
            with: String::from_utf8(route).unwrap(),
        }
    }
}

/// A failed match attempt.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum MatchError {
    /// The path was missing a trailing slash.
    MissingTrailingSlash,
    /// The path had an extra trailing slash.
    ExtraTrailingSlash,
    /// No matching route was found.
    NotFound,
}

impl MatchError {
    pub(crate) fn unsure(full_path: &[u8]) -> Self {
        if full_path[full_path.len() - 1] == b'/' {
            MatchError::ExtraTrailingSlash
        } else {
            MatchError::MissingTrailingSlash
        }
    }
}

impl fmt::Display for MatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match self {
            MatchError::MissingTrailingSlash => "match error: expected trailing slash",
            MatchError::ExtraTrailingSlash => "match error: found extra trailing slash",
            MatchError::NotFound => "match error: route not found",
        };

        write!(f, "{}", msg)
    }
}

impl std::error::Error for MatchError {}
