// Copyright 2019 The Cockroach Authors.
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

package join

import (
	"errors"
	"testing"

	"github.com/cockroachdb/errors/safedetails"
	"github.com/cockroachdb/redact"
)

func TestJoin(t *testing.T) {
	e := Join(errors.New("abc123"), errors.New("def456"))
	expected := "abc123\ndef456"
	if e.Error() != expected {
		t.Errorf("Expected: %s; Got: %s", expected, e.Error())
	}

	e = Join(errors.New("abc123"), nil, errors.New("def456"), nil)
	if e.Error() != expected {
		t.Errorf("Expected: %s; Got: %s", expected, e.Error())
	}

	e = Join(nil, nil, nil)
	if e != nil {
		t.Errorf("expected nil error")
	}

	e = Join(
		errors.New("information"),
		safedetails.WithSafeDetails(errors.New("detailed error"), "trace_id: %d", redact.Safe(1234)),
	)
	printed := redact.Sprintf("%+v", e)
	expectedR := redact.RedactableString(`‹information›
(1) ‹information›
  | ‹detailed error›
Wraps: (2) trace_id: 1234
└─ Wraps: (3) ‹detailed error›
Wraps: (4) ‹information›
Error types: (1) *join.joinError (2) *safedetails.withSafeDetails (3) *errors.errorString (4) *errors.errorString`)
	if printed != expectedR {
		t.Errorf("Expected: %s; Got: %s", expectedR, printed)
	}
}
