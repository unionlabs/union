package types

import (
	"testing"

	"github.com/cosmos/cosmos-sdk/store/types"
	"github.com/stretchr/testify/suite"
)

type storeIntSuite struct {
	suite.Suite
}

func TestStoreIntSuite(t *testing.T) {
	suite.Run(t, new(storeIntSuite))
}

func (s *storeIntSuite) TestAssertNoPrefix() {
	testCases := []struct {
		keys        []string
		expectPanic bool
	}{
		{[]string{""}, false},
		{[]string{"a"}, false},
		{[]string{"a", "b"}, false},
		{[]string{"a", "b1"}, false},
		{[]string{"b2", "b1"}, false},
		{[]string{"b1", "bb", "b2"}, false},

		{[]string{"a", ""}, true},
		{[]string{"a", "b", "a"}, true},
		{[]string{"a", "b", "aa"}, true},
		{[]string{"a", "b", "ab"}, true},
		{[]string{"a", "b1", "bb", "b12"}, true},
	}

	require := s.Require()
	for _, tc := range testCases {
		if tc.expectPanic {
			require.Panics(func() { assertNoPrefix(tc.keys) })
		} else {
			assertNoPrefix(tc.keys)
		}
	}
}

func (s *storeIntSuite) TestNewKVStoreKeys() {
	require := s.Require()
	require.Panics(func() { NewKVStoreKeys("a1", "a") }, "should fail one key is a prefix of another one")

	require.Equal(map[string]*types.KVStoreKey{}, NewKVStoreKeys())
	require.Equal(1, len(NewKVStoreKeys("one")))

	key := "baca"
	stores := NewKVStoreKeys(key, "a")
	require.Len(stores, 2)
	require.Equal(key, stores[key].Name())
}
