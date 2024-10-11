package blockchain

import (
	"testing"

	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func TestIsInvalidBlock(t *testing.T) {
	require.Equal(t, true, IsInvalidBlock(ErrInvalidPayload)) // Already wrapped.
	err := invalidBlock{error: ErrInvalidPayload}
	require.Equal(t, true, IsInvalidBlock(err))

	newErr := errors.Wrap(err, "wrap me")
	require.Equal(t, true, IsInvalidBlock(newErr))
	require.DeepEqual(t, [][32]byte(nil), InvalidAncestorRoots(err))
}

func TestInvalidBlockRoot(t *testing.T) {
	require.Equal(t, [32]byte{}, InvalidBlockRoot(ErrUndefinedExecutionEngineError))
	require.Equal(t, [32]byte{}, InvalidBlockRoot(ErrInvalidPayload))

	err := invalidBlock{error: ErrInvalidPayload, root: [32]byte{'a'}}
	require.Equal(t, [32]byte{'a'}, InvalidBlockRoot(err))
	require.DeepEqual(t, [][32]byte(nil), InvalidAncestorRoots(err))

	newErr := errors.Wrap(err, "wrap me")
	require.Equal(t, [32]byte{'a'}, InvalidBlockRoot(newErr))
}

func TestInvalidRoots(t *testing.T) {
	roots := [][32]byte{{'d'}, {'b'}, {'c'}}
	err := invalidBlock{error: ErrInvalidPayload, root: [32]byte{'a'}, invalidAncestorRoots: roots}

	require.Equal(t, true, IsInvalidBlock(err))
	require.Equal(t, [32]byte{'a'}, InvalidBlockRoot(err))
	require.DeepEqual(t, roots, InvalidAncestorRoots(err))

	newErr := errors.Wrap(err, "wrap me")
	require.Equal(t, true, IsInvalidBlock(err))
	require.Equal(t, [32]byte{'a'}, InvalidBlockRoot(newErr))
	require.DeepEqual(t, roots, InvalidAncestorRoots(newErr))
}
