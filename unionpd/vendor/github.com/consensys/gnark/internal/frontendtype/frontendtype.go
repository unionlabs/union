// Package frontendtype allows to assert frontend type.
package frontendtype

type Type int

const (
	R1CS Type = iota
	SCS
)

type FrontendTyper interface {
	FrontendType() Type
}
