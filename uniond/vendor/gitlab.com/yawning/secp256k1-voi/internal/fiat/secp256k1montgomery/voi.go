package secp256k1montgomery

import "gitlab.com/yawning/secp256k1-voi/internal/helpers"

// Uint64ToUint1 converts u to fiat's uint1, ensuring that the returned
// value is in the range [0,1].
func Uint64ToUint1(u uint64) uint1 {
	return uint1(helpers.Uint64IsNonzero(u))
}
