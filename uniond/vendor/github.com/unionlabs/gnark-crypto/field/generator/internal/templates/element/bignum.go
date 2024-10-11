package element

const BigNum = `

{{/* Only used for the Pornin Extended GCD Inverse Algorithm*/}}
{{if $.UsingP20Inverse}}

// negL negates in place [x | xHi] and return the new most significant word xHi
func negL(x *{{.ElementName}}, xHi uint64) uint64 {
	var b uint64

	x[0], b = bits.Sub64(0, x[0], 0)
	{{- range $i := .NbWordsIndexesNoZero}}
	x[{{$i}}], b = bits.Sub64(0, x[{{$i}}], b)
	{{- end}}
	xHi, _ = bits.Sub64(0, xHi, b)

	return xHi
}

// mulWNonModular multiplies by one word in non-montgomery, without reducing
func (z *{{.ElementName}}) mulWNonModular(x *{{.ElementName}}, y int64) uint64 {

	// w := abs(y)
	m := y >> 63
	w := uint64((y^m)-m)

	var c uint64
	c, z[0] = bits.Mul64(x[0], w)
	{{- range $i := .NbWordsIndexesNoZero }}
	c, z[{{$i}}] = madd1(x[{{$i}}], w, c)
	{{- end}}

	if y < 0 {
		c = negL(z, c)
	}

	return c
}

// linearCombNonModular computes a linear combination without modular reduction
func (z *{{.ElementName}}) linearCombNonModular(x *{{.ElementName}}, xC int64, y *{{.ElementName}}, yC int64) uint64 {
	var yTimes {{.ElementName}}

	yHi := yTimes.mulWNonModular(y, yC)
	xHi := z.mulWNonModular(x, xC)

	var carry uint64

	{{- range $i := .NbWordsIndexesFull}}
		z[{{$i}}], carry = bits.Add64(z[{{$i}}], yTimes[{{$i}}], {{- if eq $i 0}}0{{- else}}carry{{- end}})
	{{- end}}

	yHi, _ = bits.Add64(xHi, yHi, carry)

	return yHi
}

{{- end}}
`
