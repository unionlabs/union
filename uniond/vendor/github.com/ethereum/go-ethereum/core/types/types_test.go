// Copyright 2021 The go-ethereum Authors
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
	"math/big"
	"testing"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/crypto"
	"github.com/ethereum/go-ethereum/rlp"
)

type devnull struct{ len int }

func (d *devnull) Write(p []byte) (int, error) {
	d.len += len(p)
	return len(p), nil
}

func BenchmarkEncodeRLP(b *testing.B) {
	benchRLP(b, true)
}

func BenchmarkDecodeRLP(b *testing.B) {
	benchRLP(b, false)
}

func benchRLP(b *testing.B, encode bool) {
	key, _ := crypto.HexToECDSA("b71c71a67e1177ad4e901695e1b4b9ee17ae16c6668d313eac2f96dbcda3f291")
	to := common.HexToAddress("0x00000000000000000000000000000000deadbeef")
	signer := NewLondonSigner(big.NewInt(1337))
	for _, tc := range []struct {
		name string
		obj  interface{}
	}{
		{
			"legacy-header",
			&Header{
				Difficulty: big.NewInt(10000000000),
				Number:     big.NewInt(1000),
				GasLimit:   8_000_000,
				GasUsed:    8_000_000,
				Time:       555,
				Extra:      make([]byte, 32),
			},
		},
		{
			"london-header",
			&Header{
				Difficulty: big.NewInt(10000000000),
				Number:     big.NewInt(1000),
				GasLimit:   8_000_000,
				GasUsed:    8_000_000,
				Time:       555,
				Extra:      make([]byte, 32),
				BaseFee:    big.NewInt(10000000000),
			},
		},
		{
			"receipt-for-storage",
			&ReceiptForStorage{
				Status:            ReceiptStatusSuccessful,
				CumulativeGasUsed: 0x888888888,
				Logs:              make([]*Log, 0),
			},
		},
		{
			"receipt-full",
			&Receipt{
				Status:            ReceiptStatusSuccessful,
				CumulativeGasUsed: 0x888888888,
				Logs:              make([]*Log, 0),
			},
		},
		{
			"legacy-transaction",
			MustSignNewTx(key, signer,
				&LegacyTx{
					Nonce:    1,
					GasPrice: big.NewInt(500),
					Gas:      1000000,
					To:       &to,
					Value:    big.NewInt(1),
				}),
		},
		{
			"access-transaction",
			MustSignNewTx(key, signer,
				&AccessListTx{
					Nonce:    1,
					GasPrice: big.NewInt(500),
					Gas:      1000000,
					To:       &to,
					Value:    big.NewInt(1),
				}),
		},
		{
			"1559-transaction",
			MustSignNewTx(key, signer,
				&DynamicFeeTx{
					Nonce:     1,
					Gas:       1000000,
					To:        &to,
					Value:     big.NewInt(1),
					GasTipCap: big.NewInt(500),
					GasFeeCap: big.NewInt(500),
				}),
		},
	} {
		if encode {
			b.Run(tc.name, func(b *testing.B) {
				b.ReportAllocs()
				var null = &devnull{}
				for i := 0; i < b.N; i++ {
					rlp.Encode(null, tc.obj)
				}
				b.SetBytes(int64(null.len / b.N))
			})
		} else {
			data, _ := rlp.EncodeToBytes(tc.obj)
			// Test decoding
			b.Run(tc.name, func(b *testing.B) {
				b.ReportAllocs()
				for i := 0; i < b.N; i++ {
					if err := rlp.DecodeBytes(data, tc.obj); err != nil {
						b.Fatal(err)
					}
				}
				b.SetBytes(int64(len(data)))
			})
		}
	}
}
