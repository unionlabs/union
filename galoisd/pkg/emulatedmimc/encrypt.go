package emulatedmimc

import (
	"github.com/consensys/gnark/std/math/emulated"
)

// -------------------------------------------------------------------------------------------------
// encryption functions

func pow5[T emulated.FieldParams](field *emulated.Field[T], x *emulated.Element[T]) *emulated.Element[T] {
	r := field.Mul(x, x)
	r = field.Mul(r, r)
	return field.Mul(r, x)
}

func pow7[T emulated.FieldParams](field *emulated.Field[T], x *emulated.Element[T]) *emulated.Element[T] {
	t := field.Mul(x, x)
	r := field.Mul(t, t)
	r = field.Mul(r, t)
	return field.Mul(r, x)
}

func pow17[T emulated.FieldParams](field *emulated.Field[T], x *emulated.Element[T]) *emulated.Element[T] {
	r := field.Mul(x, x)
	r = field.Mul(r, r)
	r = field.Mul(r, r)
	r = field.Mul(r, r)
	return field.Mul(r, x)
}

func encryptPow5[T emulated.FieldParams](h MiMC[T], m *emulated.Element[T]) *emulated.Element[T] {
	x := m
	for i := 0; i < len(h.params); i++ {
		x = pow5(h.field, h.field.Add(h.field.Add(x, h.h), h.params[i]))
	}
	return h.field.Add(x, h.h)
}

func encryptPow7[T emulated.FieldParams](h MiMC[T], m *emulated.Element[T]) *emulated.Element[T] {
	x := m
	for i := 0; i < len(h.params); i++ {
		x = pow7(h.field, h.field.Add(h.field.Add(x, h.h), h.params[i]))
	}
	return h.field.Add(x, h.h)
}

func encryptPow17[T emulated.FieldParams](h MiMC[T], m *emulated.Element[T]) *emulated.Element[T] {
	x := m
	for i := 0; i < len(h.params); i++ {
		// res = (res+key+c)**17
		x = pow17(h.field, h.field.Add(h.field.Add(x, h.h), h.params[i]))
	}
	return h.field.Add(x, h.h)
}
