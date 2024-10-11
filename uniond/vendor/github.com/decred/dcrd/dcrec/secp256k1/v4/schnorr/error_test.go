// Copyright (c) 2020 The Decred developers
// Use of this source code is governed by an ISC
// license that can be found in the LICENSE file.

package schnorr

import (
	"errors"
	"testing"
)

// TestErrorKindStringer tests the stringized output for the ErrorKind type.
func TestErrorKindStringer(t *testing.T) {
	tests := []struct {
		in   ErrorKind
		want string
	}{
		{ErrInvalidHashLen, "ErrInvalidHashLen"},
		{ErrPrivateKeyIsZero, "ErrPrivateKeyIsZero"},
		{ErrSchnorrHashValue, "ErrSchnorrHashValue"},
		{ErrPubKeyNotOnCurve, "ErrPubKeyNotOnCurve"},
		{ErrSigRYIsOdd, "ErrSigRYIsOdd"},
		{ErrSigRNotOnCurve, "ErrSigRNotOnCurve"},
		{ErrUnequalRValues, "ErrUnequalRValues"},
		{ErrSigTooShort, "ErrSigTooShort"},
		{ErrSigTooLong, "ErrSigTooLong"},
		{ErrSigRTooBig, "ErrSigRTooBig"},
		{ErrSigSTooBig, "ErrSigSTooBig"},
	}

	for i, test := range tests {
		result := test.in.Error()
		if result != test.want {
			t.Errorf("#%d: got: %s want: %s", i, result, test.want)
			continue
		}
	}
}

// TestError tests the error output for the Error type.
func TestError(t *testing.T) {
	tests := []struct {
		in   Error
		want string
	}{{
		Error{Description: "some error"},
		"some error",
	}, {
		Error{Description: "human-readable error"},
		"human-readable error",
	}}

	for i, test := range tests {
		result := test.in.Error()
		if result != test.want {
			t.Errorf("#%d: got: %s want: %s", i, result, test.want)
			continue
		}
	}
}

// TestErrorKindIsAs ensures both ErrorKind and Error can be identified
// as being a specific error via errors.Is and unwrapped via errors.As.
func TestErrorKindIsAs(t *testing.T) {
	tests := []struct {
		name      string
		err       error
		target    error
		wantMatch bool
		wantAs    ErrorKind
	}{{
		name:      "ErrInvalidHashLen == ErrInvalidHashLen",
		err:       ErrInvalidHashLen,
		target:    ErrInvalidHashLen,
		wantMatch: true,
		wantAs:    ErrInvalidHashLen,
	}, {
		name:      "Error.ErrInvalidHashLen == ErrInvalidHashLen",
		err:       signatureError(ErrInvalidHashLen, ""),
		target:    ErrInvalidHashLen,
		wantMatch: true,
		wantAs:    ErrInvalidHashLen,
	}, {
		name:      "Error.ErrInvalidHashLen == Error.ErrInvalidHashLen",
		err:       signatureError(ErrInvalidHashLen, ""),
		target:    signatureError(ErrInvalidHashLen, ""),
		wantMatch: true,
		wantAs:    ErrInvalidHashLen,
	}, {
		name:      "ErrPrivateKeyIsZero != ErrInvalidHashLen",
		err:       ErrPrivateKeyIsZero,
		target:    ErrInvalidHashLen,
		wantMatch: false,
		wantAs:    ErrPrivateKeyIsZero,
	}, {
		name:      "Error.ErrPrivateKeyIsZero != ErrInvalidHashLen",
		err:       signatureError(ErrPrivateKeyIsZero, ""),
		target:    ErrInvalidHashLen,
		wantMatch: false,
		wantAs:    ErrPrivateKeyIsZero,
	}, {
		name:      "ErrPrivateKeyIsZero != Error.ErrInvalidHashLen",
		err:       ErrPrivateKeyIsZero,
		target:    signatureError(ErrInvalidHashLen, ""),
		wantMatch: false,
		wantAs:    ErrPrivateKeyIsZero,
	}, {
		name:      "Error.ErrPrivateKeyIsZero != Error.ErrInvalidHashLen",
		err:       signatureError(ErrPrivateKeyIsZero, ""),
		target:    signatureError(ErrInvalidHashLen, ""),
		wantMatch: false,
		wantAs:    ErrPrivateKeyIsZero,
	}}

	for _, test := range tests {
		// Ensure the error matches or not depending on the expected result.
		result := errors.Is(test.err, test.target)
		if result != test.wantMatch {
			t.Errorf("%s: incorrect error identification -- got %v, want %v",
				test.name, result, test.wantMatch)
			continue
		}

		// Ensure the underlying error kind can be unwrapped and is the
		// expected code.
		var code ErrorKind
		if !errors.As(test.err, &code) {
			t.Errorf("%s: unable to unwrap to error", test.name)
			continue
		}
		if !errors.Is(code, test.wantAs) {
			t.Errorf("%s: unexpected unwrapped error -- got %v, want %v",
				test.name, code, test.wantAs)
			continue
		}
	}
}
