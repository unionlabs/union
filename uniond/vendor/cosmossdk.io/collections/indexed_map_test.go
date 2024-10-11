package collections_test

import (
	"testing"

	"github.com/stretchr/testify/require"

	"cosmossdk.io/collections"
	"cosmossdk.io/collections/colltest"
	"cosmossdk.io/collections/indexes"
	"cosmossdk.io/core/testing"
)

type company struct {
	City string
	Vat  uint64
}

type companyIndexes struct {
	// City is an index of the company indexed map. It indexes a company
	// given its city. The index is multi, meaning that there can be multiple
	// companies from the same city.
	City *indexes.Multi[string, string, company]
	// Vat is an index of the company indexed map. It indexes a company
	// given its VAT number. The index is unique, meaning that there can be
	// only one VAT number for a company.
	Vat *indexes.Unique[uint64, string, company]
}

func (c companyIndexes) IndexesList() []collections.Index[string, company] {
	return []collections.Index[string, company]{c.City, c.Vat}
}

func newTestIndexedMap(schema *collections.SchemaBuilder) *collections.IndexedMap[string, company, companyIndexes] {
	return collections.NewIndexedMap(schema, collections.NewPrefix(0), "companies", collections.StringKey, colltest.MockValueCodec[company](),
		companyIndexes{
			City: indexes.NewMulti(schema, collections.NewPrefix(1), "companies_by_city", collections.StringKey, collections.StringKey, func(pk string, value company) (string, error) {
				return value.City, nil
			}),
			Vat: indexes.NewUnique(schema, collections.NewPrefix(2), "companies_by_vat", collections.Uint64Key, collections.StringKey, func(pk string, value company) (uint64, error) {
				return value.Vat, nil
			}),
		},
	)
}

func TestIndexedMap(t *testing.T) {
	ctx := coretesting.Context()
	sk := coretesting.KVStoreService(ctx, "test")

	schema := collections.NewSchemaBuilder(sk)

	im := newTestIndexedMap(schema)

	// test insertion
	err := im.Set(ctx, "1", company{
		City: "milan",
		Vat:  0,
	})
	require.NoError(t, err)

	err = im.Set(ctx, "2", company{
		City: "milan",
		Vat:  1,
	})
	require.NoError(t, err)

	err = im.Set(ctx, "3", company{
		City: "milan",
		Vat:  4,
	})
	require.NoError(t, err)

	pk, err := im.Indexes.Vat.MatchExact(ctx, 1)
	require.NoError(t, err)
	require.Equal(t, "2", pk)

	// test a set which updates the indexes
	err = im.Set(ctx, "2", company{
		City: "milan",
		Vat:  2,
	})
	require.NoError(t, err)

	pk, err = im.Indexes.Vat.MatchExact(ctx, 2)
	require.NoError(t, err)
	require.Equal(t, "2", pk)

	_, err = im.Indexes.Vat.MatchExact(ctx, 1)
	require.ErrorIs(t, err, collections.ErrNotFound)

	// test removal
	err = im.Remove(ctx, "2")
	require.NoError(t, err)
	_, err = im.Indexes.Vat.MatchExact(ctx, 2)
	require.ErrorIs(t, err, collections.ErrNotFound)

	// test iteration
	iter, err := im.Iterate(ctx, nil)
	require.NoError(t, err)
	keys, err := iter.Keys()
	require.NoError(t, err)
	require.Equal(t, []string{"1", "3"}, keys)

	// test get
	v, err := im.Get(ctx, "3")
	require.NoError(t, err)
	require.Equal(t, company{"milan", 4}, v)
}

type inferIndex struct {
	City *indexes.Multi[string, string, company]
	Vat  *indexes.Unique[uint64, string, company]
}

func newInferIndex(schema *collections.SchemaBuilder) *inferIndex {
	return &inferIndex{
		City: indexes.NewMulti(schema, collections.NewPrefix(1), "companies_by_city", collections.StringKey, collections.StringKey, func(pk string, value company) (string, error) {
			return value.City, nil
		}),
		Vat: indexes.NewUnique(schema, collections.NewPrefix(2), "companies_by_vat", collections.Uint64Key, collections.StringKey, func(pk string, value company) (uint64, error) {
			return value.Vat, nil
		}),
	}
}

func TestIndexedMapInfer(t *testing.T) {
	sk := coretesting.KVStoreService(coretesting.Context(), "test")
	schema := collections.NewSchemaBuilder(sk)

	_, err := collections.NewIndexedMapSafe(schema, collections.NewPrefix(0), "im", collections.StringKey, colltest.MockValueCodec[company](), newInferIndex(schema))
	require.NoError(t, err)
}
