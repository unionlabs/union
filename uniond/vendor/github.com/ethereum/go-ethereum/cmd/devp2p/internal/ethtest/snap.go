// Copyright 2022 The go-ethereum Authors
// This file is part of go-ethereum.
//
// go-ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// go-ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with go-ethereum. If not, see <http://www.gnu.org/licenses/>.

package ethtest

import (
	"bytes"
	"errors"
	"fmt"
	"math/rand"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/crypto"
	"github.com/ethereum/go-ethereum/eth/protocols/snap"
	"github.com/ethereum/go-ethereum/internal/utesting"
	"github.com/ethereum/go-ethereum/trie"
	"github.com/ethereum/go-ethereum/trie/trienode"
	"golang.org/x/crypto/sha3"
)

func (s *Suite) TestSnapStatus(t *utesting.T) {
	conn, err := s.dialSnap()
	if err != nil {
		t.Fatalf("dial failed: %v", err)
	}
	defer conn.Close()
	if err := conn.peer(s.chain, nil); err != nil {
		t.Fatalf("peering failed: %v", err)
	}
}

type accRangeTest struct {
	nBytes uint64
	root   common.Hash
	origin common.Hash
	limit  common.Hash

	expAccounts int
	expFirst    common.Hash
	expLast     common.Hash
}

// TestSnapGetAccountRange various forms of GetAccountRange requests.
func (s *Suite) TestSnapGetAccountRange(t *utesting.T) {
	var (
		root           = s.chain.RootAt(999)
		ffHash         = common.MaxHash
		zero           = common.Hash{}
		firstKeyMinus1 = common.HexToHash("0x00bf49f440a1cd0527e4d06e2765654c0f56452257516d793a9b8d604dcfdf29")
		firstKey       = common.HexToHash("0x00bf49f440a1cd0527e4d06e2765654c0f56452257516d793a9b8d604dcfdf2a")
		firstKeyPlus1  = common.HexToHash("0x00bf49f440a1cd0527e4d06e2765654c0f56452257516d793a9b8d604dcfdf2b")
		secondKey      = common.HexToHash("0x09e47cd5056a689e708f22fe1f932709a320518e444f5f7d8d46a3da523d6606")
		storageRoot    = common.HexToHash("0xbe3d75a1729be157e79c3b77f00206db4d54e3ea14375a015451c88ec067c790")
	)
	for i, tc := range []accRangeTest{
		// Tests decreasing the number of bytes
		{4000, root, zero, ffHash, 76, firstKey, common.HexToHash("0xd2669dcf3858e7f1eecb8b5fedbf22fbea3e9433848a75035f79d68422c2dcda")},
		{3000, root, zero, ffHash, 57, firstKey, common.HexToHash("0x9b63fa753ece5cb90657d02ecb15df4dc1508d8c1d187af1bf7f1a05e747d3c7")},
		{2000, root, zero, ffHash, 38, firstKey, common.HexToHash("0x5e6140ecae4354a9e8f47559a8c6209c1e0e69cb077b067b528556c11698b91f")},
		{1, root, zero, ffHash, 1, firstKey, firstKey},

		// Tests variations of the range
		//
		// [00b to firstkey]: should return [firstkey, secondkey], where secondkey is out of bounds
		{4000, root, common.HexToHash("0x00bf000000000000000000000000000000000000000000000000000000000000"), common.HexToHash("0x00bf49f440a1cd0527e4d06e2765654c0f56452257516d793a9b8d604dcfdf2b"), 2, firstKey, secondKey},
		// [00b0 to 0bf0]: where both are before firstkey. Should return firstKey (even though it's out of bounds)
		{4000, root, common.HexToHash("0x00b0000000000000000000000000000000000000000000000000000000000000"), common.HexToHash("0x00bf100000000000000000000000000000000000000000000000000000000000"), 1, firstKey, firstKey},
		{4000, root, zero, zero, 1, firstKey, firstKey},
		{4000, root, firstKey, ffHash, 76, firstKey, common.HexToHash("0xd2669dcf3858e7f1eecb8b5fedbf22fbea3e9433848a75035f79d68422c2dcda")},
		{4000, root, firstKeyPlus1, ffHash, 76, secondKey, common.HexToHash("0xd28f55d3b994f16389f36944ad685b48e0fc3f8fbe86c3ca92ebecadf16a783f")},

		// Test different root hashes
		//
		// A stateroot that does not exist
		{4000, common.Hash{0x13, 37}, zero, ffHash, 0, zero, zero},
		// The genesis stateroot (we expect it to not be served)
		{4000, s.chain.RootAt(0), zero, ffHash, 0, zero, zero},
		// A 127 block old stateroot, expected to be served
		{4000, s.chain.RootAt(999 - 127), zero, ffHash, 77, firstKey, common.HexToHash("0xe4c6fdef5dd4e789a2612390806ee840b8ec0fe52548f8b4efe41abb20c37aac")},
		// A root which is not actually an account root, but a storage root
		{4000, storageRoot, zero, ffHash, 0, zero, zero},

		// And some non-sensical requests
		//
		// range from [0xFF to 0x00], wrong order. Expect not to be serviced
		{4000, root, ffHash, zero, 0, zero, zero},
		// range from [firstkey, firstkey-1], wrong order. Expect to get first key.
		{4000, root, firstKey, firstKeyMinus1, 1, firstKey, firstKey},
		// range from [firstkey, 0], wrong order. Expect to get first key.
		{4000, root, firstKey, zero, 1, firstKey, firstKey},
		// Max bytes: 0. Expect to deliver one account.
		{0, root, zero, ffHash, 1, firstKey, firstKey},
	} {
		tc := tc
		if err := s.snapGetAccountRange(t, &tc); err != nil {
			t.Errorf("test %d \n root: %x\n range: %#x - %#x\n bytes: %d\nfailed: %v", i, tc.root, tc.origin, tc.limit, tc.nBytes, err)
		}
	}
}

