// Copyright (c) HashiCorp, Inc.
// SPDX-License-Identifier: MPL-2.0

package version

import (
	"fmt"
	"reflect"
	"sort"
	"testing"
)

func TestNewConstraint(t *testing.T) {
	cases := []struct {
		input string
		count int
		err   bool
	}{
		{">= 1.2", 1, false},
		{"1.0", 1, false},
		{">= 1.x", 0, true},
		{">= 1.2, < 1.0", 2, false},

		// Out of bounds
		{"11387778780781445675529500000000000000000", 0, true},
	}

	for _, tc := range cases {
		v, err := NewConstraint(tc.input)
		if tc.err && err == nil {
			t.Fatalf("expected error for input: %s", tc.input)
		} else if !tc.err && err != nil {
			t.Fatalf("error for input %s: %s", tc.input, err)
		}

		if len(v) != tc.count {
			t.Fatalf("input: %s\nexpected len: %d\nactual: %d",
				tc.input, tc.count, len(v))
		}
	}
}

func TestConstraintCheck(t *testing.T) {
	cases := []struct {
		constraint string
		version    string
		check      bool
	}{
		{">= 1.0, < 1.2", "1.1.5", true},
		{"< 1.0, < 1.2", "1.1.5", false},
		{"= 1.0", "1.1.5", false},
		{"= 1.0", "1.0.0", true},
		{"1.0", "1.0.0", true},
		{"~> 1.0", "2.0", false},
		{"~> 1.0", "1.1", true},
		{"~> 1.0", "1.2.3", true},
		{"~> 1.0.0", "1.2.3", false},
		{"~> 1.0.0", "1.0.7", true},
		{"~> 1.0.0", "1.1.0", false},
		{"~> 1.0.7", "1.0.4", false},
		{"~> 1.0.7", "1.0.7", true},
		{"~> 1.0.7", "1.0.8", true},
		{"~> 1.0.7", "1.0.7.5", true},
		{"~> 1.0.7", "1.0.6.99", false},
		{"~> 1.0.7", "1.0.8.0", true},
		{"~> 1.0.9.5", "1.0.9.5", true},
		{"~> 1.0.9.5", "1.0.9.4", false},
		{"~> 1.0.9.5", "1.0.9.6", true},
		{"~> 1.0.9.5", "1.0.9.5.0", true},
		{"~> 1.0.9.5", "1.0.9.5.1", true},
		{"~> 2.0", "2.1.0-beta", false},
		{"~> 2.1.0-a", "2.2.0", false},
		{"~> 2.1.0-a", "2.1.0", false},
		{"~> 2.1.0-a", "2.1.0-beta", true},
		{"~> 2.1.0-a", "2.2.0-alpha", false},
		{"> 2.0", "2.1.0-beta", false},
		{">= 2.1.0-a", "2.1.0-beta", true},
		{">= 2.1.0-a", "2.1.1-beta", false},
		{">= 2.0.0", "2.1.0-beta", false},
		{">= 2.1.0-a", "2.1.1", true},
		{">= 2.1.0-a", "2.1.1-beta", false},
		{">= 2.1.0-a", "2.1.0", true},
		{"<= 2.1.0-a", "2.0.0", true},
	}

	for _, tc := range cases {
		c, err := NewConstraint(tc.constraint)
		if err != nil {
			t.Fatalf("err: %s", err)
		}

		v, err := NewVersion(tc.version)
		if err != nil {
			t.Fatalf("err: %s", err)
		}

		actual := c.Check(v)
		expected := tc.check
		if actual != expected {
			t.Fatalf("Version: %s\nConstraint: %s\nExpected: %#v",
				tc.version, tc.constraint, expected)
		}
	}
}

func TestConstraintPrerelease(t *testing.T) {
	cases := []struct {
		constraint string
		prerelease bool
	}{
		{"= 1.0", false},
		{"= 1.0-beta", true},
		{"~> 2.1.0", false},
		{"~> 2.1.0-dev", true},
		{"> 2.0", false},
		{">= 2.1.0-a", true},
	}

	for _, tc := range cases {
		c, err := parseSingle(tc.constraint)
		if err != nil {
			t.Fatalf("err: %s", err)
		}

		actual := c.Prerelease()
		expected := tc.prerelease
		if actual != expected {
			t.Fatalf("Constraint: %s\nExpected: %#v",
				tc.constraint, expected)
		}
	}
}

func TestConstraintEqual(t *testing.T) {
	cases := []struct {
		leftConstraint  string
		rightConstraint string
		expectedEqual   bool
	}{
		{
			"0.0.1",
			"0.0.1",
			true,
		},
		{ // whitespaces
			" 0.0.1 ",
			"0.0.1",
			true,
		},
		{ // equal op implied
			"=0.0.1 ",
			"0.0.1",
			true,
		},
		{ // version difference
			"=0.0.1",
			"=0.0.2",
			false,
		},
		{ // operator difference
			">0.0.1",
			"=0.0.1",
			false,
		},
		{ // different order
			">0.1.0, <=1.0.0",
			"<=1.0.0, >0.1.0",
			true,
		},
	}

	for _, tc := range cases {
		leftCon, err := NewConstraint(tc.leftConstraint)
		if err != nil {
			t.Fatalf("err: %s", err)
		}
		rightCon, err := NewConstraint(tc.rightConstraint)
		if err != nil {
			t.Fatalf("err: %s", err)
		}

		actual := leftCon.Equals(rightCon)
		if actual != tc.expectedEqual {
			t.Fatalf("Constraints: %s vs %s\nExpected: %t\nActual: %t",
				tc.leftConstraint, tc.rightConstraint, tc.expectedEqual, actual)
		}
	}
}

func TestConstraint_sort(t *testing.T) {
	cases := []struct {
		constraint          string
		expectedConstraints string
	}{
		{
			">= 0.1.0,< 1.12",
			"< 1.12,>= 0.1.0",
		},
		{
			"< 1.12,>= 0.1.0",
			"< 1.12,>= 0.1.0",
		},
		{
			"< 1.12,>= 0.1.0,0.2.0",
			"< 1.12,0.2.0,>= 0.1.0",
		},
		{
			">1.0,>0.1.0,>0.3.0,>0.2.0",
			">0.1.0,>0.2.0,>0.3.0,>1.0",
		},
	}

	for i, tc := range cases {
		t.Run(fmt.Sprintf("%d", i), func(t *testing.T) {
			c, err := NewConstraint(tc.constraint)
			if err != nil {
				t.Fatalf("err: %s", err)
			}

			sort.Sort(c)

			actual := c.String()

			if !reflect.DeepEqual(actual, tc.expectedConstraints) {
				t.Fatalf("unexpected order\nexpected: %#v\nactual: %#v",
					tc.expectedConstraints, actual)
			}
		})
	}
}

func TestConstraintsString(t *testing.T) {
	cases := []struct {
		constraint string
		result     string
	}{
		{">= 1.0, < 1.2", ""},
		{"~> 1.0.7", ""},
	}

	for _, tc := range cases {
		c, err := NewConstraint(tc.constraint)
		if err != nil {
			t.Fatalf("err: %s", err)
		}

		actual := c.String()
		expected := tc.result
		if expected == "" {
			expected = tc.constraint
		}

		if actual != expected {
			t.Fatalf("Constraint: %s\nExpected: %#v\nActual: %s",
				tc.constraint, expected, actual)
		}
	}
}
