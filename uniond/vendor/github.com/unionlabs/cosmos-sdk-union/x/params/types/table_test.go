package types_test

import (
	"testing"
	"time"

	"github.com/stretchr/testify/require"

	"cosmossdk.io/x/params/types"
)

func TestKeyTable(t *testing.T) {
	table := types.NewKeyTable()

	require.Panics(t, func() { table.RegisterType(types.ParamSetPair{[]byte(""), nil, nil}) })
	require.Panics(t, func() { table.RegisterType(types.ParamSetPair{[]byte("!@#$%"), nil, nil}) })
	require.Panics(t, func() { table.RegisterType(types.ParamSetPair{[]byte("hello,"), nil, nil}) })
	require.Panics(t, func() { table.RegisterType(types.ParamSetPair{[]byte("hello"), nil, nil}) })

	require.NotPanics(t, func() {
		table.RegisterType(types.ParamSetPair{keyBondDenom, string("stake"), validateBondDenom})
	})
	require.NotPanics(t, func() {
		table.RegisterType(types.ParamSetPair{keyMaxValidators, uint16(100), validateMaxValidators})
	})
	require.Panics(t, func() {
		table.RegisterType(types.ParamSetPair{keyUnbondingTime, time.Duration(1), nil})
	})
	require.NotPanics(t, func() {
		table.RegisterType(types.ParamSetPair{keyUnbondingTime, time.Duration(1), validateMaxValidators})
	})
	require.NotPanics(t, func() {
		newTable := types.NewKeyTable()
		newTable.RegisterParamSet(&params{})
	})

	require.Panics(t, func() { table.RegisterParamSet(&params{}) })
	require.Panics(t, func() { types.NewKeyTable(types.ParamSetPair{[]byte(""), nil, nil}) })

	require.NotPanics(t, func() {
		types.NewKeyTable(
			types.ParamSetPair{[]byte("test"), string("stake"), validateBondDenom},
			types.ParamSetPair{[]byte("test2"), uint16(100), validateMaxValidators},
		)
	})
}
