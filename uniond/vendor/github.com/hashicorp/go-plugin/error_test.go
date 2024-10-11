// Copyright (c) HashiCorp, Inc.
// SPDX-License-Identifier: MPL-2.0

package plugin

import (
	"errors"
	"testing"
)

func TestBasicError_ImplementsError(t *testing.T) {
	var _ error = new(BasicError)
}

func TestBasicError_MatchesMessage(t *testing.T) {
	err := errors.New("foo")
	wrapped := NewBasicError(err)

	if wrapped.Error() != err.Error() {
		t.Fatalf("bad: %#v", wrapped.Error())
	}
}

func TestNewBasicError_nil(t *testing.T) {
	r := NewBasicError(nil)
	if r != nil {
		t.Fatalf("bad: %#v", r)
	}
}
