// Copyright 2023 The Cockroach Authors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or
// implied. See the License for the specific language governing
// permissions and limitations under the License.

package errbase

import (
	"context"
	"errors"
	"fmt"
	"testing"

	"github.com/cockroachdb/errors/testutils"
	"github.com/kr/pretty"
)

func TestUnknownWrapperTraversalWithMessageOverride(t *testing.T) {
	// Simulating scenario where the new field on the opaque wrapper is dropped
	// in the middle of the chain by a node running an older version.

	origErr := fmt.Errorf("this is a wrapped err %w with a non-prefix wrap msg", errors.New("hello"))
	t.Logf("start err: %# v", pretty.Formatter(origErr))

	// Encode the error, this will use the encoder.
	enc := EncodeError(context.Background(), origErr)
	t.Logf("encoded: %# v", pretty.Formatter(enc))

	newErr := DecodeError(context.Background(), enc)
	t.Logf("decoded: %# v", pretty.Formatter(newErr))

	// simulate node not knowing about `messageType` field
	newErr.(*opaqueWrapper).messageType = Prefix

	// Encode it again, to simulate the error passed on to another system.
	enc2 := EncodeError(context.Background(), newErr)
	t.Logf("encoded2: %# v", pretty.Formatter(enc))

	// Then decode again.
	newErr2 := DecodeError(context.Background(), enc2)
	t.Logf("decoded: %# v", pretty.Formatter(newErr2))

	tt := testutils.T{T: t}

	// We expect to see an erroneous `: hello` because our
	// error passes through a node which drops the new protobuf
	// field.
	tt.CheckEqual(newErr2.Error(), origErr.Error()+": hello")
}
