// Copyright 2013-2023 The Cobra Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

package doc

import (
	"bytes"
	"fmt"
	"io/ioutil"
	"os"
	"path/filepath"
	"testing"

	"github.com/spf13/cobra"
)

func TestGenYamlDoc(t *testing.T) {
	// We generate on s subcommand so we have both subcommands and parents
	buf := new(bytes.Buffer)
	if err := GenYaml(echoCmd, buf); err != nil {
		t.Fatal(err)
	}
	output := buf.String()

	checkStringContains(t, output, echoCmd.Long)
	checkStringContains(t, output, echoCmd.Example)
	checkStringContains(t, output, "boolone")
	checkStringContains(t, output, "rootflag")
	checkStringContains(t, output, rootCmd.Short)
	checkStringContains(t, output, echoSubCmd.Short)
	checkStringContains(t, output, fmt.Sprintf("- %s - %s", echoSubCmd.CommandPath(), echoSubCmd.Short))
}

func TestGenYamlNoTag(t *testing.T) {
	rootCmd.DisableAutoGenTag = true
	defer func() { rootCmd.DisableAutoGenTag = false }()

	buf := new(bytes.Buffer)
	if err := GenYaml(rootCmd, buf); err != nil {
		t.Fatal(err)
	}
	output := buf.String()

	checkStringOmits(t, output, "Auto generated")
}

func TestGenYamlTree(t *testing.T) {
	c := &cobra.Command{Use: "do [OPTIONS] arg1 arg2"}

	tmpdir, err := ioutil.TempDir("", "test-gen-yaml-tree")
	if err != nil {
		t.Fatalf("Failed to create tmpdir: %s", err.Error())
	}
	defer os.RemoveAll(tmpdir)

	if err := GenYamlTree(c, tmpdir); err != nil {
		t.Fatalf("GenYamlTree failed: %s", err.Error())
	}

	if _, err := os.Stat(filepath.Join(tmpdir, "do.yaml")); err != nil {
		t.Fatalf("Expected file 'do.yaml' to exist")
	}
}

func TestGenYamlDocRunnable(t *testing.T) {
	// Testing a runnable command: should contain the "usage" field
	buf := new(bytes.Buffer)
	if err := GenYaml(rootCmd, buf); err != nil {
		t.Fatal(err)
	}
	output := buf.String()

	checkStringContains(t, output, "usage: "+rootCmd.Use)
}

func BenchmarkGenYamlToFile(b *testing.B) {
	file, err := ioutil.TempFile("", "")
	if err != nil {
		b.Fatal(err)
	}
	defer os.Remove(file.Name())
	defer file.Close()

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		if err := GenYaml(rootCmd, file); err != nil {
			b.Fatal(err)
		}
	}
}
