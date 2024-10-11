package element

// MulCIOS text book CIOS works for all modulus.
//
// There are couple of variations to the multiplication (and squaring) algorithms.
//
// All versions are derived from the Montgomery CIOS algorithm: see
// section 2.3.2 of Tolga Acar's thesis
// https://www.microsoft.com/en-us/research/wp-content/uploads/1998/06/97Acar.pdf
//
// For 1-word modulus, the generator will call mul_cios_one_limb (standard REDC)
//
// For 13-word+ modulus, the generator will output a unoptimized textbook CIOS code, in plain Go.
//
// For all other moduli, we look at the available bits in the last limb.
// If they are none (like secp256k1) we generate a unoptimized textbook CIOS code, in plain Go, for all architectures.
// If there is at least one we can omit a carry propagation in the CIOS algorithm.
// If there is at least two we can use the same technique for the CIOS Squaring.
// See appendix in https://eprint.iacr.org/2022/1400.pdf for the exact condition.
//
// In practice, we have 3 different targets in mind: x86(amd64), arm64 and wasm.
//
// For amd64, we can leverage (when available) the BMI2 and ADX instructions to have 2-carry-chains in parallel.
// This make the use of assembly worth it as it results in a significant perf improvement; most CPUs since 2016 support these
// instructions, and we assume it to be the "default path"; in case the CPU has no support, we fall back to a slow, unoptimized version.
//
// On amd64, the Squaring algorithm always call the Multiplication (assembly) implementation.
//
// For arm64, we unroll the loops in the CIOS (+nocarry optimization) algorithm, such that the instructions generated
// by the Go compiler closely match what we would hand-write. Hence, there is no assembly needed for arm64 target.
//
// Additionally, if 2-bits+ are available on the last limb, we have a template to generate a dedicated Squaring algorithm
// This is not activated by default, to minimize the codebase size.
// On M1, AWS Graviton3 it results in a 5-10% speedup. On some mobile devices, speed up observed was more important (~20%).
//
// The same (arm64) unrolled Go code produce satisfying performance for WASM (compiled using TinyGo).
const MulCIOS = `
{{ define "mul_cios" }}
	var t [{{add .all.NbWords 1}}]uint64
	var D uint64
	var m, C uint64

	{{- range $j := .all.NbWordsIndexesFull}}
		// -----------------------------------
		// First loop
		{{ if eq $j 0}}
			C, t[0] = bits.Mul64({{$.V2}}[{{$j}}], {{$.V1}}[0])
			{{- range $i := $.all.NbWordsIndexesNoZero}}
				C, t[{{$i}}] = madd1({{$.V2}}[{{$j}}], {{$.V1}}[{{$i}}], C)
			{{- end}}
		{{ else }}
			C, t[0] = madd1({{$.V2}}[{{$j}}], {{$.V1}}[0], t[0])
			{{- range $i := $.all.NbWordsIndexesNoZero}}
				C, t[{{$i}}] = madd2({{$.V2}}[{{$j}}], {{$.V1}}[{{$i}}], t[{{$i}}], C)
			{{- end}}
		{{ end }}
		t[{{$.all.NbWords}}], D = bits.Add64(t[{{$.all.NbWords}}], C, 0)

		// m = t[0]n'[0] mod W
		m = t[0] * qInvNeg

		// -----------------------------------
		// Second loop
		C = madd0(m, q0, t[0])
		{{- range $i := $.all.NbWordsIndexesNoZero}}
				C, t[{{sub $i 1}}] = madd2(m, q{{$i}}, t[{{$i}}], C)
		{{- end}}

		 t[{{sub $.all.NbWords 1}}], C = bits.Add64(t[{{$.all.NbWords}}], C, 0)
		 t[{{$.all.NbWords}}], _ = bits.Add64(0, D, C)
	{{- end}}


	if t[{{$.all.NbWords}}] != 0 {
		// we need to reduce, we have a result on {{add 1 $.all.NbWords}} words
		{{- if gt $.all.NbWords 1}}
		var b uint64
		{{- end}}
		z[0], {{- if gt $.all.NbWords 1}}b{{- else}}_{{- end}} = bits.Sub64(t[0], q0, 0)
		{{- range $i := .all.NbWordsIndexesNoZero}}
			{{-  if eq $i $.all.NbWordsLastIndex}}
				z[{{$i}}], _ = bits.Sub64(t[{{$i}}], q{{$i}}, b)
			{{-  else  }}
				z[{{$i}}], b = bits.Sub64(t[{{$i}}], q{{$i}}, b)
			{{- end}}
		{{- end}}
		return {{if $.ReturnZ }} z{{- end}}
	}

	// copy t into z 
	{{- range $i := $.all.NbWordsIndexesFull}}
		z[{{$i}}] = t[{{$i}}]
	{{- end}}

{{ end }}

{{ define "mul_cios_one_limb" }}
	// In fact, since the modulus R fits on one register, the CIOS algorithm gets reduced to standard REDC (textbook Montgomery reduction):
	// hi, lo := x * y
	// m := (lo * qInvNeg) mod R
	// (*) r := (hi * R + lo + m * q) / R
	// reduce r if necessary

	// On the emphasized line, we get r = hi + (lo + m * q) / R
	// If we write hi2, lo2 = m * q then R | m * q - lo2 ⇒ R | (lo * qInvNeg) q - lo2 = -lo - lo2
	// This shows lo + lo2 = 0 mod R. i.e. lo + lo2 = 0 if lo = 0 and R otherwise.
	// Which finally gives (lo + m * q) / R = (lo + lo2 + R hi2) / R = hi2 + (lo+lo2) / R = hi2 + (lo != 0)
	// This "optimization" lets us do away with one MUL instruction on ARM architectures and is available for all q < R.

	var r uint64
	hi, lo := bits.Mul64({{$.V1}}[0], {{$.V2}}[0])
	if lo != 0 {
		hi++ // x[0] * y[0] ≤ 2¹²⁸ - 2⁶⁵ + 1, meaning hi ≤ 2⁶⁴ - 2 so no need to worry about overflow
	}
	m := lo * qInvNeg
	hi2, _ := bits.Mul64(m, q)
	r, carry := bits.Add64(hi2, hi, 0)

	if carry != 0 || r >= q  {
		// we need to reduce
		r -= q 
	}
	z[0] = r 
{{ end }}
`

