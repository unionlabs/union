package prime

import "github.com/mmcloughlin/addchain/internal/polynomial"

// References:
//
//	[aranha]      Diego F. Aranha, Paulo S. L. M. Barreto, Geovandro C. C. F. Pereira and
//	              Jefferson E. Ricardini. A note on high-security general-purpose elliptic curves.
//	              Cryptology ePrint Archive, Report 2013/647. 2013.
//	              https://eprint.iacr.org/2013/647
//	[curve25519]  Bernstein, Daniel J. Curve25519: New Diffie-Hellman Speed Records. In Public Key
//	              Cryptography - PKC 2006, pages 207--228. 2006.
//	              https://cr.yp.to/ecdh/curve25519-20060209.pdf
//	[elligator]   Daniel J. Bernstein, Mike Hamburg, Anna Krasnova and Tanja Lange. Elligator:
//	              Elliptic-curve points indistinguishable from uniform random strings. Cryptology
//	              ePrint Archive, Report 2013/325. 2013. https://eprint.iacr.org/2013/325
//	[fips186-2]   NIST. Digital Signature Standard (DSS). Federal Information Processing Standards
//	              Publication 186-2. 2000.
//	              https://csrc.nist.gov/csrc/media/publications/fips/186/2/archive/2000-01-27/documents/fips186-2.pdf
//	[goldilocks]  Mike Hamburg. Ed448-Goldilocks, a new elliptic curve. Cryptology ePrint Archive,
//	              Report 2015/625. 2015. https://eprint.iacr.org/2015/625
//	[nistdanger]  Daniel J. Bernstein and Tanja Lange. Security dangers of the NIST curves. 2013.
//	              https://cr.yp.to/talks/2013.09.16/slides-djb-20130916-a4.pdf
//	[safecurves]  Daniel J. Bernstein and Tanja Lange. SafeCurves: choosing safe curves for
//	              elliptic-curve cryptography. https://safecurves.cr.yp.to
//	[sec2]        Certicom Research. SEC 2: Recommended Elliptic Curve Domain Parameters, Version
//	              2.0. Standards for Efficient Cryptography 2. 2010.
//	              https://safecurves.cr.yp.to/www.secg.org/sec2-v2.pdf

var (
	// P2213 is the prime 2²²¹ - 3 used in curve M-221 [aranha].
	P2213 = NewCrandall(221, 3)

	// P222117 is the prime 2²²² - 117 used in curve E-222 [aranha].
	P222117 = NewCrandall(222, 117)

	// P2519 is the prime 2²⁵¹ - 9 used in Curve1174 [elligator].
	P2519 = NewCrandall(251, 9)

	// P25519 is the prime 2²⁵⁵ - 19 used in Curve25519 [curve25519].
	P25519 = NewCrandall(255, 19)

	// P382105 is the prime 2³⁸² - 105 used in curve E-382 [aranha].
	P382105 = NewCrandall(382, 105)

	// P383187 is the prime 2³⁸³ - 187 used in curves M-383 and Curve383187 [aranha].
	P383187 = NewCrandall(383, 187)

	// P41417 is the prime 2⁴¹⁴ - 17 used in Curve41417 [nistdanger].
	P41417 = NewCrandall(414, 17)

	// P511187 is the prime 2⁵¹¹ - 187 used in M-511 [aranha].
	P511187 = NewCrandall(511, 187)

	// NISTP192 is the P-192 prime 2¹⁹² - 2⁶⁴ - 1 defined in [fips186-2].
	NISTP192 = NewSolinas(polynomial.Polynomial{{A: -1, N: 0}, {A: -1, N: 1}, {A: 1, N: 3}}, 64)

	// NISTP224 is the P-224 prime 2²²⁴ - 2⁹⁶ + 1 defined in [fips186-2].
	NISTP224 = NewSolinas(polynomial.Polynomial{{A: 1, N: 0}, {A: -1, N: 3}, {A: 1, N: 7}}, 32)

	// NISTP256 is the P-256 prime 2²⁵⁶ - 2²²⁴ + 2¹⁹² + 2⁹⁶ - 1 defined in [fips186-2].
	NISTP256 = NewSolinas(polynomial.Polynomial{{A: -1, N: 0}, {A: +1, N: 3}, {A: +1, N: 6}, {A: -1, N: 7}, {A: +1, N: 8}}, 32)

	// NISTP384 is the P-384 prime 2³⁸⁴ - 2¹²⁸ - 2⁹⁶ + 2³² - 1 defined in [fips186-2].
	NISTP384 = NewSolinas(polynomial.Polynomial{{A: -1, N: 0}, {A: 1, N: 1}, {A: -1, N: 3}, {A: -1, N: 4}, {A: 1, N: 12}}, 32)

	// Goldilocks is the prime 2⁴⁴⁸ - 2²²⁴ - 1 defined in [goldilocks].
	Goldilocks = NewSolinas(polynomial.Polynomial{{A: -1, N: 0}, {A: -1, N: 1}, {A: 1, N: 2}}, 224)

	// Secp192k1 is the prime for the 192-bit Koblitz curve recommended in [sec2].
	Secp192k1 = MustHex("FFFFFFFF_FFFFFFFF_FFFFFFFF_FFFFFFFF_FFFFFFFE_FFFFEE37")

	// Secp192r1 is the prime for the 192-bit "random" curve recommended in [sec2].
	Secp192r1 = NISTP192

	// Secp224k1 is the prime for the 224-bit Koblitz curve recommended in [sec2].
	Secp224k1 = MustHex("FFFFFFFF_FFFFFFFF_FFFFFFFF_FFFFFFFF_FFFFFFFF_FFFFFFFE_FFFFE56D")

	// Secp224r1 is the prime for the 224-bit "random" curve recommended in [sec2].
	Secp224r1 = NISTP224

	// Secp256k1 is the prime for the 256-bit Koblitz curve recommended in [sec2].
	Secp256k1 = MustHex("FFFFFFFF_FFFFFFFF_FFFFFFFF_FFFFFFFF_FFFFFFFF_FFFFFFFF_FFFFFFFE_FFFFFC2F")

	// Secp256r1 is the prime for the 256-bit "random" curve recommended in [sec2].
	Secp256r1 = NISTP256

	// Secp384r1 is the prime for the 384-bit "random" curve recommended in [sec2].
	Secp384r1 = NISTP384
)

// Distinguished is a list of well-known primes.
var Distinguished = []Prime{
	P2213,
	P222117,
	P2519,
	P25519,
	P382105,
	P383187,
	P41417,
	P511187,
	NISTP192,
	NISTP224,
	NISTP256,
	NISTP384,
	Goldilocks,
	Secp192k1,
	Secp224k1,
	Secp256k1,
}