type stRangesTest struct {
	root     common.Hash
	accounts []common.Hash
	origin   []byte
	limit    []byte
	nBytes   uint64

	expSlots int
}

// TestSnapGetStorageRanges various forms of GetStorageRanges requests.
func (s *Suite) TestSnapGetStorageRanges(t *utesting.T) {
	var (
		ffHash    = common.MaxHash
		zero      = common.Hash{}
		firstKey  = common.HexToHash("0x00bf49f440a1cd0527e4d06e2765654c0f56452257516d793a9b8d604dcfdf2a")
		secondKey = common.HexToHash("0x09e47cd5056a689e708f22fe1f932709a320518e444f5f7d8d46a3da523d6606")
	)
	for i, tc := range []stRangesTest{
		{
			root:     s.chain.RootAt(999),
			accounts: []common.Hash{secondKey, firstKey},
			origin:   zero[:],
			limit:    ffHash[:],
			nBytes:   500,
			expSlots: 0,
		},

		/*
			Some tests against this account:
			{
			  "balance": "0",
			  "nonce": 1,
			  "root": "0xbe3d75a1729be157e79c3b77f00206db4d54e3ea14375a015451c88ec067c790",
			  "codeHash": "0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470",
			  "storage": {
			    "0x405787fa12a823e0f2b7631cc41b3ba8828b3321ca811111fa75cd3aa3bb5ace": "02",
			    "0xb10e2d527612073b26eecdfd717e6a320cf44b4afac2b0732d9fcbe2b7fa0cf6": "01",
			    "0xc2575a0e9e593c00f959f8c92f12db2869c3395a3b0502d05e2516446f71f85b": "03"
			  },
			  "key": "0xf493f79c43bd747129a226ad42529885a4b108aba6046b2d12071695a6627844"
			}
		*/
		{ // [:] -> [slot1, slot2, slot3]
			root:     s.chain.RootAt(999),
			accounts: []common.Hash{common.HexToHash("0xf493f79c43bd747129a226ad42529885a4b108aba6046b2d12071695a6627844")},
			origin:   zero[:],
			limit:    ffHash[:],
			nBytes:   500,
			expSlots: 3,
		},
		{ // [slot1:] -> [slot1, slot2, slot3]
			root:     s.chain.RootAt(999),
			accounts: []common.Hash{common.HexToHash("0xf493f79c43bd747129a226ad42529885a4b108aba6046b2d12071695a6627844")},
			origin:   common.FromHex("0x405787fa12a823e0f2b7631cc41b3ba8828b3321ca811111fa75cd3aa3bb5ace"),
			limit:    ffHash[:],
			nBytes:   500,
			expSlots: 3,
		},
		{ // [slot1+ :] -> [slot2, slot3]
			root:     s.chain.RootAt(999),
			accounts: []common.Hash{common.HexToHash("0xf493f79c43bd747129a226ad42529885a4b108aba6046b2d12071695a6627844")},
			origin:   common.FromHex("0x405787fa12a823e0f2b7631cc41b3ba8828b3321ca811111fa75cd3aa3bb5acf"),
			limit:    ffHash[:],
			nBytes:   500,
			expSlots: 2,
		},
		{ // [slot1:slot2] -> [slot1, slot2]
			root:     s.chain.RootAt(999),
			accounts: []common.Hash{common.HexToHash("0xf493f79c43bd747129a226ad42529885a4b108aba6046b2d12071695a6627844")},
			origin:   common.FromHex("0x405787fa12a823e0f2b7631cc41b3ba8828b3321ca811111fa75cd3aa3bb5ace"),
			limit:    common.FromHex("0xb10e2d527612073b26eecdfd717e6a320cf44b4afac2b0732d9fcbe2b7fa0cf6"),
			nBytes:   500,
			expSlots: 2,
		},
		{ // [slot1+:slot2+] -> [slot2, slot3]
			root:     s.chain.RootAt(999),
			accounts: []common.Hash{common.HexToHash("0xf493f79c43bd747129a226ad42529885a4b108aba6046b2d12071695a6627844")},
			origin:   common.FromHex("0x4fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"),
			limit:    common.FromHex("0xb10e2d527612073b26eecdfd717e6a320cf44b4afac2b0732d9fcbe2b7fa0cf7"),
			nBytes:   500,
			expSlots: 2,
		},
	} {
		tc := tc
		if err := s.snapGetStorageRanges(t, &tc); err != nil {
			t.Errorf("test %d \n root: %x\n range: %#x - %#x\n bytes: %d\n #accounts: %d\nfailed: %v",
				i, tc.root, tc.origin, tc.limit, tc.nBytes, len(tc.accounts), err)
		}
	}
}

