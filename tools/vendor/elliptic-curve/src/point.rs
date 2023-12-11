//! Traits for elliptic curve points.

#[cfg(feature = "arithmetic")]
mod non_identity;

#[cfg(feature = "arithmetic")]
pub use {self::non_identity::NonIdentity, crate::CurveArithmetic};

use crate::{Curve, FieldBytes};
use subtle::{Choice, CtOption};

/// Affine point type for a given curve with a [`CurveArithmetic`]
/// implementation.
#[cfg(feature = "arithmetic")]
pub type AffinePoint<C> = <C as CurveArithmetic>::AffinePoint;

/// Projective point type for a given curve with a [`CurveArithmetic`]
/// implementation.
#[cfg(feature = "arithmetic")]
pub type ProjectivePoint<C> = <C as CurveArithmetic>::ProjectivePoint;

/// Access to the affine coordinates of an elliptic curve point.
// TODO: use zkcrypto/group#30 coordinate API when available
pub trait AffineCoordinates {
    /// Field element representation.
    type FieldRepr: AsRef<[u8]>;

    /// Get the affine x-coordinate as a serialized field element.
    fn x(&self) -> Self::FieldRepr;

    /// Is the affine y-coordinate odd?
    fn y_is_odd(&self) -> Choice;
}

/// Double a point (i.e. add it to itself)
pub trait Double {
    /// Double this point.
    fn double(&self) -> Self;
}

/// Decompress an elliptic curve point.
///
/// Point decompression recovers an original curve point from its x-coordinate
/// and a boolean flag indicating whether or not the y-coordinate is odd.
pub trait DecompressPoint<C: Curve>: Sized {
    /// Attempt to decompress an elliptic curve point.
    fn decompress(x: &FieldBytes<C>, y_is_odd: Choice) -> CtOption<Self>;
}

/// Decompact an elliptic curve point from an x-coordinate.
///
/// Decompaction relies on properties of specially-generated keys but provides
/// a more compact representation than standard point compression.
pub trait DecompactPoint<C: Curve>: Sized {
    /// Attempt to decompact an elliptic curve point
    fn decompact(x: &FieldBytes<C>) -> CtOption<Self>;
}

/// Point compression settings.
pub trait PointCompression {
    /// Should point compression be applied by default?
    const COMPRESS_POINTS: bool;
}

/// Point compaction settings.
pub trait PointCompaction {
    /// Should point compaction be applied by default?
    const COMPACT_POINTS: bool;
}
