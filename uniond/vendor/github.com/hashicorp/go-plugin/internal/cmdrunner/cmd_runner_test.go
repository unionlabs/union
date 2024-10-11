// Copyright (c) HashiCorp, Inc.
// SPDX-License-Identifier: MPL-2.0

package cmdrunner

import (
	"os"
	"path/filepath"
	"strings"
	"testing"
)

func TestAdditionalNotesAboutCommand(t *testing.T) {
	files := []string{
		"windows-amd64.exe",
		"windows-386.exe",
		"linux-amd64",
		"darwin-amd64",
		"darwin-arm64",
	}
	for _, file := range files {
		fullFile := filepath.Join("testdata", file)
		if _, err := os.Stat(fullFile); os.IsNotExist(err) {
			t.Skipf("testdata executables not present; please run 'make' in testdata/ directory for this test")
		}

		notes := additionalNotesAboutCommand(fullFile)
		if strings.Contains(file, "windows") && !strings.Contains(notes, "PE") {
			t.Errorf("Expected notes to contain Windows information:\n%s", notes)
		}
		if strings.Contains(file, "linux") && !strings.Contains(notes, "ELF") {
			t.Errorf("Expected notes to contain Linux information:\n%s", notes)
		}
		if strings.Contains(file, "darwin") && !strings.Contains(notes, "MachO") {
			t.Errorf("Expected notes to contain macOS information:\n%s", notes)
		}

		if strings.Contains(file, "amd64") && !(strings.Contains(notes, "amd64") || strings.Contains(notes, "EM_X86_64") || strings.Contains(notes, "CpuAmd64")) {
			t.Errorf("Expected notes to contain amd64 information:\n%s", notes)
		}

		if strings.Contains(file, "arm64") && !strings.Contains(notes, "CpuArm64") {
			t.Errorf("Expected notes to contain arm64 information:\n%s", notes)
		}
		if strings.Contains(file, "386") && !strings.Contains(notes, "386") {
			t.Errorf("Expected notes to contain 386 information:\n%s", notes)
		}

	}
}
