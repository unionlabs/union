//! Formats `FixedVector<u64,N>` using quotes.
//!
//! E.g., `FixedVector::from(vec![0, 1, 2])` serializes as `["0", "1", "2"]`.
//!
//! Quotes can be optional during decoding. If `N` does not equal the length deserialization will fail.

// The (de)serialization functions for variable lists are now sufficiently general that we can
// implement fixed vector (de)serialization in terms of them.
pub use crate::serde_utils::quoted_u64_var_list::{deserialize, serialize};

#[cfg(test)]
mod test {
    use serde::{Deserialize, Serialize};
    use typenum::U4;

    use crate::FixedVector;

    #[derive(Debug, Serialize, Deserialize)]
    struct Obj {
        #[serde(with = "crate::serde_utils::quoted_u64_fixed_vec")]
        values: FixedVector<u64, U4>,
    }

    #[test]
    fn quoted_list_success() {
        let obj: Obj = serde_json::from_str(r#"{ "values": ["1", "2", "3", "4"] }"#).unwrap();
        let expected: FixedVector<u64, U4> = vec![1, 2, 3, 4].try_into().unwrap();
        assert_eq!(obj.values, expected);
    }

    #[test]
    fn unquoted_list_success() {
        let obj: Obj = serde_json::from_str(r#"{ "values": [1, 2, 3, 4] }"#).unwrap();
        let expected: FixedVector<u64, U4> = vec![1, 2, 3, 4].try_into().unwrap();
        assert_eq!(obj.values, expected);
    }

    #[test]
    fn mixed_list_success() {
        let obj: Obj = serde_json::from_str(r#"{ "values": ["1", 2, "3", "4"] }"#).unwrap();
        let expected: FixedVector<u64, U4> = vec![1, 2, 3, 4].try_into().unwrap();
        assert_eq!(obj.values, expected);
    }

    #[test]
    fn empty_list_err() {
        serde_json::from_str::<Obj>(r#"{ "values": [] }"#).unwrap_err();
    }

    #[test]
    fn short_list_err() {
        serde_json::from_str::<Obj>(r#"{ "values": [1, 2] }"#).unwrap_err();
    }

    #[test]
    fn long_list_err() {
        serde_json::from_str::<Obj>(r#"{ "values": [1, 2, 3, 4, 5] }"#).unwrap_err();
    }

    #[test]
    fn whole_list_quoted_err() {
        serde_json::from_str::<Obj>(r#"{ "values": "[1, 2, 3, 4]" }"#).unwrap_err();
    }
}
