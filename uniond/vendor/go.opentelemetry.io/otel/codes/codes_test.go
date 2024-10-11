// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package codes

import (
	"bytes"
	"encoding/json"
	"fmt"
	"testing"
)

func TestCodeString(t *testing.T) {
	tests := []struct {
		code Code
		want string
	}{
		{Unset, "Unset"},
		{Error, "Error"},
		{Ok, "Ok"},
	}

	for _, test := range tests {
		if got := test.code.String(); got != test.want {
			t.Errorf("String of code %d %q, want %q", test.code, got, test.want)
		}
	}
}

func TestCodeUnmarshalJSONNull(t *testing.T) {
	c := new(Code)
	orig := c
	if err := c.UnmarshalJSON([]byte("null")); err != nil {
		t.Fatalf("Code.UnmarshalJSON(\"null\") errored: %v", err)
	}
	if orig != c {
		t.Error("Code.UnmarshalJSON(\"null\") should not decode a value")
	}
}

func TestCodeUnmarshalJSONNil(t *testing.T) {
	c := (*Code)(nil)
	if err := c.UnmarshalJSON([]byte{}); err == nil {
		t.Fatalf("Code(nil).UnmarshalJSON() did not error")
	}
}

func TestCodeUnmarshalJSON(t *testing.T) {
	tests := []struct {
		input string
		want  Code
	}{
		{"0", Unset},
		{`"Unset"`, Unset},
		{"1", Error},
		{`"Error"`, Error},
		{"2", Ok},
		{`"Ok"`, Ok},
	}
	for _, test := range tests {
		c := new(Code)
		*c = Code(maxCode)

		if err := json.Unmarshal([]byte(test.input), c); err != nil {
			t.Fatalf("json.Unmarshal(%q, Code) errored: %v", test.input, err)
		}
		if *c != test.want {
			t.Errorf("failed to unmarshal %q as %v", test.input, test.want)
		}
	}
}

func TestCodeUnmarshalJSONErrorInvalidData(t *testing.T) {
	tests := []string{
		fmt.Sprintf("%d", maxCode),
		"Not a code",
		"Unset",
		"true",
		`"Not existing"`,
		"",
	}
	c := new(Code)
	for _, test := range tests {
		if err := json.Unmarshal([]byte(test), c); err == nil {
			t.Fatalf("json.Unmarshal(%q, Code) did not error", test)
		}
	}
}

func TestCodeMarshalJSONNil(t *testing.T) {
	c := (*Code)(nil)
	b, err := c.MarshalJSON()
	if err != nil {
		t.Fatalf("Code(nil).MarshalJSON() errored: %v", err)
	}
	if !bytes.Equal(b, []byte("null")) {
		t.Errorf("Code(nil).MarshalJSON() returned %s, want \"null\"", string(b))
	}
}

func TestCodeMarshalJSON(t *testing.T) {
	tests := []struct {
		code Code
		want string
	}{
		{Unset, `"Unset"`},
		{Error, `"Error"`},
		{Ok, `"Ok"`},
	}

	for _, test := range tests {
		b, err := test.code.MarshalJSON()
		if err != nil {
			t.Fatalf("Code(%s).MarshalJSON() errored: %v", test.code, err)
		}
		if !bytes.Equal(b, []byte(test.want)) {
			t.Errorf("Code(%s).MarshalJSON() returned %s, want %s", test.code, string(b), test.want)
		}
	}
}

func TestCodeMarshalJSONErrorInvalid(t *testing.T) {
	c := new(Code)
	*c = Code(maxCode)
	if b, err := c.MarshalJSON(); err == nil {
		t.Fatalf("Code(maxCode).MarshalJSON() did not error")
	} else if b != nil {
		t.Fatal("Code(maxCode).MarshalJSON() returned non-nil value")
	}
}

func TestRoundTripCodes(t *testing.T) {
	tests := []struct {
		input Code
	}{
		{Unset},
		{Error},
		{Ok},
	}
	for _, test := range tests {
		c := test.input
		out := new(Code)

		b, err := c.MarshalJSON()
		if err != nil {
			t.Fatalf("Code(%s).MarshalJSON() errored: %v", test.input, err)
		}

		if err := out.UnmarshalJSON(b); err != nil {
			t.Fatalf("Code.UnmarshalJSON(%q) errored: %v", c, err)
		}

		if *out != test.input {
			t.Errorf("failed to round trip %q, output was %v", test.input, out)
		}
	}
}
