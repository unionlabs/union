package gotooltest

import (
	"os"
	"testing"
)

func TestInitGoEnv(t *testing.T) {
	// Set up a temp directory containing a bad go.mod file to
	// ensure the working directory does not influence the probe
	// commands run during initGoEnv
	td := t.TempDir()

	// If these commands fail we are in bigger trouble
	wd, _ := os.Getwd()
	os.Chdir(td)

	t.Cleanup(func() {
		os.Chdir(wd)
		os.Remove(td)
	})

	if err := os.WriteFile("go.mod", []byte("this is rubbish"), 0600); err != nil {
		t.Fatal(err)
	}

	if err := initGoEnv(); err != nil {
		t.Fatal(err)
	}
}
