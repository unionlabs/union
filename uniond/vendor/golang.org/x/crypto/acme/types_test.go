// Copyright 2017 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package acme

import (
	"errors"
	"net/http"
	"reflect"
	"testing"
	"time"
)

func TestExternalAccountBindingString(t *testing.T) {
	eab := ExternalAccountBinding{
		KID: "kid",
		Key: []byte("key"),
	}
	got := eab.String()
	want := `&{KID: "kid", Key: redacted}`
	if got != want {
		t.Errorf("eab.String() = %q, want: %q", got, want)
	}
}

func TestRateLimit(t *testing.T) {
	now := time.Date(2017, 04, 27, 10, 0, 0, 0, time.UTC)
	f := timeNow
	defer func() { timeNow = f }()
	timeNow = func() time.Time { return now }

	h120, hTime := http.Header{}, http.Header{}
	h120.Set("Retry-After", "120")
	hTime.Set("Retry-After", "Tue Apr 27 11:00:00 2017")

	err1 := &Error{
		ProblemType: "urn:ietf:params:acme:error:nolimit",
		Header:      h120,
	}
	err2 := &Error{
		ProblemType: "urn:ietf:params:acme:error:rateLimited",
		Header:      h120,
	}
	err3 := &Error{
		ProblemType: "urn:ietf:params:acme:error:rateLimited",
		Header:      nil,
	}
	err4 := &Error{
		ProblemType: "urn:ietf:params:acme:error:rateLimited",
		Header:      hTime,
	}

	tt := []struct {
		err error
		res time.Duration
		ok  bool
	}{
		{nil, 0, false},
		{errors.New("dummy"), 0, false},
		{err1, 0, false},
		{err2, 2 * time.Minute, true},
		{err3, 0, true},
		{err4, time.Hour, true},
	}
	for i, test := range tt {
		res, ok := RateLimit(test.err)
		if ok != test.ok {
			t.Errorf("%d: RateLimit(%+v): ok = %v; want %v", i, test.err, ok, test.ok)
			continue
		}
		if res != test.res {
			t.Errorf("%d: RateLimit(%+v) = %v; want %v", i, test.err, res, test.res)
		}
	}
}

func TestAuthorizationError(t *testing.T) {
	tests := []struct {
		desc string
		err  *AuthorizationError
		msg  string
	}{
		{
			desc: "when auth error identifier is set",
			err: &AuthorizationError{
				Identifier: "domain.com",
				Errors: []error{
					(&wireError{
						Status: 403,
						Type:   "urn:ietf:params:acme:error:caa",
						Detail: "CAA record for domain.com prevents issuance",
					}).error(nil),
				},
			},
			msg: "acme: authorization error for domain.com: 403 urn:ietf:params:acme:error:caa: CAA record for domain.com prevents issuance",
		},

		{
			desc: "when auth error identifier is unset",
			err: &AuthorizationError{
				Errors: []error{
					(&wireError{
						Status: 403,
						Type:   "urn:ietf:params:acme:error:caa",
						Detail: "CAA record for domain.com prevents issuance",
					}).error(nil),
				},
			},
			msg: "acme: authorization error: 403 urn:ietf:params:acme:error:caa: CAA record for domain.com prevents issuance",
		},
	}

	for _, tt := range tests {
		if tt.err.Error() != tt.msg {
			t.Errorf("got: %s\nwant: %s", tt.err, tt.msg)
		}
	}
}

func TestSubproblems(t *testing.T) {
	tests := []struct {
		wire        wireError
		expectedOut Error
	}{
		{
			wire: wireError{
				Status: 1,
				Type:   "urn:error",
				Detail: "it's an error",
			},
			expectedOut: Error{
				StatusCode:  1,
				ProblemType: "urn:error",
				Detail:      "it's an error",
			},
		},
		{
			wire: wireError{
				Status: 1,
				Type:   "urn:error",
				Detail: "it's an error",
				Subproblems: []Subproblem{
					{
						Type:   "urn:error:sub",
						Detail: "it's a subproblem",
					},
				},
			},
			expectedOut: Error{
				StatusCode:  1,
				ProblemType: "urn:error",
				Detail:      "it's an error",
				Subproblems: []Subproblem{
					{
						Type:   "urn:error:sub",
						Detail: "it's a subproblem",
					},
				},
			},
		},
		{
			wire: wireError{
				Status: 1,
				Type:   "urn:error",
				Detail: "it's an error",
				Subproblems: []Subproblem{
					{
						Type:       "urn:error:sub",
						Detail:     "it's a subproblem",
						Identifier: &AuthzID{Type: "dns", Value: "example"},
					},
				},
			},
			expectedOut: Error{
				StatusCode:  1,
				ProblemType: "urn:error",
				Detail:      "it's an error",
				Subproblems: []Subproblem{
					{
						Type:       "urn:error:sub",
						Detail:     "it's a subproblem",
						Identifier: &AuthzID{Type: "dns", Value: "example"},
					},
				},
			},
		},
	}

	for _, tc := range tests {
		out := tc.wire.error(nil)
		if !reflect.DeepEqual(*out, tc.expectedOut) {
			t.Errorf("Unexpected error: wanted %v, got %v", tc.expectedOut, *out)
		}
	}
}

func TestErrorStringerWithSubproblems(t *testing.T) {
	err := Error{
		StatusCode:  1,
		ProblemType: "urn:error",
		Detail:      "it's an error",
		Subproblems: []Subproblem{
			{
				Type:   "urn:error:sub",
				Detail: "it's a subproblem",
			},
			{
				Type:       "urn:error:sub",
				Detail:     "it's a subproblem",
				Identifier: &AuthzID{Type: "dns", Value: "example"},
			},
		},
	}
	expectedStr := "1 urn:error: it's an error; subproblems:\n\turn:error:sub: it's a subproblem\n\turn:error:sub: [dns: example] it's a subproblem"
	if err.Error() != expectedStr {
		t.Errorf("Unexpected error string: wanted %q, got %q", expectedStr, err.Error())
	}
}
