use serde::Serialize;

use crate::errors::UnknownEnumVariant;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize)]
pub enum Order {
    Unspecified,
    Unordered,
    Ordered,
}

impl Order {
    pub fn try_from_str<T: AsRef<str>>(s: T) -> Result<Self, UnknownEnumVariant<T>> {
        match s.as_ref() {
            "ORDER_UNSPECIFIED" => Ok(Self::Unspecified),
            "ORDER_UNORDERED" => Ok(Self::Unordered),
            "ORDER_ORDERED" => Ok(Self::Ordered),
            _ => Err(UnknownEnumVariant(s)),
        }
    }
}

impl From<Order> for &'static str {
    fn from(value: Order) -> Self {
        match value {
            Order::Unspecified => "ORDER_UNSPECIFIED",
            Order::Unordered => "ORDER_UNORDERED",
            Order::Ordered => "ORDER_ORDERED",
        }
    }
}

impl TryFrom<u8> for Order {
    type Error = UnknownEnumVariant<u8>;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Order::Unspecified),
            1 => Ok(Order::Unordered),
            2 => Ok(Order::Ordered),
            _ => Err(UnknownEnumVariant(value)),
        }
    }
}

impl TryFrom<i32> for Order {
    type Error = UnknownEnumVariant<i32>;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Order::Unspecified),
            1 => Ok(Order::Unordered),
            2 => Ok(Order::Ordered),
            _ => Err(UnknownEnumVariant(value)),
        }
    }
}
