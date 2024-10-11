// Copyright 2017 Tendermint. All Rights Reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

package amino_test

import (
	"fmt"

	"github.com/tendermint/go-amino"
)

func Example() {

	defer func() {
		if e := recover(); e != nil {
			fmt.Println("Recovered:", e)
		}
	}()

	type Message interface{}

	type bcMessage struct {
		Message string
		Height  int
	}

	type bcResponse struct {
		Status  int
		Message string
	}

	type bcStatus struct {
		Peers int
	}

	var cdc = amino.NewCodec()
	cdc.RegisterInterface((*Message)(nil), nil)
	cdc.RegisterConcrete(&bcMessage{}, "bcMessage", nil)
	cdc.RegisterConcrete(&bcResponse{}, "bcResponse", nil)
	cdc.RegisterConcrete(&bcStatus{}, "bcStatus", nil)

	var bm = &bcMessage{Message: "ABC", Height: 100}
	var msg = bm

	var bz []byte // the marshalled bytes.
	var err error
	bz, err = cdc.MarshalBinaryLengthPrefixed(msg)
	fmt.Printf("Encoded: %X (err: %v)\n", bz, err)

	var msg2 Message
	err = cdc.UnmarshalBinaryLengthPrefixed(bz, &msg2)
	fmt.Printf("Decoded: %v (err: %v)\n", msg2, err)
	var bm2 = msg2.(*bcMessage)
	fmt.Printf("Decoded successfully: %v\n", *bm == *bm2)

	// Output:
	// Encoded: 0B740613650A034142431064 (err: <nil>)
	// Decoded: &{ABC 100} (err: <nil>)
	// Decoded successfully: true
}
