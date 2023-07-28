package btcec

import secp "github.com/decred/dcrd/dcrec/secp256k1/v4"

// FieldVal implements optimized fixed-precision arithmetic over the secp256k1
// finite field. This means all arithmetic is performed modulo
// '0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2f'.
//
// WARNING: Since it is so important for the field arithmetic to be extremely
// fast for high performance crypto, this type does not perform any validation
// of documented preconditions where it ordinarily would. As a result, it is
// IMPERATIVE for callers to understand some key concepts that are described
// below and ensure the methods are called with the necessary preconditions
// that each method is documented with. For example, some methods only give the
// correct result if the field value is normalized and others require the field
// values involved to have a maximum magnitude and THERE ARE NO EXPLICIT CHECKS
// TO ENSURE THOSE PRECONDITIONS ARE SATISFIED. This does, unfortunately, make
// the type more difficult to use correctly and while I typically prefer to
// ensure all state and input is valid for most code, this is a bit of an
// exception because those extra checks really add up in what ends up being
// critical hot paths.
//
// The first key concept when working with this type is normalization. In order
// to avoid the need to propagate a ton of carries, the internal representation
// provides additional overflow bits for each word of the overall 256-bit
// value.  This means that there are multiple internal representations for the
// same value and, as a result, any methods that rely on comparison of the
// value, such as equality and oddness determination, require the caller to
// provide a normalized value.
//
// The second key concept when working with this type is magnitude. As
// previously mentioned, the internal representation provides additional
// overflow bits which means that the more math operations that are performed
// on the field value between normalizations, the more those overflow bits
// accumulate. The magnitude is effectively that maximum possible number of
// those overflow bits that could possibly be required as a result of a given
// operation. Since there are only a limited number of overflow bits available,
// this implies that the max possible magnitude MUST be tracked by the caller
// and the caller MUST normalize the field value if a given operation would
// cause the magnitude of the result to exceed the max allowed value.
//
// IMPORTANT: The max allowed magnitude of a field value is 64.
type FieldVal = secp.FieldVal
