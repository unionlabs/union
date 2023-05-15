package log

import (
	"fmt"

	cmtbytes "github.com/cometbft/cometbft/libs/bytes"
)

type LazySprintf struct {
	format string
	args   []interface{}
}

// NewLazySprintf defers fmt.Sprintf until the Stringer interface is invoked.
// This is particularly useful for avoiding calling Sprintf when debugging is not
// active.
func NewLazySprintf(format string, args ...interface{}) *LazySprintf {
	return &LazySprintf{format, args}
}

func (l *LazySprintf) String() string {
	return fmt.Sprintf(l.format, l.args...)
}

type LazyBlockHash struct {
	block hashable
}

type hashable interface {
	Hash() cmtbytes.HexBytes
}

// NewLazyBlockHash defers block Hash until the Stringer interface is invoked.
// This is particularly useful for avoiding calling Sprintf when debugging is not
// active.
func NewLazyBlockHash(block hashable) *LazyBlockHash {
	return &LazyBlockHash{block}
}

func (l *LazyBlockHash) String() string {
	return l.block.Hash().String()
}