type byteCodesTest struct {
	nBytes uint64
	hashes []common.Hash

	expHashes int
}

// TestSnapGetByteCodes various forms of GetByteCodes requests.
func (s *Suite) TestSnapGetByteCodes(t *utesting.T) {
	// The halfchain import should yield these bytecodes
	var hcBytecodes []common.Hash
	for _, s := range []string{
		"0x200c90460d8b0063210d5f5b9918e053c8f2c024485e0f1b48be8b1fc71b1317",
		"0x20ba67ed4ac6aff626e0d1d4db623e2fada9593daeefc4a6eb4b70e6cff986f3",
		"0x24b5b4902cb3d897c1cee9f16be8e897d8fa277c04c6dc8214f18295fca5de44",
		"0x320b9d0a2be39b8a1c858f9f8cb96b1df0983071681de07ded3a7c0d05db5fd6",
		"0x48cb0d5275936a24632babc7408339f9f7b051274809de565b8b0db76e97e03c",
		"0x67c7a6f5cdaa43b4baa0e15b2be63346d1b9ce9f2c3d7e5804e0cacd44ee3b04",
		"0x6d8418059bdc8c3fabf445e6bfc662af3b6a4ae45999b953996e42c7ead2ab49",
		"0x7043422e5795d03f17ee0463a37235258e609fdd542247754895d72695e3e142",
		"0x727f9e6f0c4bac1ff8d72c2972122d9c8d37ccb37e04edde2339e8da193546f1",
		"0x86ccd5e23c78568a8334e0cebaf3e9f48c998307b0bfb1c378cee83b4bfb29cb",
		"0x8fc89b00d6deafd4c4279531e743365626dbfa28845ec697919d305c2674302d",
		"0x92cfc353bcb9746bb6f9996b6b9df779c88af2e9e0eeac44879ca19887c9b732",
		"0x941b4872104f0995a4898fcf0f615ea6bf46bfbdfcf63ea8f2fd45b3f3286b77",
		"0xa02fe8f41159bb39d2b704c633c3d6389cf4bfcb61a2539a9155f60786cf815f",
		"0xa4b94e0afdffcb0af599677709dac067d3145489ea7aede57672bee43e3b7373",
		"0xaf4e64edd3234c1205b725e42963becd1085f013590bd7ed93f8d711c5eb65fb",
		"0xb69a18fa855b742031420081999086f6fb56c3930ae8840944e8b8ae9931c51e",
		"0xc246c217bc73ce6666c93a93a94faa5250564f50a3fdc27ea74c231c07fe2ca6",
		"0xcd6e4ab2c3034df2a8a1dfaaeb1c4baecd162a93d22de35e854ee2945cbe0c35",
		"0xe24b692d09d6fc2f3d1a6028c400a27c37d7cbb11511907c013946d6ce263d3b",
		"0xe440c5f0e8603fd1ed25976eee261ccee8038cf79d6a4c0eb31b2bf883be737f",
		"0xe6eacbc509203d21ac814b350e72934fde686b7f673c19be8cf956b0c70078ce",
		"0xe8530de4371467b5be7ea0e69e675ab36832c426d6c1ce9513817c0f0ae1486b",
		"0xe85d487abbbc83bf3423cf9731360cf4f5a37220e18e5add54e72ee20861196a",
		"0xf195ea389a5eea28db0be93660014275b158963dec44af1dfa7d4743019a9a49",
	} {
		hcBytecodes = append(hcBytecodes, common.HexToHash(s))
	}

	for i, tc := range []byteCodesTest{
		// A few stateroots
		{
			nBytes: 10000, hashes: []common.Hash{s.chain.RootAt(0), s.chain.RootAt(999)},
			expHashes: 0,
		},
		{
			nBytes: 10000, hashes: []common.Hash{s.chain.RootAt(0), s.chain.RootAt(0)},
			expHashes: 0,
		},
		// Empties
		{
			nBytes: 10000, hashes: []common.Hash{types.EmptyRootHash},
			expHashes: 0,
		},
		{
			nBytes: 10000, hashes: []common.Hash{types.EmptyCodeHash},
			expHashes: 1,
		},
		{
			nBytes: 10000, hashes: []common.Hash{types.EmptyCodeHash, types.EmptyCodeHash, types.EmptyCodeHash},
			expHashes: 3,
		},
		// The existing bytecodes
		{
			nBytes: 10000, hashes: hcBytecodes,
			expHashes: len(hcBytecodes),
		},
		// The existing, with limited byte arg
		{
			nBytes: 1, hashes: hcBytecodes,
			expHashes: 1,
		},
		{
			nBytes: 0, hashes: hcBytecodes,
			expHashes: 1,
		},
		{
			nBytes: 1000, hashes: []common.Hash{hcBytecodes[0], hcBytecodes[0], hcBytecodes[0], hcBytecodes[0]},
			expHashes: 4,
		},
	} {
		tc := tc
		if err := s.snapGetByteCodes(t, &tc); err != nil {
			t.Errorf("test %d \n bytes: %d\n #hashes: %d\nfailed: %v", i, tc.nBytes, len(tc.hashes), err)
		}
	}
}

