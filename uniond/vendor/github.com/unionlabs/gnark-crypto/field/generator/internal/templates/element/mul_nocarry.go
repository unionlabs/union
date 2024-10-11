package element

// MulNoCarry see https://eprint.iacr.org/2022/1400.pdf annex for more info on the algorithm
// Note that these templates are optimized for arm64 target, since x86 benefits from assembly impl.
const MulNoCarry = `
{{ define "mul_nocarry" }}
var {{range $i := .all.NbWordsIndexesFull}}t{{$i}}{{- if ne $i $.all.NbWordsLastIndex}},{{- end}}{{- end}} uint64
var {{range $i := .all.NbWordsIndexesFull}}u{{$i}}{{- if ne $i $.all.NbWordsLastIndex}},{{- end}}{{- end}} uint64
{{- range $i := .all.NbWordsIndexesFull}}
{
	var c0, c1, c2 uint64
	v := {{$.V1}}[{{$i}}]
	{{- if eq $i 0}}
		{{- range $j := $.all.NbWordsIndexesFull}}
			u{{$j}}, t{{$j}} = bits.Mul64(v, {{$.V2}}[{{$j}}])
		{{- end}}
	{{- else}}
		{{- range $j := $.all.NbWordsIndexesFull}}
			u{{$j}}, c1 = bits.Mul64(v, {{$.V2}}[{{$j}}])
			{{- if eq $j 0}}
				t{{$j}}, c0 = bits.Add64(c1, t{{$j}}, 0)
			{{- else }}
				t{{$j}}, c0 = bits.Add64(c1, t{{$j}}, c0)
			{{- end}}
			{{- if eq $j $.all.NbWordsLastIndex}}
				{{/* yes, we're tempted to write c2 = c0, but that slow the whole MUL by 20% */}}
				c2, _ = bits.Add64(0, 0, c0)
			{{- end}}
		{{- end}}
	{{- end}}

	{{- range $j := $.all.NbWordsIndexesFull}}
	{{- if eq $j 0}}
		t{{add $j 1}}, c0 = bits.Add64(u{{$j}}, t{{add $j 1}}, 0)
	{{- else if eq $j $.all.NbWordsLastIndex}}
		{{- if eq $i 0}}
			c2, _ = bits.Add64(u{{$j}}, 0, c0)
		{{- else}}
			c2, _ = bits.Add64(u{{$j}},c2, c0)
		{{- end}}
	{{- else }}
		t{{add $j 1}}, c0 = bits.Add64(u{{$j}}, t{{add $j 1}}, c0)
	{{- end}}
	{{- end}}
	
	{{- $k := $.all.NbWordsLastIndex}}

	m := qInvNeg * t0

	u0, c1 = bits.Mul64(m, q0)
	{{- range $j := $.all.NbWordsIndexesFull}}
	{{- if ne $j 0}}
		{{- if eq $j 1}}
			_, c0 = bits.Add64(t0, c1, 0)
		{{- else}}
			t{{sub $j 2}}, c0 = bits.Add64(t{{sub $j 1}}, c1, c0)
		{{- end}}
		u{{$j}}, c1 = bits.Mul64(m, q{{$j}})
	{{- end}}
	{{- end}}
	{{/* TODO @gbotrel it seems this can create a carry (c0) -- study the bounds */}}
	t{{sub $.all.NbWordsLastIndex 1}}, c0 = bits.Add64(0, c1, c0) 
	u{{$k}}, _ = bits.Add64(u{{$k}}, 0, c0)

	{{- range $j := $.all.NbWordsIndexesFull}}
		{{- if eq $j 0}}
			t{{$j}}, c0 = bits.Add64(u{{$j}}, t{{$j}}, 0)
		{{- else if eq $j $.all.NbWordsLastIndex}}
			c2, _ = bits.Add64(c2, 0, c0)
		{{- else}}
			t{{$j}}, c0 = bits.Add64(u{{$j}}, t{{$j}}, c0)
		{{- end}}
	{{- end}}

	{{- $l := sub $.all.NbWordsLastIndex 1}}
	t{{$l}}, c0 = bits.Add64(t{{$k}}, t{{$l}}, 0)
	t{{$k}}, _ = bits.Add64(u{{$k}}, c2, c0)

}
{{- end}}


{{- range $i := $.all.NbWordsIndexesFull}}
z[{{$i}}] = t{{$i}}
{{- end}}

{{ end }}



{{ define "square_nocarry" }}
var {{range $i := .all.NbWordsIndexesFull}}t{{$i}}{{- if ne $i $.all.NbWordsLastIndex}},{{- end}}{{- end}} uint64
var {{range $i := $.all.NbWordsIndexesFull}}u{{$i}}{{- if ne $i $.all.NbWordsLastIndex}},{{- end}}{{- end}} uint64
var {{range $i := interval 0 (add $.all.NbWordsLastIndex 1)}}lo{{$i}}{{- if ne $i $.all.NbWordsLastIndex}},{{- end}}{{- end}} uint64

// note that if hi, _ = bits.Mul64() didn't generate
// UMULH and MUL, (but just UMULH) we could use same pattern
// as in mulRaw and reduce the stack space of this function (no need for lo..)

{{- range $i := .all.NbWordsIndexesFull}}
{

	{{$jStart := add $i 1}}
	{{$jEnd := add $.all.NbWordsLastIndex 1}}

	var c0, c2 uint64


	// for j=i+1 to N-1
	//     p,C,t[j] = 2*a[j]*a[i] + t[j] + (p,C)
	// A = C

	{{- if eq $i 0}}
		u{{$i}}, lo1 = bits.Mul64(x[{{$i}}], x[{{$i}}])
		{{- range $j := interval $jStart $jEnd}}
			u{{$j}}, t{{$j}} = bits.Mul64(x[{{$j}}], x[{{$i}}])
		{{- end}}

		// propagate lo, from t[j] to end, twice.
		{{- range $j := interval $jStart $jEnd}}
			{{- if eq $j $jStart}}
				t{{$j}}, c0 = bits.Add64(t{{$j}}, t{{$j}}, 0)
			{{- else }}
				t{{$j}}, c0 = bits.Add64(t{{$j}}, t{{$j}}, c0)
			{{- end}}
			{{- if eq $j $.all.NbWordsLastIndex}}
				c2, _ = bits.Add64(c2, 0, c0)
			{{- end}}
		{{- end}}

		t{{$i}}, c0 = bits.Add64( lo1,t{{$i}}, 0)
	{{- else}}
		{{- range $j := interval (sub $jStart 1) $jEnd}}
			u{{$j}}, lo{{$j}} = bits.Mul64(x[{{$j}}], x[{{$i}}])
		{{- end}}

		// propagate lo, from t[j] to end, twice.
		{{- range $j := interval $jStart $jEnd}}
			{{- if eq $j $jStart}}
				lo{{$j}}, c0 = bits.Add64(lo{{$j}}, lo{{$j}}, 0)
			{{- else }}
				lo{{$j}}, c0 = bits.Add64(lo{{$j}}, lo{{$j}}, c0)
			{{- end}}
			{{- if eq $j $.all.NbWordsLastIndex}}
				c2, _ = bits.Add64(c2, 0, c0)
			{{- end}}
		{{- end}}
		{{- range $j := interval $jStart $jEnd}}
			{{- if eq $j $jStart}}
				t{{$j}}, c0 = bits.Add64(lo{{$j}}, t{{$j}}, 0)
			{{- else }}
				t{{$j}}, c0 = bits.Add64(lo{{$j}}, t{{$j}}, c0)
			{{- end}}
			{{- if eq $j $.all.NbWordsLastIndex}}
				c2, _ = bits.Add64(c2, 0, c0)
			{{- end}}
		{{- end}}

		t{{$i}}, c0 = bits.Add64( lo{{$i}},t{{$i}}, 0)
	{{- end}}


	// propagate u{{$i}} + hi
	{{- range $j := interval $jStart $jEnd}}
		t{{$j}}, c0 = bits.Add64(u{{sub $j 1}}, t{{$j}}, c0)
	{{- end}}
	c2, _ = bits.Add64(u{{$.all.NbWordsLastIndex}}, c2, c0)

	// hi again
	{{- range $j := interval $jStart $jEnd}}
		{{- if eq $j $.all.NbWordsLastIndex}}
		c2, _ = bits.Add64(c2, u{{$j}}, {{- if eq $j $jStart}} 0 {{- else}}c0{{- end}})
		{{- else if eq $j $jStart}}
			t{{add $j 1}}, c0 = bits.Add64(u{{$j}}, t{{add $j 1}}, 0)
		{{- else }}
			t{{add $j 1}}, c0 = bits.Add64(u{{$j}}, t{{add $j 1}}, c0)
		{{- end}}
	{{- end}}

	{{- $k := $.all.NbWordsLastIndex}}

	// this part is unchanged.
	m := qInvNeg * t0
	{{- range $j := $.all.NbWordsIndexesFull}}
		u{{$j}}, lo{{$j}} = bits.Mul64(m, q{{$j}})
	{{- end}}
	{{- range $j := $.all.NbWordsIndexesFull}}
	{{- if ne $j 0}}
		{{- if eq $j 1}}
			_, c0 = bits.Add64(t0, lo{{sub $j 1}}, 0)
		{{- else}}
			t{{sub $j 2}}, c0 = bits.Add64(t{{sub $j 1}}, lo{{sub $j 1}}, c0)
		{{- end}}
	{{- end}}
	{{- end}}
	t{{sub $.all.NbWordsLastIndex 1}}, c0 = bits.Add64(0, lo{{$.all.NbWordsLastIndex}}, c0) 
	u{{$k}}, _ = bits.Add64(u{{$k}}, 0, c0)

	{{- range $j := $.all.NbWordsIndexesFull}}
		{{- if eq $j 0}}
			t{{$j}}, c0 = bits.Add64(u{{$j}}, t{{$j}}, 0)
		{{- else if eq $j $.all.NbWordsLastIndex}}
			c2, _ = bits.Add64(c2, 0, c0)
		{{- else}}
			t{{$j}}, c0 = bits.Add64(u{{$j}}, t{{$j}}, c0)
		{{- end}}
	{{- end}}

	{{- $k := sub $.all.NbWordsLastIndex 0}}
	{{- $l := sub $.all.NbWordsLastIndex 1}}
	t{{$l}}, c0 = bits.Add64(t{{$k}}, t{{$l}}, 0)
	t{{$k}}, _ = bits.Add64(u{{$k}}, c2, c0)
}
{{- end}}


{{- range $i := $.all.NbWordsIndexesFull}}
z[{{$i}}] = t{{$i}}
{{- end}}

{{ end }}


`
