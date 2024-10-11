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

package errbase_test

import (
	"context"
	goErr "errors"
	"fmt"
	"testing"

	"github.com/cockroachdb/errors/errbase"
	"github.com/cockroachdb/errors/errorspb"
	"github.com/cockroachdb/errors/testutils"
)

func genEncoded(mt errorspb.MessageType) errorspb.EncodedError {
	return errorspb.EncodedError{
		Error: &errorspb.EncodedError_Wrapper{
			Wrapper: &errorspb.EncodedWrapper{
				Cause: errorspb.EncodedError{
					Error: &errorspb.EncodedError_Leaf{
						Leaf: &errorspb.EncodedErrorLeaf{
							Message: "leaf-error-msg",
						},
					},
				},
				Message:     "wrapper-error-msg: leaf-error-msg: extra info",
				Details:     errorspb.EncodedErrorDetails{},
				MessageType: mt,
			},
		},
	}
}

func TestDecodeOldVersion(t *testing.T) {
	tt := testutils.T{T: t}

	errOldEncoded := genEncoded(errorspb.MessageType_PREFIX)
	errOldDecoded := errbase.DecodeError(context.Background(), errOldEncoded)
	// Ensure that we will continue to just concat leaf after wrapper
	// with older errors for backward compatibility.
	tt.CheckEqual(errOldDecoded.Error(), "wrapper-error-msg: leaf-error-msg: extra info: leaf-error-msg")

	// Check to ensure that when flag is present, we interpret things correctly.
	errNewEncoded := genEncoded(errorspb.MessageType_FULL_MESSAGE)
	errNewDecoded := errbase.DecodeError(context.Background(), errNewEncoded)
	tt.CheckEqual(errNewDecoded.Error(), "wrapper-error-msg: leaf-error-msg: extra info")
}

func TestEncodeDecodeNewVersion(t *testing.T) {
	tt := testutils.T{T: t}
	errNewEncoded := errbase.EncodeError(
		context.Background(),
		fmt.Errorf(
			"wrapper-error-msg: %w: extra info",
			goErr.New("leaf-error-msg"),
		),
	)

	errNew := errorspb.EncodedError{
		Error: &errorspb.EncodedError_Wrapper{
			Wrapper: &errorspb.EncodedWrapper{
				Cause: errorspb.EncodedError{
					Error: &errorspb.EncodedError_Leaf{
						Leaf: &errorspb.EncodedErrorLeaf{
							Message: "leaf-error-msg",
							Details: errorspb.EncodedErrorDetails{
								OriginalTypeName:  "errors/*errors.errorString",
								ErrorTypeMark:     errorspb.ErrorTypeMark{FamilyName: "errors/*errors.errorString", Extension: ""},
								ReportablePayload: nil,
								FullDetails:       nil,
							},
						},
					},
				},
				Message: "wrapper-error-msg: leaf-error-msg: extra info",
				Details: errorspb.EncodedErrorDetails{
					OriginalTypeName:  "fmt/*fmt.wrapError",
					ErrorTypeMark:     errorspb.ErrorTypeMark{FamilyName: "fmt/*fmt.wrapError", Extension: ""},
					ReportablePayload: nil,
					FullDetails:       nil,
				},
				MessageType: errorspb.MessageType_FULL_MESSAGE,
			},
		},
	}

	tt.CheckDeepEqual(errNewEncoded, errNew)
	newErr := errbase.DecodeError(context.Background(), errNew)

	// New version correctly decodes error
	tt.CheckEqual(newErr.Error(), "wrapper-error-msg: leaf-error-msg: extra info")
}