type trieNodesTest struct {
	root   common.Hash
	paths  []snap.TrieNodePathSet
	nBytes uint64

	expHashes []common.Hash
	expReject bool
}

func decodeNibbles(nibbles []byte, bytes []byte) {
	for bi, ni := 0, 0; ni < len(nibbles); bi, ni = bi+1, ni+2 {
		bytes[bi] = nibbles[ni]<<4 | nibbles[ni+1]
	}
}

// hasTerm returns whether a hex key has the terminator flag.
func hasTerm(s []byte) bool {
	return len(s) > 0 && s[len(s)-1] == 16
}

func keybytesToHex(str []byte) []byte {
	l := len(str)*2 + 1
	var nibbles = make([]byte, l)
	for i, b := range str {
		nibbles[i*2] = b / 16
		nibbles[i*2+1] = b % 16
	}
	nibbles[l-1] = 16
	return nibbles
}

func hexToCompact(hex []byte) []byte {
	terminator := byte(0)
	if hasTerm(hex) {
		terminator = 1
		hex = hex[:len(hex)-1]
	}
	buf := make([]byte, len(hex)/2+1)
	buf[0] = terminator << 5 // the flag byte
	if len(hex)&1 == 1 {
		buf[0] |= 1 << 4 // odd flag
		buf[0] |= hex[0] // first nibble is contained in the first byte
		hex = hex[1:]
	}
	decodeNibbles(hex, buf[1:])
	return buf
}

