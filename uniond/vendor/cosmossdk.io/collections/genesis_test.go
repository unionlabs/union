package collections

import (
	"bytes"
	"context"
	"io"
	"testing"

	"github.com/stretchr/testify/require"

	"cosmossdk.io/core/appmodule"
)

func TestDefaultGenesis(t *testing.T) {
	f := initFixture(t)
	var writers []*bufCloser
	require.NoError(t, f.schema.DefaultGenesis(func(field string) (io.WriteCloser, error) {
		w := newBufCloser(t, "")
		writers = append(writers, w)
		return w, nil
	}))
	require.Len(t, writers, 4)
	require.Equal(t, `[]`, writers[0].Buffer.String())
	require.Equal(t, `[]`, writers[1].Buffer.String())
	require.Equal(t, `[]`, writers[2].Buffer.String())
	require.Equal(t, `[]`, writers[3].Buffer.String())
}

func TestValidateGenesis(t *testing.T) {
	f := initFixture(t)
	require.NoError(t, f.schema.ValidateGenesis(createTestGenesisSource(t)))
}

func TestImportGenesis(t *testing.T) {
	f := initFixture(t)
	require.NoError(t, f.schema.InitGenesis(f.ctx, createTestGenesisSource(t)))
	// assert map correct genesis
	mapIt, err := f.m.Iterate(f.ctx, nil)
	require.NoError(t, err)
	defer mapIt.Close()

	kvs, err := mapIt.KeyValues()
	require.NoError(t, err)
	require.Equal(t, KeyValue[string, uint64]{Key: "abc", Value: 1}, kvs[0])
	require.Equal(t, KeyValue[string, uint64]{Key: "def", Value: 2}, kvs[1])

	// assert item correct genesis
	x, err := f.i.Get(f.ctx)
	require.NoError(t, err)
	require.Equal(t, "superCoolItem", x)

	// assert keyset correct genesis
	ksIt, err := f.ks.Iterate(f.ctx, nil)
	require.NoError(t, err)
	defer ksIt.Close()

	keys, err := ksIt.Keys()
	require.NoError(t, err)
	require.Equal(t, []string{"0", "1", "2"}, keys)

	// assert sequence correct genesis
	seq, err := f.s.Peek(f.ctx)
	require.NoError(t, err)
	require.Equal(t, uint64(1000), seq)
}

func TestExportGenesis(t *testing.T) {
	f := initFixture(t)
	require.NoError(t, f.schema.InitGenesis(f.ctx, createTestGenesisSource(t)))

	var writers []*bufCloser
	require.NoError(t, f.schema.ExportGenesis(f.ctx, func(field string) (io.WriteCloser, error) {
		w := newBufCloser(t, "")
		writers = append(writers, w)
		return w, nil
	}))
	require.Len(t, writers, 4)
	require.Equal(t, expectedItemGenesis, writers[0].Buffer.String())
	require.Equal(t, expectedKeySetGenesis, writers[1].Buffer.String())
	require.Equal(t, expectedMapGenesis, writers[2].Buffer.String())
	require.Equal(t, expectedSequenceGenesis, writers[3].Buffer.String())
}

type testFixture struct {
	schema Schema
	ctx    context.Context
	m      Map[string, uint64]
	i      Item[string]
	s      Sequence
	ks     KeySet[string]
}

func initFixture(t *testing.T) *testFixture {
	t.Helper()
	sk, ctx := deps()
	schemaBuilder := NewSchemaBuilder(sk)
	m := NewMap(schemaBuilder, NewPrefix(1), "map", StringKey, Uint64Value)
	i := NewItem(schemaBuilder, NewPrefix(2), "item", StringValue)
	s := NewSequence(schemaBuilder, NewPrefix(3), "sequence")
	ks := NewKeySet(schemaBuilder, NewPrefix(4), "key_set", StringKey)
	schema, err := schemaBuilder.Build()
	require.NoError(t, err)
	return &testFixture{
		schema: schema,
		ctx:    ctx,
		m:      m,
		i:      i,
		s:      s,
		ks:     ks,
	}
}

func createTestGenesisSource(t *testing.T) appmodule.GenesisSource {
	t.Helper()
	expectedOrder := []string{"item", "key_set", "map", "sequence"}
	currentIndex := 0
	return func(field string) (io.ReadCloser, error) {
		require.Equal(t, expectedOrder[currentIndex], field, "unordered genesis")
		currentIndex++

		switch field {
		case "map":
			return newBufCloser(t, expectedMapGenesis), nil
		case "item":
			return newBufCloser(t, expectedItemGenesis), nil
		case "key_set":
			return newBufCloser(t, expectedKeySetGenesis), nil
		case "sequence":
			return newBufCloser(t, expectedSequenceGenesis), nil
		default:
			return nil, nil
		}
	}
}

const (
	expectedMapGenesis      = `[{"key":"abc","value":"1"},{"key":"def","value":"2"}]`
	expectedItemGenesis     = `[{"key":"item","value":"superCoolItem"}]`
	expectedKeySetGenesis   = `[{"key":"0"},{"key":"1"},{"key":"2"}]`
	expectedSequenceGenesis = `[{"key":"item","value":"1000"}]`
)

type bufCloser struct {
	*bytes.Buffer
	closed bool
}

func (b *bufCloser) Close() error {
	b.closed = true
	return nil
}

func newBufCloser(t *testing.T, str string) *bufCloser {
	t.Helper()
	b := &bufCloser{
		Buffer: bytes.NewBufferString(str),
		closed: false,
	}
	// this ensures Close was called by the implementation
	t.Cleanup(func() {
		require.True(t, b.closed)
	})
	return b
}
