package types_test

import (
	"testing"

	"gotest.tools/v3/assert"

	"cosmossdk.io/store/types"
)

func TestPrefixEndBytes(t *testing.T) {
	t.Parallel()
	testCases := []struct {
		prefix   []byte
		expected []byte
	}{
		{[]byte{byte(55), byte(255), byte(255), byte(0)}, []byte{byte(55), byte(255), byte(255), byte(1)}},
		{[]byte{byte(55), byte(255), byte(255), byte(15)}, []byte{byte(55), byte(255), byte(255), byte(16)}},
		{[]byte{byte(55), byte(200), byte(255)}, []byte{byte(55), byte(201)}},
		{[]byte{byte(55), byte(255), byte(255)}, []byte{byte(56)}},
		{[]byte{byte(255), byte(255), byte(255)}, nil},
		{[]byte{byte(255)}, nil},
		{nil, nil},
	}

	for _, test := range testCases {
		end := types.PrefixEndBytes(test.prefix)
		assert.DeepEqual(t, test.expected, end)
	}
}

func TestInclusiveEndBytes(t *testing.T) {
	t.Parallel()
	assert.DeepEqual(t, []byte{0x00}, types.InclusiveEndBytes(nil))
	bs := []byte("test")
	assert.DeepEqual(t, append(bs, byte(0x00)), types.InclusiveEndBytes(bs))
}
