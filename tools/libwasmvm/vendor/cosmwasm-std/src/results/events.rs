use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::forward_ref_partial_eq;

/// A full [*Cosmos SDK* event].
///
/// This version uses string attributes (similar to [*Cosmos SDK* StringEvent]),
/// which then get magically converted to bytes for Tendermint somewhere between
/// the Rust-Go interface, JSON deserialization and the `NewEvent` call in Cosmos SDK.
///
/// [*Cosmos SDK* event]: https://docs.cosmos.network/main/core/events.html
/// [*Cosmos SDK* StringEvent]: https://github.com/cosmos/cosmos-sdk/blob/v0.42.5/proto/cosmos/base/abci/v1beta1/abci.proto#L56-L70
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[non_exhaustive]
pub struct Event {
    /// The event type. This is renamed to "ty" because "type" is reserved in Rust. This sucks, we know.
    #[serde(rename = "type")]
    pub ty: String,
    /// The attributes to be included in the event.
    ///
    /// You can learn more about these from [*Cosmos SDK* docs].
    ///
    /// [*Cosmos SDK* docs]: https://docs.cosmos.network/main/core/events.html
    pub attributes: Vec<Attribute>,
}

forward_ref_partial_eq!(Event, Event);

impl Event {
    /// Create a new event with the given type and an empty list of attributes.
    pub fn new(ty: impl Into<String>) -> Self {
        Event {
            ty: ty.into(),
            attributes: Vec::with_capacity(10),
        }
    }

    /// Add an attribute to the event.
    pub fn add_attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.push(Attribute {
            key: key.into(),
            value: value.into(),
        });
        self
    }

    /// Bulk add attributes to the event.
    ///
    /// Anything that can be turned into an iterator and yields something
    /// that can be converted into an `Attribute` is accepted.
    pub fn add_attributes<A: Into<Attribute>>(
        mut self,
        attrs: impl IntoIterator<Item = A>,
    ) -> Self {
        self.attributes.extend(attrs.into_iter().map(A::into));
        self
    }
}

/// An key value pair that is used in the context of event attributes in logs
#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq, Eq, JsonSchema)]
pub struct Attribute {
    pub key: String,
    pub value: String,
}

forward_ref_partial_eq!(Attribute, Attribute);

impl Attribute {
    /// Creates a new Attribute. `attr` is just an alias for this.
    pub fn new(key: impl Into<String>, value: impl Into<String>) -> Self {
        let key = key.into();

        #[cfg(debug_assertions)]
        if key.starts_with('_') {
            panic!(
                "attribute key `{}` is invalid - keys starting with an underscore are reserved",
                key
            );
        }

        Self {
            key,
            value: value.into(),
        }
    }
}

impl<K: Into<String>, V: Into<String>> From<(K, V)> for Attribute {
    fn from((k, v): (K, V)) -> Self {
        Attribute::new(k, v)
    }
}

impl<K: AsRef<str>, V: AsRef<str>> PartialEq<(K, V)> for Attribute {
    fn eq(&self, (k, v): &(K, V)) -> bool {
        (self.key.as_str(), self.value.as_str()) == (k.as_ref(), v.as_ref())
    }
}

impl<K: AsRef<str>, V: AsRef<str>> PartialEq<Attribute> for (K, V) {
    fn eq(&self, attr: &Attribute) -> bool {
        attr == self
    }
}

impl<K: AsRef<str>, V: AsRef<str>> PartialEq<(K, V)> for &Attribute {
    fn eq(&self, (k, v): &(K, V)) -> bool {
        (self.key.as_str(), self.value.as_str()) == (k.as_ref(), v.as_ref())
    }
}

impl<K: AsRef<str>, V: AsRef<str>> PartialEq<&Attribute> for (K, V) {
    fn eq(&self, attr: &&Attribute) -> bool {
        attr == self
    }
}

/// Creates a new Attribute. `Attribute::new` is an alias for this.
#[inline]
pub fn attr(key: impl Into<String>, value: impl Into<String>) -> Attribute {
    Attribute::new(key, value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Uint128;

    #[test]
    fn event_construction() {
        let event_direct = Event {
            ty: "test".to_string(),
            attributes: vec![attr("foo", "bar"), attr("bar", "baz")],
        };
        let event_builder = Event::new("test").add_attributes(vec![("foo", "bar"), ("bar", "baz")]);

        assert_eq!(event_direct, event_builder);
    }

    #[test]
    #[should_panic]
    fn attribute_new_reserved_key_panicks() {
        Attribute::new("_invalid", "value");
    }

    #[test]
    #[should_panic]
    fn attribute_new_reserved_key_panicks2() {
        Attribute::new("_", "value");
    }

    #[test]
    fn attr_works_for_different_types() {
        let expected = ("foo", "42");

        assert_eq!(attr("foo", "42"), expected);
        assert_eq!(attr("foo", "42"), expected);
        assert_eq!(attr("foo", "42"), expected);
        assert_eq!(attr("foo", Uint128::new(42)), expected);
    }
}
