#[cfg(feature = "v4")]
pub(crate) fn bytes() -> [u8; 16] {
    #[cfg(not(feature = "fast-rng"))]
    {
        let mut bytes = [0u8; 16];

        private_getrandom::getrandom(&mut bytes).unwrap_or_else(|err| {
            // NB: getrandom::Error has no source; this is adequate display
            panic!("could not retrieve random bytes for uuid: {}", err)
        });

        bytes
    }

    #[cfg(feature = "fast-rng")]
    {
        private_rand::random()
    }
}

#[cfg(feature = "v1")]
pub(crate) fn u16() -> u16 {
    #[cfg(not(feature = "fast-rng"))]
    {
        let mut bytes = [0u8; 2];

        private_getrandom::getrandom(&mut bytes).unwrap_or_else(|err| {
            // NB: getrandom::Error has no source; this is adequate display
            panic!("could not retrieve random bytes for uuid: {}", err)
        });

        ((bytes[0] as u16) << 8) | (bytes[1] as u16)
    }

    #[cfg(feature = "fast-rng")]
    {
        private_rand::random()
    }
}
