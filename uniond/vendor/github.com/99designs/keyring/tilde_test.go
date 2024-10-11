//go:build linux
// +build linux

package keyring

import "testing"

func TestExpandTilde(t *testing.T) {
	t.Setenv("HOME", "/home/testing")
	actual, err := ExpandTilde("~/one/two")
	if err != nil {
		t.Fatal(err)
	}
	expected := "/home/testing/one/two"
	if actual != expected {
		t.Fatalf("%s != %s", expected, actual)
	}
}

func TestExpandTildeWithoutSlash(t *testing.T) {
	t.Setenv("HOME", "/home/testing")
	actual, err := ExpandTilde("~one/two")
	if err != nil {
		t.Fatal(err)
	}
	expected := "~one/two"
	if actual != expected {
		t.Fatalf("%s != %s", expected, actual)
	}
}
func TestExpandTildeWithoutLeadingTilde(t *testing.T) {
	t.Setenv("HOME", "/home/testing")
	actual, err := ExpandTilde("one/two~")
	if err != nil {
		t.Fatal(err)
	}
	expected := "one/two~"
	if actual != expected {
		t.Fatalf("%s != %s", expected, actual)
	}
}
