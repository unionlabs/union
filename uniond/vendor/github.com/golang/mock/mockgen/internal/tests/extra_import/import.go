// Package extra_import makes sure output does not import it. See #515.
package extra_import

//go:generate mockgen -destination mock.go -package extra_import . Foo

type Message struct {
	Text string
}

type Foo interface {
	Bar(channels []string, message chan<- Message)
}
