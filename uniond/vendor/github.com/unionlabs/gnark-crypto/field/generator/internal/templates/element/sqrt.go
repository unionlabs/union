package element

const Sqrt = `

{{ if not .UseAddChain}}
var (
	_bLegendreExponent{{.ElementName}} *big.Int
	_bSqrtExponent{{.ElementName}} *big.Int
)

func init() {
	_bLegendreExponent{{.ElementName}}, _ = new(big.Int).SetString("{{.LegendreExponent}}", 16)
	{{- if .SqrtQ3Mod4}}
		const sqrtExponent{{.ElementName}} = "{{.SqrtQ3Mod4Exponent}}"
	{{- else if .SqrtAtkin}}
		const sqrtExponent{{.ElementName}} = "{{.SqrtAtkinExponent}}"
	{{- else if .SqrtTonelliShanks}}
		const sqrtExponent{{.ElementName}} = "{{.SqrtSMinusOneOver2}}"
	{{- end }}
	_bSqrtExponent{{.ElementName}}, _ = new(big.Int).SetString(sqrtExponent{{.ElementName}}, 16)
}

{{- end }}

// Legendre returns the Legendre symbol of z (either +1, -1, or 0.)
func (z *{{.ElementName}}) Legendre() int {
	var l {{.ElementName}}
	// z^((q-1)/2)
	{{- if .UseAddChain}}
	l.expByLegendreExp(*z)
	{{- else}}
	l.Exp(*z, _bLegendreExponent{{.ElementName}})
	{{- end}}
	
	if l.IsZero() {
		return 0
	} 

	// if l == 1
	if l.IsOne()  {
		return 1
	}
	return -1
}


// Sqrt z = √x (mod q)
// if the square root doesn't exist (x is not a square mod q)
// Sqrt leaves z unchanged and returns nil
func (z *{{.ElementName}}) Sqrt(x *{{.ElementName}}) *{{.ElementName}} {
	{{- if .SqrtQ3Mod4}}
		// q ≡ 3 (mod 4)
		// using  z ≡ ± x^((p+1)/4) (mod q)
		var y, square {{.ElementName}}
		{{- if .UseAddChain}}
		y.expBySqrtExp(*x)
		{{- else}}
		y.Exp(*x, _bSqrtExponent{{.ElementName}})
		{{- end }}
		// as we didn't compute the legendre symbol, ensure we found y such that y * y = x
		square.Square(&y)
		if square.Equal(x) {
			return z.Set(&y)
		} 
		return nil
	{{- else if .SqrtAtkin}}
		// q ≡ 5 (mod 8)
		// see modSqrt5Mod8Prime in math/big/int.go
		var one, alpha, beta, tx, square {{.ElementName}}
		one.SetOne()
		tx.Double(x)
		{{- if .UseAddChain}}
		alpha.expBySqrtExp(tx)
		{{ else }}
		alpha.Exp(tx, _bSqrtExponent{{.ElementName}})
		{{- end }}
		beta.Square(&alpha).
			Mul(&beta, &tx).
			Sub(&beta, &one).
			Mul(&beta, x).
			Mul(&beta, &alpha)
		
		// as we didn't compute the legendre symbol, ensure we found beta such that beta * beta = x
		square.Square(&beta)
		if square.Equal(x) {
			return z.Set(&beta)
		}
		return nil
	{{- else if .SqrtTonelliShanks}}
		// q ≡ 1 (mod 4)
		// see modSqrtTonelliShanks in math/big/int.go
		// using https://www.maa.org/sites/default/files/pdf/upload_library/22/Polya/07468342.di020786.02p0470a.pdf

		var y, b,t, w  {{.ElementName}}
		// w = x^((s-1)/2))
		{{- if .UseAddChain}}
		w.expBySqrtExp(*x)
		{{- else}}
		w.Exp(*x, _bSqrtExponent{{.ElementName}})
		{{- end}}

		// y = x^((s+1)/2)) = w * x
		y.Mul(x, &w)

		// b = xˢ = w * w * x = y * x
		b.Mul(&w, &y)

		// g = nonResidue ^ s
		var g = {{.ElementName}}{
			{{- range $i := .SqrtG}}
			{{$i}},{{end}}
		}
		r := uint64({{.SqrtE}})

		// compute legendre symbol
		// t = x^((q-1)/2) = r-1 squaring of xˢ
		t = b
		for i:=uint64(0); i < r-1; i++ {
			t.Square(&t)
		}
		if t.IsZero() {
			return z.SetZero()
		}
		if !t.IsOne() {
			// t != 1, we don't have a square root
			return nil
		}
		for {
			var m uint64
			t = b 

			// for t != 1
			for !t.IsOne() {
				t.Square(&t)
				m++
			}

			if m == 0 {
				return z.Set(&y)
			}
			// t = g^(2^(r-m-1)) (mod q)
			ge := int(r - m - 1)
			t = g
			for ge > 0 {
				t.Square(&t)
				ge--
			}

			g.Square(&t)
			y.Mul(&y, &t)
			b.Mul(&b, &g)
			r = m
		}

	{{- else}}
		panic("not implemented")	
	{{- end}}
}



`