// TestSnapTrieNodes various forms of GetTrieNodes requests.
func (s *Suite) TestSnapTrieNodes(t *utesting.T) {
	key := common.FromHex("0x00bf49f440a1cd0527e4d06e2765654c0f56452257516d793a9b8d604dcfdf2a")
	// helper function to iterate the key, and generate the compact-encoded
	// trie paths along the way.
	pathTo := func(length int) snap.TrieNodePathSet {
		hex := keybytesToHex(key)[:length]
		hex[len(hex)-1] = 0 // remove term flag
		hKey := hexToCompact(hex)
		return snap.TrieNodePathSet{hKey}
	}
	var accPaths []snap.TrieNodePathSet
	for i := 1; i <= 65; i++ {
		accPaths = append(accPaths, pathTo(i))
	}
	empty := types.EmptyCodeHash
	for i, tc := range []trieNodesTest{
		{
			root:      s.chain.RootAt(999),
			paths:     nil,
			nBytes:    500,
			expHashes: nil,
		},
		{
			root: s.chain.RootAt(999),
			paths: []snap.TrieNodePathSet{
				{}, // zero-length pathset should 'abort' and kick us off
				{[]byte{0}},
			},
			nBytes:    5000,
			expHashes: []common.Hash{},
			expReject: true,
		},
		{
			root: s.chain.RootAt(999),
			paths: []snap.TrieNodePathSet{
				{[]byte{0}},
				{[]byte{1}, []byte{0}},
			},
			nBytes: 5000,
			//0x6b3724a41b8c38b46d4d02fba2bb2074c47a507eb16a9a4b978f91d32e406faf
			expHashes: []common.Hash{s.chain.RootAt(999)},
		},
		{ // nonsensically long path
			root: s.chain.RootAt(999),
			paths: []snap.TrieNodePathSet{
				{[]byte{0, 1, 2, 3, 4, 5, 6, 7, 8, 0, 1, 2, 3, 4, 5, 6, 7, 8, 0, 1, 2, 3, 4, 5, 6, 7, 8,
					0, 1, 2, 3, 4, 5, 6, 7, 8, 0, 1, 2, 3, 4, 5, 6, 7, 8, 0, 1, 2, 3, 4, 5, 6, 7, 8}},
			},
			nBytes:    5000,
			expHashes: []common.Hash{common.HexToHash("0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470")},
		},
		{
			root: s.chain.RootAt(0),
			paths: []snap.TrieNodePathSet{
				{[]byte{0}},
				{[]byte{1}, []byte{0}},
			},
			nBytes: 5000,
			expHashes: []common.Hash{
				common.HexToHash("0x1ee1bb2fbac4d46eab331f3e8551e18a0805d084ed54647883aa552809ca968d"),
			},
		},
		{
			// The leaf is only a couple of levels down, so the continued trie traversal causes lookup failures.
			root:   s.chain.RootAt(999),
			paths:  accPaths,
			nBytes: 5000,
			expHashes: []common.Hash{
				common.HexToHash("0xbcefee69b37cca1f5bf3a48aebe08b35f2ea1864fa958bb0723d909a0e0d28d8"),
				common.HexToHash("0x4fb1e4e2391e4b4da471d59641319b8fa25d76c973d4bec594d7b00a69ae5135"),
				empty, empty, empty, empty, empty, empty, empty, empty, empty, empty, empty, empty,
				empty, empty, empty, empty, empty, empty, empty, empty, empty, empty, empty, empty,
				empty, empty, empty, empty, empty, empty, empty, empty, empty, empty, empty, empty,
				empty, empty, empty, empty, empty, empty, empty, empty, empty, empty, empty, empty,
				empty, empty, empty, empty, empty, empty, empty, empty, empty, empty, empty, empty,
				empty, empty, empty},
		},
		{
			// Basically the same as above, with different ordering
			root: s.chain.RootAt(999),
			paths: []snap.TrieNodePathSet{
				accPaths[10], accPaths[1], accPaths[0],
			},
			nBytes: 5000,
			expHashes: []common.Hash{
				empty,
				common.HexToHash("0x4fb1e4e2391e4b4da471d59641319b8fa25d76c973d4bec594d7b00a69ae5135"),
				common.HexToHash("0xbcefee69b37cca1f5bf3a48aebe08b35f2ea1864fa958bb0723d909a0e0d28d8"),
			},
		},
		{
			/*
				A test against this account, requesting trie nodes for the storage trie
				{
				  "balance": "0",
				  "nonce": 1,
				  "root": "0xbe3d75a1729be157e79c3b77f00206db4d54e3ea14375a015451c88ec067c790",
				  "codeHash": "0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470",
				  "storage": {
				    "0x405787fa12a823e0f2b7631cc41b3ba8828b3321ca811111fa75cd3aa3bb5ace": "02",
				    "0xb10e2d527612073b26eecdfd717e6a320cf44b4afac2b0732d9fcbe2b7fa0cf6": "01",
				    "0xc2575a0e9e593c00f959f8c92f12db2869c3395a3b0502d05e2516446f71f85b": "03"
				  },
				  "key": "0xf493f79c43bd747129a226ad42529885a4b108aba6046b2d12071695a6627844"
				}
			*/
			root: s.chain.RootAt(999),
			paths: []snap.TrieNodePathSet{
				{
					common.FromHex("0xf493f79c43bd747129a226ad42529885a4b108aba6046b2d12071695a6627844"),
					[]byte{0},
				},
			},
			nBytes: 5000,
			expHashes: []common.Hash{
				common.HexToHash("0xbe3d75a1729be157e79c3b77f00206db4d54e3ea14375a015451c88ec067c790"),
			},
		},
	}[7:] {
		tc := tc
		if err := s.snapGetTrieNodes(t, &tc); err != nil {
			t.Errorf("test %d \n #hashes %x\n root: %#x\n bytes: %d\nfailed: %v", i, len(tc.expHashes), tc.root, tc.nBytes, err)
		}
	}
}

