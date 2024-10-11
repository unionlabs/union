// Copyright 2023, Google Inc.
// All rights reserved.
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are
// met:
//
//     * Redistributions of source code must retain the above copyright
// notice, this list of conditions and the following disclaimer.
//     * Redistributions in binary form must reproduce the above
// copyright notice, this list of conditions and the following disclaimer
// in the documentation and/or other materials provided with the
// distribution.
//     * Neither the name of Google Inc. nor the names of its
// contributors may be used to endorse or promote products derived from
// this software without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
// "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
// LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
// A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
// OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
// LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
// DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
// THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
// (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

package callctx_test

import (
	"context"
	"fmt"

	"github.com/googleapis/gax-go/v2/callctx"
	"google.golang.org/genproto/googleapis/api/metric"
	"google.golang.org/protobuf/types/known/fieldmaskpb"
)

func ExampleSetHeaders() {
	ctx := context.Background()
	ctx = callctx.SetHeaders(ctx, "key", "value")

	// Send the returned context to the request you are making. Later on these
	// values will be retrieved and set on outgoing requests.

	headers := callctx.HeadersFromContext(ctx)
	fmt.Println(headers["key"][0])
	// Output: value
}

func ExampleXGoogFieldMaskHeader() {
	ctx := context.Background()
	ctx = callctx.SetHeaders(ctx, callctx.XGoogFieldMaskHeader, "field_one,field.two")

	// Send the returned context to the request you are making.
}

func ExampleXGoogFieldMaskHeader_fieldmaskpb() {
	// Build a mask using the expected response protobuf message.
	mask, err := fieldmaskpb.New(&metric.MetricDescriptor{}, "display_name", "metadata.launch_stage")
	if err != nil {
		// handle error
	}

	ctx := context.Background()
	ctx = callctx.SetHeaders(ctx, callctx.XGoogFieldMaskHeader, mask.String())

	// Send the returned context to the request you are making.
}
