package element

const Reduce = `
{{ define "reduce" }}
// if z ⩾ q → z -= q
if !z.smallerThanModulus() {
{{- if eq $.NbWords 1}}
		z[0] -= q
{{- else}}
	var b uint64
	z[0], b = bits.Sub64(z[0], q0, 0)
	{{- range $i := .NbWordsIndexesNoZero}}
		{{-  if eq $i $.NbWordsLastIndex}}
			z[{{$i}}], _ = bits.Sub64(z[{{$i}}], q{{$i}}, b)
		{{-  else  }}
			z[{{$i}}], b = bits.Sub64(z[{{$i}}], q{{$i}}, b)
		{{- end}}
	{{- end}}
{{-  end }}
}

{{-  end }}

`