func (s *Suite) snapGetAccountRange(t *utesting.T, tc *accRangeTest) error {
	conn, err := s.dialSnap()
	if err != nil {
		t.Fatalf("dial failed: %v", err)
	}
	defer conn.Close()
	if err = conn.peer(s.chain, nil); err != nil {
		t.Fatalf("peering failed: %v", err)
	}
	// write request
	req := &GetAccountRange{
		ID:     uint64(rand.Int63()),
		Root:   tc.root,
		Origin: tc.origin,
		Limit:  tc.limit,
		Bytes:  tc.nBytes,
	}
	resp, err := conn.snapRequest(req, req.ID, s.chain)
	if err != nil {
		return fmt.Errorf("account range request failed: %v", err)
	}
	var res *snap.AccountRangePacket
	if r, ok := resp.(*AccountRange); !ok {
		return fmt.Errorf("account range response wrong: %T %v", resp, resp)
	} else {
		res = (*snap.AccountRangePacket)(r)
	}
	if exp, got := tc.expAccounts, len(res.Accounts); exp != got {
		return fmt.Errorf("expected %d accounts, got %d", exp, got)
	}
	// Check that the encoding order is correct
	for i := 1; i < len(res.Accounts); i++ {
		if bytes.Compare(res.Accounts[i-1].Hash[:], res.Accounts[i].Hash[:]) >= 0 {
			return fmt.Errorf("accounts not monotonically increasing: #%d [%x] vs #%d [%x]", i-1, res.Accounts[i-1].Hash[:], i, res.Accounts[i].Hash[:])
		}
	}
	var (
		hashes   []common.Hash
		accounts [][]byte
		proof    = res.Proof
	)
	hashes, accounts, err = res.Unpack()
	if err != nil {
		return err
	}
	if len(hashes) == 0 && len(accounts) == 0 && len(proof) == 0 {
		return nil
	}
	if len(hashes) > 0 {
		if exp, got := tc.expFirst, res.Accounts[0].Hash; exp != got {
			return fmt.Errorf("expected first account %#x, got %#x", exp, got)
		}
		if exp, got := tc.expLast, res.Accounts[len(res.Accounts)-1].Hash; exp != got {
			return fmt.Errorf("expected last account %#x, got %#x", exp, got)
		}
	}
	// Reconstruct a partial trie from the response and verify it
	keys := make([][]byte, len(hashes))
	for i, key := range hashes {
		keys[i] = common.CopyBytes(key[:])
	}
	nodes := make(trienode.ProofList, len(proof))
	for i, node := range proof {
		nodes[i] = node
	}
	proofdb := nodes.Set()

	_, err = trie.VerifyRangeProof(tc.root, tc.origin[:], keys, accounts, proofdb)
	return err
}