const MulDoc = `
{{define "mul_doc noCarry"}}
// Implements CIOS multiplication -- section 2.3.2 of Tolga Acar's thesis
// https://www.microsoft.com/en-us/research/wp-content/uploads/1998/06/97Acar.pdf
// 
// The algorithm:
// 
// for i=0 to N-1
// 		C := 0
// 		for j=0 to N-1
// 			(C,t[j]) := t[j] + x[j]*y[i] + C
// 		(t[N+1],t[N]) := t[N] + C
//		
// 		C := 0
// 		m := t[0]*q'[0] mod D
// 		(C,_) := t[0] + m*q[0]
// 		for j=1 to N-1
// 			(C,t[j-1]) := t[j] + m*q[j] + C
//		
// 		(C,t[N-1]) := t[N] + C
// 		t[N] := t[N+1] + C
//
// → N is the number of machine words needed to store the modulus q
// → D is the word size. For example, on a 64-bit architecture D is 2	64
// → x[i], y[i], q[i] is the ith word of the numbers x,y,q
// → q'[0] is the lowest word of the number -q⁻¹ mod r. This quantity is pre-computed, as it does not depend on the inputs.
// → t is a temporary array of size N+2 
// → C, S are machine words. A pair (C,S) refers to (hi-bits, lo-bits) of a two-word number
{{- if .noCarry}}
// 
// As described here https://hackmd.io/@gnark/modular_multiplication we can get rid of one carry chain and simplify:
// (also described in https://eprint.iacr.org/2022/1400.pdf annex)
//
// for i=0 to N-1
// 		(A,t[0]) := t[0] + x[0]*y[i] 
// 		m := t[0]*q'[0] mod W
// 		C,_ := t[0] + m*q[0]
// 		for j=1 to N-1
// 			(A,t[j])  := t[j] + x[j]*y[i] + A
// 			(C,t[j-1]) := t[j] + m*q[j] + C
//		
// 		t[N-1] = C + A
//
// This optimization saves 5N + 2 additions in the algorithm, and can be used whenever the highest bit 
// of the modulus is zero (and not all of the remaining bits are set).
{{- end}}
{{ end }}
`
