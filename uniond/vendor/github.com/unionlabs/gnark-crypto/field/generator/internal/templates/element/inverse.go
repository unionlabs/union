package element

const Inverse = `

{{ define "addQ" }}
if b != 0 {
	// z[{{.NbWordsLastIndex}}] = -1
	// negative: add q
	const neg1 = 0xFFFFFFFFFFFFFFFF

	var carry uint64
	{{$lastIndex := sub .NbWords 1}}
	{{- range $i :=  iterate 0 $lastIndex}}
	z[{{$i}}], carry = bits.Add64(z[{{$i}}], q{{$i}}, {{- if eq $i 0}}0{{- else}}carry{{- end}})
	{{- end}}
	z[{{.NbWordsLastIndex}}], _ = bits.Add64(neg1, q{{$.NbWordsLastIndex}}, carry)
}
{{- end}}

{{/* We use big.Int for Inverse for these type of moduli */}}
{{if not $.UsingP20Inverse}}

{{- if eq .NbWords 1}}
// Inverse z = x⁻¹ (mod q) 
//
// if x == 0, sets and returns z = x 
func (z *{{.ElementName}}) Inverse( x *{{.ElementName}}) *{{.ElementName}} {
	// Algorithm 16 in "Efficient Software-Implementation of Finite Fields with Applications to Cryptography"
	const q uint64 = q0
	if x.IsZero() {
		z.SetZero()
		return z
	}

	var r,s,u,v uint64
	u = q
	s = {{index .RSquare 0}} // s = r²
	r = 0
	v = x[0]

	var carry, borrow uint64

	for  (u != 1) && (v != 1){
		for v&1 == 0 {
			v >>= 1
			if s&1 == 0 {
				s >>= 1
			} else {
				s, carry = bits.Add64(s, q, 0)
				s >>= 1
				if carry != 0 {
					s |= (1 << 63)
				}
			}
		} 
		for u&1 == 0 {
			u >>= 1
			if r&1 == 0 {
				r >>= 1
			} else {
				r, carry = bits.Add64(r, q, 0)
				r >>= 1
				if carry != 0 {
					r |= (1 << 63)
				}
			}
		} 
		if v >= u  {
			v -= u
			s, borrow = bits.Sub64(s, r, 0)
			if borrow == 1 {
				s += q
			}
		} else {
			u -= v
			r, borrow = bits.Sub64(r, s, 0)
			if borrow == 1 {
				r += q
			}
		}
	}

	if u == 1 {
		z[0] = r
	} else {
		z[0] = s
	}
	
	return z
}
{{- else}}
// Inverse z = x⁻¹ (mod q) 
//
// note: allocates a big.Int (math/big)
func (z *{{.ElementName}}) Inverse( x *{{.ElementName}}) *{{.ElementName}} {
	var _xNonMont big.Int
	x.BigInt(&_xNonMont)
	_xNonMont.ModInverse(&_xNonMont, Modulus())
	z.SetBigInt(&_xNonMont)
	return z
}
{{- end}}

{{ else }}

const (
	k = 32 // word size / 2
	signBitSelector = uint64(1) << 63
	approxLowBitsN = k - 1
	approxHighBitsN = k + 1
)

const (
{{- range $i := .NbWordsIndexesFull}}
inversionCorrectionFactorWord{{$i}} = {{index $.P20InversionCorrectiveFac $i}}
{{- end}}
invIterationsN = {{.P20InversionNbIterations}}
)

// Inverse z = x⁻¹ (mod q)
//
// if x == 0, sets and returns z = x
func (z *{{.ElementName}}) Inverse(x *{{.ElementName}}) *{{.ElementName}} {
	// Implements "Optimized Binary GCD for Modular Inversion"
	// https://github.com/pornin/bingcd/blob/main/doc/bingcd.pdf

	a := *x
	b := {{.ElementName}} {
		{{- range $i := .NbWordsIndexesFull}}
		q{{$i}},{{end}}
	}	// b := q

	u := {{.ElementName}}{1}

	// Update factors: we get [u; v] ← [f₀ g₀; f₁ g₁] [u; v]
	// cᵢ = fᵢ + 2³¹ - 1 + 2³² * (gᵢ + 2³¹ - 1)
 	var c0, c1 int64

	// Saved update factors to reduce the number of field multiplications
 	var pf0, pf1, pg0, pg1 int64

	var i uint

	var v, s {{.ElementName}}

	// Since u,v are updated every other iteration, we must make sure we terminate after evenly many iterations
	// This also lets us get away with half as many updates to u,v
	// To make this constant-time-ish, replace the condition with i < invIterationsN
	for i = 0; i&1 == 1 || !a.IsZero(); i++ {
		n := max(a.BitLen(), b.BitLen())
		aApprox, bApprox := approximate(&a, n), approximate(&b, n)

		// f₀, g₀, f₁, g₁ = 1, 0, 0, 1
 		c0, c1 = updateFactorIdentityMatrixRow0, updateFactorIdentityMatrixRow1

		for j := 0; j < approxLowBitsN; j++ {

			// -2ʲ < f₀, f₁ ≤ 2ʲ
			// |f₀| + |f₁| < 2ʲ⁺¹

			if aApprox&1 == 0 {
				aApprox /= 2
			} else {
				s, borrow := bits.Sub64(aApprox, bApprox, 0)
				if borrow == 1 {
					s = bApprox - aApprox
					bApprox = aApprox
					c0, c1 = c1, c0
					// invariants unchanged
				}

				aApprox = s / 2
				c0 = c0 - c1

				// Now |f₀| < 2ʲ⁺¹ ≤ 2ʲ⁺¹ (only the weaker inequality is needed, strictly speaking)
                // Started with f₀ > -2ʲ and f₁ ≤ 2ʲ, so f₀ - f₁ > -2ʲ⁺¹
                // Invariants unchanged for f₁
			}

			c1 *= 2
			// -2ʲ⁺¹ < f₁ ≤ 2ʲ⁺¹
            // So now |f₀| + |f₁| < 2ʲ⁺²
		}

		s = a

		var g0 int64
		// from this point on c0 aliases for f0
		c0, g0 = updateFactorsDecompose(c0)
		aHi := a.linearCombNonModular(&s, c0, &b, g0)
		if aHi & signBitSelector != 0 {
			// if aHi < 0
			c0, g0 = -c0, -g0
			aHi = negL(&a, aHi)
		}
		// right-shift a by k-1 bits

		{{- range $i := .NbWordsIndexesFull}}
			{{-  if eq $i $.NbWordsLastIndex}}
				a[{{$i}}] = (a[{{$i}}] >> approxLowBitsN) | (aHi << approxHighBitsN)
			{{-  else  }}
				a[{{$i}}] = (a[{{$i}}] >> approxLowBitsN) | ((a[{{add $i 1}}]) << approxHighBitsN)
			{{- end}}
		{{- end}}

		var f1 int64
		// from this point on c1 aliases for g0
		f1, c1 = updateFactorsDecompose(c1)
		bHi := b.linearCombNonModular(&s, f1, &b, c1)
		if bHi & signBitSelector != 0 {
			// if bHi < 0
			f1, c1 = -f1, -c1
			bHi = negL(&b, bHi)
		}
		// right-shift b by k-1 bits

		{{- range $i := .NbWordsIndexesFull}}
			{{-  if eq $i $.NbWordsLastIndex}}
				b[{{$i}}] = (b[{{$i}}] >> approxLowBitsN) | (bHi << approxHighBitsN)
			{{-  else  }}
				b[{{$i}}] = (b[{{$i}}] >> approxLowBitsN) | ((b[{{add $i 1}}]) << approxHighBitsN)
			{{- end}}
		{{- end}}

		if i&1 == 1 {
			// Combine current update factors with previously stored ones
			// [F₀, G₀; F₁, G₁] ← [f₀, g₀; f₁, g₁] [pf₀, pg₀; pf₁, pg₁], with capital letters denoting new combined values
            // We get |F₀| = | f₀pf₀ + g₀pf₁ | ≤ |f₀pf₀| + |g₀pf₁| = |f₀| |pf₀| + |g₀| |pf₁| ≤ 2ᵏ⁻¹|pf₀| + 2ᵏ⁻¹|pf₁|
            // = 2ᵏ⁻¹ (|pf₀| + |pf₁|) < 2ᵏ⁻¹ 2ᵏ = 2²ᵏ⁻¹
            // So |F₀| < 2²ᵏ⁻¹ meaning it fits in a 2k-bit signed register

			// c₀ aliases f₀, c₁ aliases g₁
			c0, g0, f1, c1 = c0*pf0+g0*pf1,
				c0*pg0+g0*pg1,
				f1*pf0+c1*pf1,
				f1*pg0+c1*pg1

			s = u

			// 0 ≤ u, v < 2²⁵⁵
            // |F₀|, |G₀| < 2⁶³
            u.linearComb(&u, c0, &v, g0)
            // |F₁|, |G₁| < 2⁶³
            v.linearComb(&s, f1, &v, c1)

		} else {
			// Save update factors
			pf0, pg0, pf1, pg1 = c0, g0, f1, c1
		}
	}

	// For every iteration that we miss, v is not being multiplied by 2ᵏ⁻²
	const pSq uint64 = 1 << (2 * (k - 1))
	a = {{.ElementName}}{pSq}
	// If the function is constant-time ish, this loop will not run (no need to take it out explicitly)
	for ; i < invIterationsN; i += 2 {
		// could optimize further with mul by word routine or by pre-computing a table since with k=26,
		// we would multiply by pSq up to 13times;
		// on x86, the assembly routine outperforms generic code for mul by word
		// on arm64, we may loose up to ~5% for 6 limbs
		v.Mul(&v, &a)
	}

	u.Set(x) // for correctness check

	z.Mul(&v, &{{.ElementName}}{
		{{- range $i := .NbWordsIndexesFull }}
		inversionCorrectionFactorWord{{$i}},
		{{- end}}
	})

	// correctness check
    v.Mul(&u, z)
    if !v.IsOne() && !u.IsZero() {
            return z.inverseExp(u)
    }

	return z
}

// inverseExp computes z = x⁻¹ (mod q) = x**(q-2) (mod q) 
func (z *{{.ElementName}}) inverseExp(x {{.ElementName}}) *{{.ElementName}} {
	// e == q-2
	e := Modulus()
	e.Sub(e, big.NewInt(2))

	z.Set(&x)

	for i := e.BitLen() - 2; i >= 0; i-- {
		z.Square(z)
		if e.Bit(i) == 1 {
			z.Mul(z, &x)
		}
	}

	return z
}

// approximate a big number x into a single 64 bit word using its uppermost and lowermost bits
// if x fits in a word as is, no approximation necessary
func approximate(x *{{.ElementName}}, nBits int) uint64 {

	if nBits <= 64 {
		return x[0]
	}

	const mask = (uint64(1) << (k - 1)) - 1 // k-1 ones
	lo := mask & x[0]

	hiWordIndex := (nBits - 1) / 64

	hiWordBitsAvailable := nBits - hiWordIndex * 64
	hiWordBitsUsed := min(hiWordBitsAvailable, approxHighBitsN)

	mask_ := uint64(^((1 << (hiWordBitsAvailable - hiWordBitsUsed)) - 1))
	hi := (x[hiWordIndex] & mask_) << (64 - hiWordBitsAvailable)

	mask_ = ^(1<<(approxLowBitsN + hiWordBitsUsed) - 1)
	mid := (mask_ & x[hiWordIndex-1]) >> hiWordBitsUsed

	return lo | mid | hi
}

// linearComb z = xC * x + yC * y;
// 0 ≤ x, y < 2{{supScr .NbBits}}
// |xC|, |yC| < 2⁶³
func (z *{{.ElementName}}) linearComb(x *{{.ElementName}}, xC int64, y *{{.ElementName}}, yC int64) {
	{{- $elementCapacityNbBits := mul .NbWords 64}}
    // | (hi, z) | < 2 * 2⁶³ * 2{{supScr .NbBits}} = 2{{supScr (add 64 .NbBits)}}
	// therefore | hi | < 2{{supScr (sub (add 64 .NbBits) $elementCapacityNbBits)}} ≤ 2⁶³
	hi := z.linearCombNonModular(x, xC, y, yC)
	z.montReduceSigned(z, hi)
}

// montReduceSigned z = (xHi * r + x) * r⁻¹ using the SOS algorithm
// Requires |xHi| < 2⁶³. Most significant bit of xHi is the sign bit.
func (z *{{.ElementName}}) montReduceSigned(x *{{.ElementName}}, xHi uint64) {
	const signBitRemover = ^signBitSelector
	mustNeg := xHi & signBitSelector != 0
	// the SOS implementation requires that most significant bit is 0
	// Let X be xHi*r + x
	// If X is negative we would have initially stored it as 2⁶⁴ r + X (à la 2's complement)
	xHi &= signBitRemover
	// with this a negative X is now represented as 2⁶³ r + X

	var t [2*Limbs - 1]uint64
	var C uint64

	m := x[0] * qInvNeg

	C = madd0(m, q0, x[0])
	{{- range $i := .NbWordsIndexesNoZero}}
	C, t[{{$i}}] = madd2(m, q{{$i}}, x[{{$i}}], C)
	{{- end}}

	// m * qElement[{{.NbWordsLastIndex}}] ≤ (2⁶⁴ - 1) * (2⁶³ - 1) = 2¹²⁷ - 2⁶⁴ - 2⁶³ + 1
    // x[{{.NbWordsLastIndex}}] + C ≤ 2*(2⁶⁴ - 1) = 2⁶⁵ - 2
    // On LHS, (C, t[{{.NbWordsLastIndex}}]) ≤ 2¹²⁷ - 2⁶⁴ - 2⁶³ + 1 + 2⁶⁵ - 2 = 2¹²⁷ + 2⁶³ - 1
    // So on LHS, C ≤ 2⁶³
	t[{{.NbWords}}] = xHi + C
	// xHi + C < 2⁶³ + 2⁶³ = 2⁶⁴

	{{/* $NbWordsIndexesNoZeroInnerLoop := .NbWordsIndexesNoZero*/}}// <standard SOS>
	{{- range $i := iterate 1 $.NbWordsLastIndex}}
	{
		const i = {{$i}}
		m = t[i] * qInvNeg

		C = madd0(m, q0, t[i+0])

		{{- range $j := $.NbWordsIndexesNoZero}}
		C, t[i + {{$j}}] = madd2(m, q{{$j}}, t[i +  {{$j}}], C)
		{{- end}}

		t[i + Limbs] += C
	}
	{{- end}}
	{
		const i = {{.NbWordsLastIndex}}
		m := t[i] * qInvNeg

		C = madd0(m, q0, t[i+0])
		{{- range $j := iterate 1 $.NbWordsLastIndex}}
		C, z[{{sub $j 1}}] = madd2(m, q{{$j}}, t[i+{{$j}}], C)
		{{- end}}
		z[{{.NbWordsLastIndex}}], z[{{sub .NbWordsLastIndex 1}}] = madd2(m, q{{.NbWordsLastIndex}}, t[i+{{.NbWordsLastIndex}}], C)
	}
    {{ template "reduce" . }}
	// </standard SOS>

	if mustNeg {
		// We have computed ( 2⁶³ r + X ) r⁻¹ = 2⁶³ + X r⁻¹ instead
		var b uint64
		z[0], b = bits.Sub64(z[0], signBitSelector, 0)

		{{- range $i := .NbWordsIndexesNoZero}}
		z[{{$i}}], b = bits.Sub64(z[{{$i}}], 0, b)
		{{- end}}

		// Occurs iff x == 0 && xHi < 0, i.e. X = rX' for -2⁶³ ≤ X' < 0
		{{ template "addQ" .}}
	}
}

const (
	updateFactorsConversionBias int64 = 0x7fffffff7fffffff // (2³¹ - 1)(2³² + 1)
	updateFactorIdentityMatrixRow0 = 1
	updateFactorIdentityMatrixRow1 = 1 << 32
)

func updateFactorsDecompose(c int64) (int64, int64) {
	c += updateFactorsConversionBias
 	const low32BitsFilter int64 = 0xFFFFFFFF
 	f := c&low32BitsFilter - 0x7FFFFFFF
 	g := c>>32&low32BitsFilter - 0x7FFFFFFF
 	return f, g
}

{{ end }}

`
