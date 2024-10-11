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

package types

import (
	"encoding/json"
	"errors"
	"reflect"
	"testing"

	"github.com/davecgh/go-spew/spew"
	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/common/hexutil"
)

var unmarshalLogTests = map[string]struct {
	input     string
	want      *Log
	wantError error
}{
	"ok": {
		input: `{"address":"0xecf8f87f810ecf450940c9f60066b4a7a501d6a7","blockHash":"0x656c34545f90a730a19008c0e7a7cd4fb3895064b48d6d69761bd5abad681056","blockNumber":"0x1ecfa4","data":"0x000000000000000000000000000000000000000000000001a055690d9db80000","logIndex":"0x2","topics":["0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef","0x00000000000000000000000080b2c9d7cbbf30a1b0fc8983c647d754c6525615"],"transactionHash":"0x3b198bfd5d2907285af009e9ae84a0ecd63677110d89d7e030251acb87f6487e","transactionIndex":"0x3"}`,
		want: &Log{
			Address:     common.HexToAddress("0xecf8f87f810ecf450940c9f60066b4a7a501d6a7"),
			BlockHash:   common.HexToHash("0x656c34545f90a730a19008c0e7a7cd4fb3895064b48d6d69761bd5abad681056"),
			BlockNumber: 2019236,
			Data:        hexutil.MustDecode("0x000000000000000000000000000000000000000000000001a055690d9db80000"),
			Index:       2,
			TxIndex:     3,
			TxHash:      common.HexToHash("0x3b198bfd5d2907285af009e9ae84a0ecd63677110d89d7e030251acb87f6487e"),
			Topics: []common.Hash{
				common.HexToHash("0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef"),
				common.HexToHash("0x00000000000000000000000080b2c9d7cbbf30a1b0fc8983c647d754c6525615"),
			},
		},
	},
	"empty data": {
		input: `{"address":"0xecf8f87f810ecf450940c9f60066b4a7a501d6a7","blockHash":"0x656c34545f90a730a19008c0e7a7cd4fb3895064b48d6d69761bd5abad681056","blockNumber":"0x1ecfa4","data":"0x","logIndex":"0x2","topics":["0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef","0x00000000000000000000000080b2c9d7cbbf30a1b0fc8983c647d754c6525615"],"transactionHash":"0x3b198bfd5d2907285af009e9ae84a0ecd63677110d89d7e030251acb87f6487e","transactionIndex":"0x3"}`,
		want: &Log{
			Address:     common.HexToAddress("0xecf8f87f810ecf450940c9f60066b4a7a501d6a7"),
			BlockHash:   common.HexToHash("0x656c34545f90a730a19008c0e7a7cd4fb3895064b48d6d69761bd5abad681056"),
			BlockNumber: 2019236,
			Data:        []byte{},
			Index:       2,
			TxIndex:     3,
			TxHash:      common.HexToHash("0x3b198bfd5d2907285af009e9ae84a0ecd63677110d89d7e030251acb87f6487e"),
			Topics: []common.Hash{
				common.HexToHash("0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef"),
				common.HexToHash("0x00000000000000000000000080b2c9d7cbbf30a1b0fc8983c647d754c6525615"),
			},
		},
	},
	"missing block fields (pending logs)": {
		input: `{"address":"0xecf8f87f810ecf450940c9f60066b4a7a501d6a7","data":"0x","logIndex":"0x0","topics":["0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef"],"transactionHash":"0x3b198bfd5d2907285af009e9ae84a0ecd63677110d89d7e030251acb87f6487e","transactionIndex":"0x3"}`,
		want: &Log{
			Address:     common.HexToAddress("0xecf8f87f810ecf450940c9f60066b4a7a501d6a7"),
			BlockHash:   common.Hash{},
			BlockNumber: 0,
			Data:        []byte{},
			Index:       0,
			TxIndex:     3,
			TxHash:      common.HexToHash("0x3b198bfd5d2907285af009e9ae84a0ecd63677110d89d7e030251acb87f6487e"),
			Topics: []common.Hash{
				common.HexToHash("0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef"),
			},
		},
	},
	"Removed: true": {
		input: `{"address":"0xecf8f87f810ecf450940c9f60066b4a7a501d6a7","blockHash":"0x656c34545f90a730a19008c0e7a7cd4fb3895064b48d6d69761bd5abad681056","blockNumber":"0x1ecfa4","data":"0x","logIndex":"0x2","topics":["0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef"],"transactionHash":"0x3b198bfd5d2907285af009e9ae84a0ecd63677110d89d7e030251acb87f6487e","transactionIndex":"0x3","removed":true}`,
		want: &Log{
			Address:     common.HexToAddress("0xecf8f87f810ecf450940c9f60066b4a7a501d6a7"),
			BlockHash:   common.HexToHash("0x656c34545f90a730a19008c0e7a7cd4fb3895064b48d6d69761bd5abad681056"),
			BlockNumber: 2019236,
			Data:        []byte{},
			Index:       2,
			TxIndex:     3,
			TxHash:      common.HexToHash("0x3b198bfd5d2907285af009e9ae84a0ecd63677110d89d7e030251acb87f6487e"),
			Topics: []common.Hash{
				common.HexToHash("0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef"),
			},
			Removed: true,
		},
	},
	"missing data": {
		input:     `{"address":"0xecf8f87f810ecf450940c9f60066b4a7a501d6a7","blockHash":"0x656c34545f90a730a19008c0e7a7cd4fb3895064b48d6d69761bd5abad681056","blockNumber":"0x1ecfa4","logIndex":"0x2","topics":["0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef","0x00000000000000000000000080b2c9d7cbbf30a1b0fc8983c647d754c6525615","0x000000000000000000000000f9dff387dcb5cc4cca5b91adb07a95f54e9f1bb6"],"transactionHash":"0x3b198bfd5d2907285af009e9ae84a0ecd63677110d89d7e030251acb87f6487e","transactionIndex":"0x3"}`,
		wantError: errors.New("missing required field 'data' for Log"),
	},
}

func TestUnmarshalLog(t *testing.T) {
	dumper := spew.ConfigState{DisableMethods: true, Indent: "    "}
	for name, test := range unmarshalLogTests {
		var log *Log
		err := json.Unmarshal([]byte(test.input), &log)
		checkError(t, name, err, test.wantError)
		if test.wantError == nil && err == nil {
			if !reflect.DeepEqual(log, test.want) {
				t.Errorf("test %q:\nGOT %sWANT %s", name, dumper.Sdump(log), dumper.Sdump(test.want))
			}
		}
	}
}

func checkError(t *testing.T, testname string, got, want error) bool {
	if got == nil {
		if want != nil {
			t.Errorf("test %q: got no error, want %q", testname, want)
			return false
		}
		return true
	}
	if want == nil {
		t.Errorf("test %q: unexpected error %q", testname, got)
	} else if got.Error() != want.Error() {
		t.Errorf("test %q: got error %q, want %q", testname, got, want)
	}
	return false
}
