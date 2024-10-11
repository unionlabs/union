package guid

import (
	"encoding/json"
	"fmt"
	"testing"
)

func mustNewV4(t *testing.T) GUID {
	g, err := NewV4()
	if err != nil {
		t.Fatal(err)
	}
	return g
}

func mustNewV5(t *testing.T, namespace GUID, name []byte) GUID {
	g, err := NewV5(namespace, name)
	if err != nil {
		t.Fatal(err)
	}
	return g
}

func mustFromString(t *testing.T, s string) GUID {
	g, err := FromString(s)
	if err != nil {
		t.Fatal(err)
	}
	return g
}

func Test_Variant(t *testing.T) {
	type testCase struct {
		g GUID
		v Variant
	}
	testCases := []testCase{
		{mustFromString(t, "f5cbc1a9-4cba-45a0-0fdd-b6761fc7dcc0"), VariantNCS},
		{mustFromString(t, "f5cbc1a9-4cba-45a0-7fdd-b6761fc7dcc0"), VariantNCS},
		{mustFromString(t, "f5cbc1a9-4cba-45a0-bfdd-b6761fc7dcc0"), VariantRFC4122},
		{mustFromString(t, "f5cbc1a9-4cba-45a0-9fdd-b6761fc7dcc0"), VariantRFC4122},
		{mustFromString(t, "f5cbc1a9-4cba-45a0-cfdd-b6761fc7dcc0"), VariantMicrosoft},
		{mustFromString(t, "f5cbc1a9-4cba-45a0-dfdd-b6761fc7dcc0"), VariantMicrosoft},
		{mustFromString(t, "f5cbc1a9-4cba-45a0-efdd-b6761fc7dcc0"), VariantFuture},
		{mustFromString(t, "f5cbc1a9-4cba-45a0-ffdd-b6761fc7dcc0"), VariantFuture},
	}
	for _, tc := range testCases {
		t.Run(tc.v.String()+"/"+tc.g.String(), func(t *testing.T) {
			actualVariant := tc.g.Variant()
			if actualVariant != tc.v {
				t.Fatalf("Variant is not correct.\nExpected: %d\nActual: %d\nGUID: %s", tc.v, actualVariant, tc.g)
			}
		})
	}
}

func Test_SetVariant(t *testing.T) {
	g := mustFromString(t, "f5cbc1a9-4cba-45a0-bfdd-b6761fc7dcc0")
	for i := 0; i < len(_Variant_index)-1; i++ {
		v := Variant(i)
		if v == VariantUnknown {
			// Unknown is not a valid variant
			continue
		}
		t.Run(v.String(), func(t *testing.T) {
			g.setVariant(v)
			if g.Variant() != v {
				t.Fatalf("Variant is incorrect.\nExpected: %d\nActual: %d", v, g.Variant())
			}
		})
	}
}

func Test_Version(t *testing.T) {
	type testCase struct {
		g GUID
		v Version
	}
	testCases := []testCase{
		{mustFromString(t, "f5cbc1a9-4cba-15a0-0fdd-b6761fc7dcc0"), 1},
		{mustFromString(t, "f5cbc1a9-4cba-25a0-0fdd-b6761fc7dcc0"), 2},
		{mustFromString(t, "f5cbc1a9-4cba-35a0-0fdd-b6761fc7dcc0"), 3},
		{mustFromString(t, "f5cbc1a9-4cba-45a0-0fdd-b6761fc7dcc0"), 4},
		{mustFromString(t, "f5cbc1a9-4cba-55a0-0fdd-b6761fc7dcc0"), 5},
	}
	for _, tc := range testCases {
		t.Run(tc.v.String()+"-"+tc.g.String(), func(t *testing.T) {
			actualVersion := tc.g.Version()
			if actualVersion != tc.v {
				t.Fatalf("Version is not correct.\nExpected: %d\nActual: %d\nGUID: %s", tc.v, actualVersion, tc.g)
			}
		})
	}
}

func Test_SetVersion(t *testing.T) {
	g := mustFromString(t, "f5cbc1a9-4cba-45a0-bfdd-b6761fc7dcc0")
	for tc := 0; tc < 16; tc++ {
		v := Version(tc)
		t.Run(v.String(), func(t *testing.T) {
			g.setVersion(v)
			if g.Version() != v {
				t.Fatalf("Version is incorrect.\nExpected: %d\nActual: %d", v, g.Version())
			}
		})
	}
}

func Test_NewV4IsUnique(t *testing.T) {
	g := mustNewV4(t)
	g2 := mustNewV4(t)
	if g == g2 {
		t.Fatalf("GUIDs are equal: %s, %s", g, g2)
	}
}

func Test_V4HasCorrectVersionAndVariant(t *testing.T) {
	g := mustNewV4(t)
	if g.Version() != 4 {
		t.Fatalf("Version is not 4: %s", g)
	}
	if g.Variant() != VariantRFC4122 {
		t.Fatalf("Variant is not RFC4122: %s", g)
	}
}

func Test_V5HasCorrectVersionAndVariant(t *testing.T) {
	namespace := mustFromString(t, "f5cbc1a9-4cba-45a0-bfdd-b6761fc7dcc0")
	g := mustNewV5(t, namespace, []byte("Foo"))
	if g.Version() != 5 {
		t.Fatalf("Version is not 5: %s", g)
	}
	if g.Variant() != VariantRFC4122 {
		t.Fatalf("Variant is not RFC4122: %s", g)
	}
}

