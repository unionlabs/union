package textual_test

import (
	"context"
	"testing"

	"github.com/stretchr/testify/require"
	"google.golang.org/protobuf/reflect/protoreflect"

	"cosmossdk.io/x/tx/signing/textual"
)

func TestBool(t *testing.T) {
	// test true
	rend := textual.NewBoolValueRenderer()
	screens, err := rend.Format(context.Background(), protoreflect.ValueOfBool(true))
	require.NoError(t, err)
	require.Equal(t, 1, len(screens))
	require.Equal(t, "True", screens[0].Content)
	val, err := rend.Parse(context.Background(), screens)
	require.NoError(t, err)
	require.Equal(t, true, val.Bool())

	// test false
	screens, err = rend.Format(context.Background(), protoreflect.ValueOfBool(false))
	require.NoError(t, err)
	require.Equal(t, 1, len(screens))
	require.Equal(t, "False", screens[0].Content)
	val, err = rend.Parse(context.Background(), screens)
	require.NoError(t, err)
	require.Equal(t, false, val.Bool())
}
