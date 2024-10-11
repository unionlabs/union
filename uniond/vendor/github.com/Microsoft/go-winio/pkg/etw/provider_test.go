//go:build windows
// +build windows

package etw

import (
	"testing"

	"github.com/Microsoft/go-winio/pkg/guid"
)

func mustGUIDFromString(t *testing.T, s string) guid.GUID {
	g, err := guid.FromString(s)
	if err != nil {
		t.Fatal(err)
	}
	return g
}

func Test_ProviderIDFromName(t *testing.T) {
	type testCase struct {
		name string
		g    guid.GUID
	}
	testCases := []testCase{
		{"wincni", mustGUIDFromString(t, "c822b598-f4cc-5a72-7933-ce2a816d033f")},
		{"Moby", mustGUIDFromString(t, "6996f090-c5de-5082-a81e-5841acc3a635")},
		{"ContainerD", mustGUIDFromString(t, "2acb92c0-eb9b-571a-69cf-8f3410f383ad")},
		{"Microsoft.Virtualization.RunHCS", mustGUIDFromString(t, "0B52781F-B24D-5685-DDF6-69830ED40EC3")},
	}
	for _, tc := range testCases {
		g := providerIDFromName(tc.name)
		if g != tc.g {
			t.Fatalf("Incorrect provider GUID.\nExpected: %s\nActual: %s", tc.g, g)
		}
	}
}
