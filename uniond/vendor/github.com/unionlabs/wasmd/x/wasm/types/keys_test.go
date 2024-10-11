package types

import (
	"bytes"
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestGetContractByCodeIDSecondaryIndexPrefix(t *testing.T) {
	specs := map[string]struct {
		src uint64
		exp []byte
	}{
		"small number": {
			src: 1,
			exp: []byte{6, 0, 0, 0, 0, 0, 0, 0, 1},
		},
		"big number": {
			src: 1 << (8 * 7),
			exp: []byte{6, 1, 0, 0, 0, 0, 0, 0, 0},
		},
	}
	for msg, spec := range specs {
		t.Run(msg, func(t *testing.T) {
			got := GetContractByCodeIDSecondaryIndexPrefix(spec.src)
			assert.Equal(t, spec.exp, got)
		})
	}
}

func TestGetContractCodeHistoryElementPrefix(t *testing.T) {
	// test that contract addresses of 20 length are still supported
	addr := bytes.Repeat([]byte{4}, 20)
	got := GetContractCodeHistoryElementPrefix(addr)
	exp := []byte{
		5,                            // prefix
		4, 4, 4, 4, 4, 4, 4, 4, 4, 4, // address 20 bytes
		4, 4, 4, 4, 4, 4, 4, 4, 4, 4,
	}
	assert.Equal(t, exp, got)

	addr = bytes.Repeat([]byte{4}, ContractAddrLen)
	got = GetContractCodeHistoryElementPrefix(addr)
	exp = []byte{
		5,                            // prefix
		4, 4, 4, 4, 4, 4, 4, 4, 4, 4, // address 32 bytes
		4, 4, 4, 4, 4, 4, 4, 4, 4, 4,
		4, 4, 4, 4, 4, 4, 4, 4, 4, 4,
		4, 4,
	}
	assert.Equal(t, exp, got)
}

func TestGetContractByCreatedSecondaryIndexKey(t *testing.T) {
	e := ContractCodeHistoryEntry{
		CodeID:  1,
		Updated: &AbsoluteTxPosition{2 + 1<<(8*7), 3 + 1<<(8*7)},
	}

	// test that contract addresses of 20 length are still supported
	addr := bytes.Repeat([]byte{4}, 20)
	got := GetContractByCreatedSecondaryIndexKey(addr, e)
	exp := []byte{
		6,                      // prefix
		0, 0, 0, 0, 0, 0, 0, 1, // codeID
		1, 0, 0, 0, 0, 0, 0, 2, // height
		1, 0, 0, 0, 0, 0, 0, 3, // index
		4, 4, 4, 4, 4, 4, 4, 4, 4, 4, // address 32 bytes
		4, 4, 4, 4, 4, 4, 4, 4, 4, 4,
	}
	assert.Equal(t, exp, got)

	addr = bytes.Repeat([]byte{4}, ContractAddrLen)
	got = GetContractByCreatedSecondaryIndexKey(addr, e)
	exp = []byte{
		6,                      // prefix
		0, 0, 0, 0, 0, 0, 0, 1, // codeID
		1, 0, 0, 0, 0, 0, 0, 2, // height
		1, 0, 0, 0, 0, 0, 0, 3, // index
		4, 4, 4, 4, 4, 4, 4, 4, 4, 4, // address 32 bytes
		4, 4, 4, 4, 4, 4, 4, 4, 4, 4,
		4, 4, 4, 4, 4, 4, 4, 4, 4, 4,
		4, 4,
	}
	assert.Equal(t, exp, got)
}

func TestGetContractByCreatorSecondaryIndexKey(t *testing.T) {
	creatorAddr := bytes.Repeat([]byte{4}, 20)
	e := ContractCodeHistoryEntry{
		CodeID:  1,
		Updated: &AbsoluteTxPosition{2 + 1<<(8*7), 3 + 1<<(8*7)},
	}

	// test that contract addresses of 20 length are still supported
	contractAddr := bytes.Repeat([]byte{4}, 20)
	got := GetContractByCreatorSecondaryIndexKey(creatorAddr, e.Updated.Bytes(), contractAddr)
	exp := []byte{
		9,                            // prefix
		20,                           // creator address length
		4, 4, 4, 4, 4, 4, 4, 4, 4, 4, // creator address with fixed length prefix
		4, 4, 4, 4, 4, 4, 4, 4, 4, 4,
		1, 0, 0, 0, 0, 0, 0, 2, // height
		1, 0, 0, 0, 0, 0, 0, 3, // index
		4, 4, 4, 4, 4, 4, 4, 4, 4, 4, // address 20 bytes
		4, 4, 4, 4, 4, 4, 4, 4, 4, 4,
	}
	assert.Equal(t, exp, got)

	// test that contract addresses of 32 length are still supported
	contractAddr = bytes.Repeat([]byte{4}, 32)
	got = GetContractByCreatorSecondaryIndexKey(creatorAddr, e.Updated.Bytes(), contractAddr)
	exp = []byte{
		9,                            // prefix
		20,                           // creator address length
		4, 4, 4, 4, 4, 4, 4, 4, 4, 4, // creator address with fixed length prefix
		4, 4, 4, 4, 4, 4, 4, 4, 4, 4,
		1, 0, 0, 0, 0, 0, 0, 2, // height
		1, 0, 0, 0, 0, 0, 0, 3, // index
		4, 4, 4, 4, 4, 4, 4, 4, 4, 4, // address 32 bytes
		4, 4, 4, 4, 4, 4, 4, 4, 4, 4,
		4, 4, 4, 4, 4, 4, 4, 4, 4, 4,
		4, 4,
	}
	assert.Equal(t, exp, got)

	// test that creator is contract addresses of 32 length
	contractAddr = bytes.Repeat([]byte{4}, 32)
	creatorAddr = bytes.Repeat([]byte{8}, 32)
	got = GetContractByCreatorSecondaryIndexKey(creatorAddr, e.Updated.Bytes(), contractAddr)
	exp = []byte{
		9,                            // prefix
		32,                           // creator address length
		8, 8, 8, 8, 8, 8, 8, 8, 8, 8, // creator address is a contract address
		8, 8, 8, 8, 8, 8, 8, 8, 8, 8,
		8, 8, 8, 8, 8, 8, 8, 8, 8, 8,
		8, 8,
		1, 0, 0, 0, 0, 0, 0, 2, // height
		1, 0, 0, 0, 0, 0, 0, 3, // index

		4, 4, 4, 4, 4, 4, 4, 4, 4, 4, // address 32 bytes
		4, 4, 4, 4, 4, 4, 4, 4, 4, 4,
		4, 4, 4, 4, 4, 4, 4, 4, 4, 4,
		4, 4,
	}
	assert.Equal(t, exp, got)
}
