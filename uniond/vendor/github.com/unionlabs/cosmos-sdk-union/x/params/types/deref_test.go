package types

import (
	"reflect"
	"testing"

	"github.com/stretchr/testify/require"
)

func TestKeyTableUnfurlsPointers(t *testing.T) {
	tbl := NewKeyTable()
	validator := func(_ interface{}) error {
		return nil
	}
	tbl = tbl.RegisterType(ParamSetPair{
		Key:         []byte("key"),
		Value:       (*****string)(nil),
		ValidatorFn: validator,
	})

	got := tbl.m["key"]
	want := attribute{
		vfn: validator,
		ty:  reflect.ValueOf("").Type(),
	}
	require.Equal(t, got.ty, want.ty)
}
