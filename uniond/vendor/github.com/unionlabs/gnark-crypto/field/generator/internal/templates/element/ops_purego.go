package element

const OpsNoAsm = `

import "math/bits"

{{ $mulConsts := list 3 5 13 }}
{{- range $i := $mulConsts }}

// MulBy{{$i}} x *= {{$i}} (mod q)
func MulBy{{$i}}(x *{{$.ElementName}}) {
	{{- if eq 1 $.NbWords}}
	var y {{$.ElementName}}
	y.SetUint64({{$i}})
	x.Mul(x, &y)
	{{- else}}
		{{- if eq $i 3}}
			_x := *x
			x.Double(x).Add(x, &_x)
		{{- else if eq $i 5}}
			_x := *x
			x.Double(x).Double(x).Add(x, &_x)
		{{- else if eq $i 13}}
			var y = {{$.ElementName}}{
				{{- range $i := $.Thirteen}}
				{{$i}},{{end}}
			}
			x.Mul(x, &y)
		{{- else }}
			NOT IMPLEMENTED
		{{- end}}
	{{- end}}
}

{{- end}}

// Butterfly sets
//  a = a + b (mod q)
//  b = a - b (mod q)
func Butterfly(a, b *{{.ElementName}}) {
	_butterflyGeneric(a, b)
}


func fromMont(z *{{.ElementName}} ) {
	_fromMontGeneric(z)
}

func reduce(z *{{.ElementName}})  {
	_reduceGeneric(z)
}



// Mul z = x * y (mod q)
{{- if $.NoCarry}}
//
// x and y must be less than q
{{- end }}
func (z *{{.ElementName}}) Mul(x, y *{{.ElementName}}) *{{.ElementName}} {
	{{- if eq $.NbWords 1}}
		{{ template "mul_cios_one_limb" dict "all" . "V1" "x" "V2" "y" }}
	{{- else }}
		{{ mul_doc $.NoCarry }}
		{{- if $.NoCarry}}
			{{ template "mul_nocarry" dict "all" . "V1" "x" "V2" "y"}}
		{{- else}}
			{{ template "mul_cios" dict "all" . "V1" "x" "V2" "y" "ReturnZ" true}}
		{{- end}}
		{{ template "reduce"  . }}
	{{- end }}
	return z
}

// Square z = x * x (mod q)
{{- if $.NoCarry}}
//
// x must be less than q
{{- end }}
func (z *{{.ElementName}}) Square(x *{{.ElementName}}) *{{.ElementName}} {
	// see Mul for algorithm documentation
	{{- if eq $.NbWords 1}}
		{{ template "mul_cios_one_limb" dict "all" . "V1" "x" "V2" "x" }}
	{{- else }}
		{{- if $.NoCarry}}
			{{ template "mul_nocarry" dict "all" . "V1" "x" "V2" "x"}}
		{{- else}}
			{{ template "mul_cios" dict "all" . "V1" "x" "V2" "x" "ReturnZ" true}}
		{{- end}}
		{{ template "reduce"  . }}
	{{- end }}
	return z
}

`
