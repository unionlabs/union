package badger

import (
	"math"
	"reflect"
	"testing"

	"github.com/stretchr/testify/require"
)

// Regression test for github.com/dgraph-io/badger/pull/1800
func TestLargeEncode(t *testing.T) {
	var headerEnc [maxHeaderSize]byte
	h := header{math.MaxUint32, math.MaxUint32, math.MaxUint64, math.MaxUint8, math.MaxUint8}
	require.NotPanics(t, func() { _ = h.Encode(headerEnc[:]) })
}

func TestNumFieldsHeader(t *testing.T) {
	// maxHeaderSize must correspond with any changes made to header
	require.Equal(t, 5, reflect.TypeOf(header{}).NumField())
}
