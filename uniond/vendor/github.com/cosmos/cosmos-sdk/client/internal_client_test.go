package client

import (
	"testing"

	"github.com/stretchr/testify/require"
)

func TestFailingInterfaceRegistry(t *testing.T) {
	reg := failingInterfaceRegistry{}

	require.Error(t, reg.UnpackAny(nil, nil))
	_, err := reg.Resolve("")
	require.Error(t, err)

	require.Panics(t, func() {
		reg.RegisterInterface("", nil)
	})
	require.Panics(t, func() {
		reg.RegisterImplementations(nil, nil)
	})
	require.Panics(t, func() {
		reg.ListAllInterfaces()
	})
	require.Panics(t, func() {
		reg.ListImplementations("")
	})
	require.Panics(t, func() {
		reg.EnsureRegistered(nil)
	})
}
