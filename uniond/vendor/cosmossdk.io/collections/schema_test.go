package collections

import (
	"testing"

	"github.com/stretchr/testify/require"
)

func TestNameRegex(t *testing.T) {
	require.Regexp(t, nameRegex, "a")
	require.Regexp(t, nameRegex, "ABC")
	require.Regexp(t, nameRegex, "foo1_xyz")
	require.NotRegexp(t, nameRegex, "1foo")
	require.NotRegexp(t, nameRegex, "_bar")
	require.NotRegexp(t, nameRegex, "abc-xyz")
}

func TestGoodSchema(t *testing.T) {
	sk, _ := deps()
	schemaBuilder := NewSchemaBuilder(sk)
	NewMap(schemaBuilder, NewPrefix(1), "abc", Uint64Key, Uint64Value)
	NewMap(schemaBuilder, NewPrefix(2), "def", Uint64Key, Uint64Value)
	_, err := schemaBuilder.Build()
	require.NoError(t, err)
}

func TestBadName(t *testing.T) {
	sk, _ := deps()
	schemaBuilder := NewSchemaBuilder(sk)
	NewMap(schemaBuilder, NewPrefix(1), "123", Uint64Key, Uint64Value)
	_, err := schemaBuilder.Build()
	require.ErrorContains(t, err, "name must match regex")
}

func TestDuplicatePrefix(t *testing.T) {
	sk, _ := deps()
	schemaBuilder := NewSchemaBuilder(sk)
	NewMap(schemaBuilder, NewPrefix(1), "abc", Uint64Key, Uint64Value)
	NewMap(schemaBuilder, NewPrefix(1), "def", Uint64Key, Uint64Value)
	_, err := schemaBuilder.Build()
	require.ErrorContains(t, err, "prefix [1] already taken")
}

func TestDuplicateName(t *testing.T) {
	sk, _ := deps()
	schemaBuilder := NewSchemaBuilder(sk)
	NewMap(schemaBuilder, NewPrefix(1), "abc", Uint64Key, Uint64Value)
	NewMap(schemaBuilder, NewPrefix(2), "abc", Uint64Key, Uint64Value)
	_, err := schemaBuilder.Build()
	require.ErrorContains(t, err, "name abc already taken")
}

func TestOverlappingPrefixes(t *testing.T) {
	sk, _ := deps()
	schemaBuilder := NewSchemaBuilder(sk)
	NewMap(schemaBuilder, NewPrefix("ab"), "ab", Uint64Key, Uint64Value)
	NewMap(schemaBuilder, NewPrefix("abc"), "abc", Uint64Key, Uint64Value)
	_, err := schemaBuilder.Build()
	require.ErrorContains(t, err, "overlapping prefixes")
}

func TestSchemaBuilderCantBeUsedAfterBuild(t *testing.T) {
	sk, _ := deps()
	schemaBuilder := NewSchemaBuilder(sk)
	NewMap(schemaBuilder, NewPrefix(1), "abc", Uint64Key, Uint64Value)
	_, err := schemaBuilder.Build()
	require.NoError(t, err)
	// can't use schema builder safely after calling build
	require.Panics(t, func() {
		NewMap(schemaBuilder, NewPrefix(2), "def", Uint64Key, Uint64Value)
	})
}
