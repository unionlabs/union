//go:build linux || darwin
// +build linux darwin

package main

import (
	"bytes"
	"log"
	"os"
	"os/exec"
	"plugin"
	"testing"
)

// This is a cursory test that checks whether things work under dynamic linking.

func TestMain(m *testing.M) {
	cmd := exec.Command(
		"go", "build",
		"-buildmode", "plugin",
		"-o", "plugin.so",
		"plugin.go",
	)
	var out bytes.Buffer
	cmd.Stdout = &out
	cmd.Stderr = &out
	if err := cmd.Run(); err != nil {
		log.Fatalf("Error building plugin: %s\nOutput:\n%s", err, out.String())
	}
	os.Exit(m.Run())
}

func TestDynamic(t *testing.T) {
	plug, err := plugin.Open("plugin.so")
	if err != nil {
		t.Fatal(err)
	}
	for _, test := range []string{
		"TestSum",
		"TestDigest",
	} {
		f, err := plug.Lookup(test)
		if err != nil {
			t.Fatalf("cannot find func %s: %s", test, err)
		}
		f.(func(*testing.T))(t)
	}
}
