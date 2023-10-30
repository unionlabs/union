use crate::Uuid;

impl Uuid {
    /// Creates a random UUID.
    ///
    /// This uses the [`getrandom`] crate to utilise the operating system's RNG
    /// as the source of random numbers. If you'd like to use a custom
    /// generator, don't use this method: generate random bytes using your
    /// custom generator and pass them to the
    /// [`uuid::Builder::from_random_bytes`][from_random_bytes] function
    /// instead.
    ///
    /// Note that usage of this method requires the `v4` feature of this crate
    /// to be enabled.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use uuid::{Uuid, Version};
    /// let uuid = Uuid::new_v4();
    ///
    /// assert_eq!(Some(Version::Random), uuid.get_version());
    /// ```
    ///
    /// # References
    ///
    /// * [Version 4 UUIDs in RFC4122](https://www.rfc-editor.org/rfc/rfc4122#section-4.4)
    ///
    /// [`getrandom`]: https://crates.io/crates/getrandom
    /// [from_random_bytes]: struct.Builder.html#method.from_random_bytes
    pub fn new_v4() -> Uuid {
        crate::Builder::from_random_bytes(crate::rng::bytes()).into_uuid()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Variant, Version};

    #[cfg(all(
        target_arch = "wasm32",
        target_vendor = "unknown",
        target_os = "unknown"
    ))]
    use wasm_bindgen_test::*;

    #[test]
    #[cfg_attr(
        all(
            target_arch = "wasm32",
            target_vendor = "unknown",
            target_os = "unknown"
        ),
        wasm_bindgen_test
    )]
    fn test_new() {
        let uuid = Uuid::new_v4();

        assert_eq!(uuid.get_version(), Some(Version::Random));
        assert_eq!(uuid.get_variant(), Variant::RFC4122);
    }

    #[test]
    #[cfg_attr(
        all(
            target_arch = "wasm32",
            target_vendor = "unknown",
            target_os = "unknown"
        ),
        wasm_bindgen_test
    )]
    fn test_get_version() {
        let uuid = Uuid::new_v4();

        assert_eq!(uuid.get_version(), Some(Version::Random));
        assert_eq!(uuid.get_version_num(), 4)
    }
}