func (s *Suite) snapGetStorageRanges(t *utesting.T, tc *stRangesTest) error {
	conn, err := s.dialSnap()
	if err != nil {
		t.Fatalf("dial failed: %v", err)
	}
	defer conn.Close()
	if err = conn.peer(s.chain, nil); err != nil {
		t.Fatalf("peering failed: %v", err)
	}
	// write request
	req := &GetStorageRanges{
		ID:       uint64(rand.Int63()),
		Root:     tc.root,
		Accounts: tc.accounts,
		Origin:   tc.origin,
		Limit:    tc.limit,
		Bytes:    tc.nBytes,
	}
	resp, err := conn.snapRequest(req, req.ID, s.chain)
	if err != nil {
		return fmt.Errorf("account range request failed: %v", err)
	}
	var res *snap.StorageRangesPacket
	if r, ok := resp.(*StorageRanges); !ok {
		return fmt.Errorf("account range response wrong: %T %v", resp, resp)
	} else {
		res = (*snap.StorageRangesPacket)(r)
	}
	gotSlots := 0
	// Ensure the ranges are monotonically increasing
	for i, slots := range res.Slots {
		gotSlots += len(slots)
		for j := 1; j < len(slots); j++ {
			if bytes.Compare(slots[j-1].Hash[:], slots[j].Hash[:]) >= 0 {
				return fmt.Errorf("storage slots not monotonically increasing for account #%d: #%d [%x] vs #%d [%x]", i, j-1, slots[j-1].Hash[:], j, slots[j].Hash[:])
			}
		}
	}
	if exp, got := tc.expSlots, gotSlots; exp != got {
		return fmt.Errorf("expected %d slots, got %d", exp, got)
	}
	return nil
}

