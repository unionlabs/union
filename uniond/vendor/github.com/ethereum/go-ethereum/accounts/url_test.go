// Copyright 2018 The go-ethereum Authors
// This file is part of the go-ethereum library.
//
// The go-ethereum library is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// The go-ethereum library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with the go-ethereum library. If not, see <http://www.gnu.org/licenses/>.

package accounts

import (
	"testing"
)

func TestURLParsing(t *testing.T) {
	url, err := parseURL("https://ethereum.org")
	if err != nil {
		t.Errorf("unexpected error: %v", err)
	}
	if url.Scheme != "https" {
		t.Errorf("expected: %v, got: %v", "https", url.Scheme)
	}
	if url.Path != "ethereum.org" {
		t.Errorf("expected: %v, got: %v", "ethereum.org", url.Path)
	}

	for _, u := range []string{"ethereum.org", ""} {
		if _, err = parseURL(u); err == nil {
			t.Errorf("input %v, expected err, got: nil", u)
		}
	}
}

func TestURLString(t *testing.T) {
	url := URL{Scheme: "https", Path: "ethereum.org"}
	if url.String() != "https://ethereum.org" {
		t.Errorf("expected: %v, got: %v", "https://ethereum.org", url.String())
	}

	url = URL{Scheme: "", Path: "ethereum.org"}
	if url.String() != "ethereum.org" {
		t.Errorf("expected: %v, got: %v", "ethereum.org", url.String())
	}
}

func TestURLMarshalJSON(t *testing.T) {
	url := URL{Scheme: "https", Path: "ethereum.org"}
	json, err := url.MarshalJSON()
	if err != nil {
		t.Errorf("unexpcted error: %v", err)
	}
	if string(json) != "\"https://ethereum.org\"" {
		t.Errorf("expected: %v, got: %v", "\"https://ethereum.org\"", string(json))
	}
}

func TestURLUnmarshalJSON(t *testing.T) {
	url := &URL{}
	err := url.UnmarshalJSON([]byte("\"https://ethereum.org\""))
	if err != nil {
		t.Errorf("unexpcted error: %v", err)
	}
	if url.Scheme != "https" {
		t.Errorf("expected: %v, got: %v", "https", url.Scheme)
	}
	if url.Path != "ethereum.org" {
		t.Errorf("expected: %v, got: %v", "https", url.Path)
	}
}

func TestURLComparison(t *testing.T) {
	tests := []struct {
		urlA   URL
		urlB   URL
		expect int
	}{
		{URL{"https", "ethereum.org"}, URL{"https", "ethereum.org"}, 0},
		{URL{"http", "ethereum.org"}, URL{"https", "ethereum.org"}, -1},
		{URL{"https", "ethereum.org/a"}, URL{"https", "ethereum.org"}, 1},
		{URL{"https", "abc.org"}, URL{"https", "ethereum.org"}, -1},
	}

	for i, tt := range tests {
		result := tt.urlA.Cmp(tt.urlB)
		if result != tt.expect {
			t.Errorf("test %d: cmp mismatch: expected: %d, got: %d", i, tt.expect, result)
		}
	}
}
