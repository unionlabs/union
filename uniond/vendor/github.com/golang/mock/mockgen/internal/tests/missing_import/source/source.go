// Package source makes sure output imports its. See #505.
package source

//go:generate mockgen -package source -destination=../output/source_mock.go -source=source.go

type Foo struct{}

type Bar interface {
	Baz(Foo)
}