func (s *Suite) snapGetByteCodes(t *utesting.T, tc *byteCodesTest) error {
	conn, err := s.dialSnap()
	if err != nil {
		t.Fatalf("dial failed: %v", err)
	}
	defer conn.Close()
	if err = conn.peer(s.chain, nil); err != nil {
		t.Fatalf("peering failed: %v", err)
	}
	// write request
	req := &GetByteCodes{
		ID:     uint64(rand.Int63()),
		Hashes: tc.hashes,
		Bytes:  tc.nBytes,
	}
	resp, err := conn.snapRequest(req, req.ID, s.chain)
	if err != nil {
		return fmt.Errorf("getBytecodes request failed: %v", err)
	}
	var res *snap.ByteCodesPacket
	if r, ok := resp.(*ByteCodes); !ok {
		return fmt.Errorf("bytecodes response wrong: %T %v", resp, resp)
	} else {
		res = (*snap.ByteCodesPacket)(r)
	}
	if exp, got := tc.expHashes, len(res.Codes); exp != got {
		for i, c := range res.Codes {
			fmt.Printf("%d. %#x\n", i, c)
		}
		return fmt.Errorf("expected %d bytecodes, got %d", exp, got)
	}
	// Cross reference the requested bytecodes with the response to find gaps
	// that the serving node is missing
	var (
		bytecodes = res.Codes
		hasher    = sha3.NewLegacyKeccak256().(crypto.KeccakState)
		hash      = make([]byte, 32)
		codes     = make([][]byte, len(req.Hashes))
	)

	for i, j := 0, 0; i < len(bytecodes); i++ {
		// Find the next hash that we've been served, leaving misses with nils
		hasher.Reset()
		hasher.Write(bytecodes[i])
		hasher.Read(hash)

		for j < len(req.Hashes) && !bytes.Equal(hash, req.Hashes[j][:]) {
			j++
		}
		if j < len(req.Hashes) {
			codes[j] = bytecodes[i]
			j++
			continue
		}
		// We've either ran out of hashes, or got unrequested data
		return errors.New("unexpected bytecode")
	}

	return nil
}

func (s *Suite) snapGetTrieNodes(t *utesting.T, tc *trieNodesTest) error {
	conn, err := s.dialSnap()
	if err != nil {
		t.Fatalf("dial failed: %v", err)
	}
	defer conn.Close()
	if err = conn.peer(s.chain, nil); err != nil {
		t.Fatalf("peering failed: %v", err)
	}
	// write request
	req := &GetTrieNodes{
		ID:    uint64(rand.Int63()),
		Root:  tc.root,
		Paths: tc.paths,
		Bytes: tc.nBytes,
	}
	resp, err := conn.snapRequest(req, req.ID, s.chain)
	if err != nil {
		if tc.expReject {
			return nil
		}
		return fmt.Errorf("trienodes  request failed: %v", err)
	}
	var res *snap.TrieNodesPacket
	if r, ok := resp.(*TrieNodes); !ok {
		return fmt.Errorf("trienodes response wrong: %T %v", resp, resp)
	} else {
		res = (*snap.TrieNodesPacket)(r)
	}

	// Check the correctness

	// Cross reference the requested trienodes with the response to find gaps
	// that the serving node is missing
	hasher := sha3.NewLegacyKeccak256().(crypto.KeccakState)
	hash := make([]byte, 32)
	trienodes := res.Nodes
	if got, want := len(trienodes), len(tc.expHashes); got != want {
		return fmt.Errorf("wrong trienode count, got %d, want %d\n", got, want)
	}
	for i, trienode := range trienodes {
		hasher.Reset()
		hasher.Write(trienode)
		hasher.Read(hash)
		if got, want := hash, tc.expHashes[i]; !bytes.Equal(got, want[:]) {
			fmt.Printf("hash %d wrong, got %#x, want %#x\n", i, got, want)
			err = fmt.Errorf("hash %d wrong, got %#x, want %#x", i, got, want)
		}
	}
	return err
}
