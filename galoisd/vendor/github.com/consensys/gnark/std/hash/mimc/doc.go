// Package mimc provides a ZKP-circuit function to compute a MiMC hash.
//
// For the reference implementation of the MiMC hash function, see the
// corresponding package in [gnark-crypto].
//
// # Length extension attack
//
// The MiMC hash function is vulnerable to a length extension attack. For
// example when we have a hash
//
//	h = MiMC(k || m)
//
// and we want to hash a new message
//
//	m' = m || m2,
//
// we can compute
//
//	h' = MiMC(k || m || m2)
//
// without knowing k by computing
//
//	h' = MiMC(h || m2).
//
// This is because the MiMC hash function is a simple iterated cipher, and the
// hash value is the state of the cipher after encrypting the message.
//
// There are several ways to mitigate this attack:
//   - use a random key for each hash
//   - use a domain separation tag for different use cases:
//     h = MiMC(k || tag || m)
//   - use the secret input as last input:
//     h = MiMC(m || k)
//
// In general, inside a circuit the length-extension attack is not a concern as
// due to the circuit definition the attacker can not append messages to
// existing hash. But the user has to consider the cases when using a secret key
// and MiMC in different contexts.
//
// [gnark-crypto]: https://pkg.go.dev/github.com/consensys/gnark-crypto/hash
package mimc
