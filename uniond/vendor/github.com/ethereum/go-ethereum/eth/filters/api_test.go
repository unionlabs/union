// Copyright 2016 The go-ethereum Authors
// This file is part of the go-ethereum library.
//
// The go-ethereum library is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// The go-ethereum library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with the go-ethereum library. If not, see <http://www.gnu.org/licenses/>.

package filters

import (
	"encoding/json"
	"fmt"
	"testing"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/rpc"
)

func TestUnmarshalJSONNewFilterArgs(t *testing.T) {
	var (
		fromBlock rpc.BlockNumber = 0x123435
		toBlock   rpc.BlockNumber = 0xabcdef
		address0                  = common.HexToAddress("70c87d191324e6712a591f304b4eedef6ad9bb9d")
		address1                  = common.HexToAddress("9b2055d370f73ec7d8a03e965129118dc8f5bf83")
		topic0                    = common.HexToHash("3ac225168df54212a25c1c01fd35bebfea408fdac2e31ddd6f80a4bbf9a5f1ca")
		topic1                    = common.HexToHash("9084a792d2f8b16a62b882fd56f7860c07bf5fa91dd8a2ae7e809e5180fef0b3")
		topic2                    = common.HexToHash("6ccae1c4af4152f460ff510e573399795dfab5dcf1fa60d1f33ac8fdc1e480ce")
	)

	// default values
	var test0 FilterCriteria
	if err := json.Unmarshal([]byte("{}"), &test0); err != nil {
		t.Fatal(err)
	}
	if test0.FromBlock != nil {
		t.Fatalf("expected nil, got %d", test0.FromBlock)
	}
	if test0.ToBlock != nil {
		t.Fatalf("expected nil, got %d", test0.ToBlock)
	}
	if len(test0.Addresses) != 0 {
		t.Fatalf("expected 0 addresses, got %d", len(test0.Addresses))
	}
	if len(test0.Topics) != 0 {
		t.Fatalf("expected 0 topics, got %d topics", len(test0.Topics))
	}

	// from, to block number
	var test1 FilterCriteria
	vector := fmt.Sprintf(`{"fromBlock":"%v","toBlock":"%v"}`, fromBlock, toBlock)
	if err := json.Unmarshal([]byte(vector), &test1); err != nil {
		t.Fatal(err)
	}
	if test1.FromBlock.Int64() != fromBlock.Int64() {
		t.Fatalf("expected FromBlock %d, got %d", fromBlock, test1.FromBlock)
	}
	if test1.ToBlock.Int64() != toBlock.Int64() {
		t.Fatalf("expected ToBlock %d, got %d", toBlock, test1.ToBlock)
	}

	// single address
	var test2 FilterCriteria
	vector = fmt.Sprintf(`{"address": "%s"}`, address0.Hex())
	if err := json.Unmarshal([]byte(vector), &test2); err != nil {
		t.Fatal(err)
	}
	if len(test2.Addresses) != 1 {
		t.Fatalf("expected 1 address, got %d address(es)", len(test2.Addresses))
	}
	if test2.Addresses[0] != address0 {
		t.Fatalf("expected address %x, got %x", address0, test2.Addresses[0])
	}

	// multiple address
	var test3 FilterCriteria
	vector = fmt.Sprintf(`{"address": ["%s", "%s"]}`, address0.Hex(), address1.Hex())
	if err := json.Unmarshal([]byte(vector), &test3); err != nil {
		t.Fatal(err)
	}
	if len(test3.Addresses) != 2 {
		t.Fatalf("expected 2 addresses, got %d address(es)", len(test3.Addresses))
	}
	if test3.Addresses[0] != address0 {
		t.Fatalf("expected address %x, got %x", address0, test3.Addresses[0])
	}
	if test3.Addresses[1] != address1 {
		t.Fatalf("expected address %x, got %x", address1, test3.Addresses[1])
	}

	// single topic
	var test4 FilterCriteria
	vector = fmt.Sprintf(`{"topics": ["%s"]}`, topic0.Hex())
	if err := json.Unmarshal([]byte(vector), &test4); err != nil {
		t.Fatal(err)
	}
	if len(test4.Topics) != 1 {
		t.Fatalf("expected 1 topic, got %d", len(test4.Topics))
	}
	if len(test4.Topics[0]) != 1 {
		t.Fatalf("expected len(topics[0]) to be 1, got %d", len(test4.Topics[0]))
	}
	if test4.Topics[0][0] != topic0 {
		t.Fatalf("got %x, expected %x", test4.Topics[0][0], topic0)
	}

	// test multiple "AND" topics
	var test5 FilterCriteria
	vector = fmt.Sprintf(`{"topics": ["%s", "%s"]}`, topic0.Hex(), topic1.Hex())
	if err := json.Unmarshal([]byte(vector), &test5); err != nil {
		t.Fatal(err)
	}
	if len(test5.Topics) != 2 {
		t.Fatalf("expected 2 topics, got %d", len(test5.Topics))
	}
	if len(test5.Topics[0]) != 1 {
		t.Fatalf("expected 1 topic, got %d", len(test5.Topics[0]))
	}
	if test5.Topics[0][0] != topic0 {
		t.Fatalf("got %x, expected %x", test5.Topics[0][0], topic0)
	}
	if len(test5.Topics[1]) != 1 {
		t.Fatalf("expected 1 topic, got %d", len(test5.Topics[1]))
	}
	if test5.Topics[1][0] != topic1 {
		t.Fatalf("got %x, expected %x", test5.Topics[1][0], topic1)
	}

	// test optional topic
	var test6 FilterCriteria
	vector = fmt.Sprintf(`{"topics": ["%s", null, "%s"]}`, topic0.Hex(), topic2.Hex())
	if err := json.Unmarshal([]byte(vector), &test6); err != nil {
		t.Fatal(err)
	}
	if len(test6.Topics) != 3 {
		t.Fatalf("expected 3 topics, got %d", len(test6.Topics))
	}
	if len(test6.Topics[0]) != 1 {
		t.Fatalf("expected 1 topic, got %d", len(test6.Topics[0]))
	}
	if test6.Topics[0][0] != topic0 {
		t.Fatalf("got %x, expected %x", test6.Topics[0][0], topic0)
	}
	if len(test6.Topics[1]) != 0 {
		t.Fatalf("expected 0 topic, got %d", len(test6.Topics[1]))
	}
	if len(test6.Topics[2]) != 1 {
		t.Fatalf("expected 1 topic, got %d", len(test6.Topics[2]))
	}
	if test6.Topics[2][0] != topic2 {
		t.Fatalf("got %x, expected %x", test6.Topics[2][0], topic2)
	}

	// test OR topics
	var test7 FilterCriteria
	vector = fmt.Sprintf(`{"topics": [["%s", "%s"], null, ["%s", null]]}`, topic0.Hex(), topic1.Hex(), topic2.Hex())
	if err := json.Unmarshal([]byte(vector), &test7); err != nil {
		t.Fatal(err)
	}
	if len(test7.Topics) != 3 {
		t.Fatalf("expected 3 topics, got %d topics", len(test7.Topics))
	}
	if len(test7.Topics[0]) != 2 {
		t.Fatalf("expected 2 topics, got %d topics", len(test7.Topics[0]))
	}
	if test7.Topics[0][0] != topic0 || test7.Topics[0][1] != topic1 {
		t.Fatalf("invalid topics expected [%x,%x], got [%x,%x]",
			topic0, topic1, test7.Topics[0][0], test7.Topics[0][1],
		)
	}
	if len(test7.Topics[1]) != 0 {
		t.Fatalf("expected 0 topic, got %d topics", len(test7.Topics[1]))
	}
	if len(test7.Topics[2]) != 0 {
		t.Fatalf("expected 0 topics, got %d topics", len(test7.Topics[2]))
	}
}
