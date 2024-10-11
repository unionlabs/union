// Copyright 2022 Google LLC.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package gensupport

import (
	"errors"
	"testing"

	"github.com/google/go-cmp/cmp"
	"github.com/googleapis/gax-go/v2/apierror"
	"google.golang.org/api/googleapi"
	"google.golang.org/genproto/googleapis/rpc/errdetails"
	"google.golang.org/protobuf/proto"
)

func TestWrapError(t *testing.T) {
	// The error format v2 for Google JSON REST APIs, per https://cloud.google.com/apis/design/errors#http_mapping.
	jsonErrStr := "{\"error\":{\"details\":[{\"@type\":\"type.googleapis.com/google.rpc.ErrorInfo\", \"reason\":\"just because\", \"domain\":\"tests\"}]}}"
	hae := &googleapi.Error{
		Body: jsonErrStr,
	}
	err := WrapError(hae)

	var aerr *apierror.APIError
	if ok := errors.As(err, &aerr); !ok {
		t.Errorf("got false, want true")
	}

	httpErrInfo := &errdetails.ErrorInfo{Reason: "just because", Domain: "tests"}
	details := apierror.ErrDetails{ErrorInfo: httpErrInfo}
	if diff := cmp.Diff(aerr.Details(), details, cmp.Comparer(proto.Equal)); diff != "" {
		t.Errorf("got(-), want(+),: \n%s", diff)
	}
	if s := aerr.Reason(); s != "just because" {
		t.Errorf("Reason() got %s, want 'just because'", s)
	}
	if s := aerr.Domain(); s != "tests" {
		t.Errorf("Domain() got %s, want nil", s)
	}
	if err := aerr.Unwrap(); err != nil {
		t.Errorf("Unwrap() got %T, want nil", err)
	}
	if m := aerr.Metadata(); m != nil {
		t.Errorf("Metadata() got %v, want nil", m)
	}
}
