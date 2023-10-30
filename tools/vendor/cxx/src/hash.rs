use core::hash::{Hash, Hasher};

#[doc(hidden)]
pub fn hash<V: Hash>(value: &V) -> usize {
    #[cfg(feature = "std")]
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    #[cfg(not(feature = "std"))]
    let mut hasher = crate::sip::SipHasher13::new();

    Hash::hash(value, &mut hasher);
    Hasher::finish(&hasher) as usize
}
