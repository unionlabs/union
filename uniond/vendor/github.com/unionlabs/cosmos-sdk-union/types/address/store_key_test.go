package address_test

import (
	"testing"

	"github.com/stretchr/testify/suite"

	"github.com/cosmos/cosmos-sdk/types/address"
)

func TestStoreKeySuite(t *testing.T) {
	suite.Run(t, new(StoreKeySuite))
}

type StoreKeySuite struct{ suite.Suite }

func (suite *StoreKeySuite) TestLengthPrefix() {
	require := suite.Require()
	addr10byte := []byte{0, 1, 2, 3, 4, 5, 6, 7, 8, 9}
	addr20byte := []byte{0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19}
	var addr0byte []byte
	addr256byte := make([]byte, 256)

	tests := []struct {
		name        string
		addr        []byte
		expStoreKey []byte
		expErr      bool
	}{
		{"10-byte address", addr10byte, append([]byte{byte(10)}, addr10byte...), false},
		{"20-byte address", addr20byte, append([]byte{byte(20)}, addr20byte...), false},
		{"0-byte address", addr0byte, addr0byte, false},
		{"256-byte address (too long)", addr256byte, nil, true},
	}

	for _, tt := range tests {
		suite.Run(tt.name, func() {
			storeKey, err := address.LengthPrefix(tt.addr)
			if tt.expErr {
				require.Error(err)
			} else {
				require.NoError(err)
				require.Equal(tt.expStoreKey, storeKey)
			}
		})
	}
}

func (suite *StoreKeySuite) TestMustLengthPrefix() {
	require := suite.Require()
	addr10byte := []byte{0, 1, 2, 3, 4, 5, 6, 7, 8, 9}
	addr256byte := make([]byte, 256)

	tests := []struct {
		name        string
		addr        []byte
		expStoreKey []byte
		expPanic    bool
	}{
		{"10-byte address with length prefix", addr10byte, append([]byte{byte(10)}, addr10byte...), false},
		{"256-byte address triggers panic due to excessive length", addr256byte, nil, true},
	}

	for _, tt := range tests {
		suite.Run(tt.name, func() {
			defer func() {
				r := recover()
				if tt.expPanic {
					require.NotNil(r)
				} else {
					require.Nil(r)
				}
			}()
			storeKey := address.MustLengthPrefix(tt.addr)
			require.Equal(tt.expStoreKey, storeKey)
		})
	}
}
