package faux

type Foreign interface {
	Method() Return
	Embedded
	error
}

type Embedded interface{}

type Return interface{}