func Test_V5KnownValues(t *testing.T) {
	type testCase struct {
		ns   GUID
		name string
		g    GUID
	}
	testCases := []testCase{
		{
			mustFromString(t, "6ba7b810-9dad-11d1-80b4-00c04fd430c8"),
			"www.sample.com",
			mustFromString(t, "4e4463eb-b0e8-54fa-8c28-12d1ab1d45b3"),
		},
		{
			mustFromString(t, "6ba7b811-9dad-11d1-80b4-00c04fd430c8"),
			"https://www.sample.com/test",
			mustFromString(t, "9e44625a-0d85-5e0a-99bc-8e8a77df5ea2"),
		},
		{
			mustFromString(t, "6ba7b812-9dad-11d1-80b4-00c04fd430c8"),
			"1.3.6.1.4.1.343",
			mustFromString(t, "6aab0456-7392-582a-b92a-ba5a7096945d"),
		},
		{
			mustFromString(t, "6ba7b814-9dad-11d1-80b4-00c04fd430c8"),
			"CN=John Smith, ou=People, o=FakeCorp, L=Seattle, S=Washington, C=US",
			mustFromString(t, "badff8dd-c869-5b64-a260-00092e66be00"),
		},
	}
	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			g := mustNewV5(t, tc.ns, []byte(tc.name))
			if g != tc.g {
				t.Fatalf("GUIDs are not equal.\nExpected: %s\nActual: %s", tc.g, g)
			}
		})
	}
}

func Test_ToArray(t *testing.T) {
	g := mustFromString(t, "73c39589-192e-4c64-9acf-6c5d0aa18528")
	b := g.ToArray()
	expected := [16]byte{0x73, 0xc3, 0x95, 0x89, 0x19, 0x2e, 0x4c, 0x64, 0x9a, 0xcf, 0x6c, 0x5d, 0x0a, 0xa1, 0x85, 0x28}
	if b != expected {
		t.Fatalf("GUID does not match array form: %x, %x", expected, b)
	}
}

func Test_FromArrayAndBack(t *testing.T) {
	b := [16]byte{0x73, 0xc3, 0x95, 0x89, 0x19, 0x2e, 0x4c, 0x64, 0x9a, 0xcf, 0x6c, 0x5d, 0x0a, 0xa1, 0x85, 0x28}
	b2 := FromArray(b).ToArray()
	if b != b2 {
		t.Fatalf("Arrays do not match: %x, %x", b, b2)
	}
}

func Test_ToWindowsArray(t *testing.T) {
	g := mustFromString(t, "73c39589-192e-4c64-9acf-6c5d0aa18528")
	b := g.ToWindowsArray()
	expected := [16]byte{0x89, 0x95, 0xc3, 0x73, 0x2e, 0x19, 0x64, 0x4c, 0x9a, 0xcf, 0x6c, 0x5d, 0x0a, 0xa1, 0x85, 0x28}
	if b != expected {
		t.Fatalf("GUID does not match array form: %x, %x", expected, b)
	}
}

func Test_FromWindowsArrayAndBack(t *testing.T) {
	b := [16]byte{0x73, 0xc3, 0x95, 0x89, 0x19, 0x2e, 0x4c, 0x64, 0x9a, 0xcf, 0x6c, 0x5d, 0x0a, 0xa1, 0x85, 0x28}
	b2 := FromWindowsArray(b).ToWindowsArray()
	if b != b2 {
		t.Fatalf("Arrays do not match: %x, %x", b, b2)
	}
}

func Test_FromString(t *testing.T) {
	orig := "8e35239e-2084-490e-a3db-ab18ee0744cb"
	g := mustFromString(t, orig)
	s := g.String()
	if orig != s {
		t.Fatalf("GUIDs not equal: %s, %s", orig, s)
	}
}

func Test_MarshalJSON(t *testing.T) {
	g := mustNewV4(t)
	j, err := json.Marshal(g)
	if err != nil {
		t.Fatal(err)
	}
	gj := fmt.Sprintf("\"%s\"", g.String())
	if string(j) != gj {
		t.Fatalf("JSON not equal: %s, %s", j, gj)
	}
}

func Test_MarshalJSON_Nested(t *testing.T) {
	type test struct {
		G GUID
	}
	g := mustNewV4(t)
	t1 := test{g}
	j, err := json.Marshal(t1)
	if err != nil {
		t.Fatal(err)
	}
	gj := fmt.Sprintf("{\"G\":\"%s\"}", g.String())
	if string(j) != gj {
		t.Fatalf("JSON not equal: %s, %s", j, gj)
	}
}

func Test_UnmarshalJSON(t *testing.T) {
	g := mustNewV4(t)
	j, err := json.Marshal(g)
	if err != nil {
		t.Fatal(err)
	}
	var g2 GUID
	if err := json.Unmarshal(j, &g2); err != nil {
		t.Fatal(err)
	}
	if g != g2 {
		t.Fatalf("GUIDs not equal: %s, %s", g, g2)
	}
}

func Test_UnmarshalJSON_Nested(t *testing.T) {
	type test struct {
		G GUID
	}
	g := mustNewV4(t)
	t1 := test{g}
	j, err := json.Marshal(t1)
	if err != nil {
		t.Fatal(err)
	}
	var t2 test
	if err := json.Unmarshal(j, &t2); err != nil {
		t.Fatal(err)
	}
	if t1.G != t2.G {
		t.Fatalf("GUIDs not equal: %v, %v", t1.G, t2.G)
	}
}
