//go:generate go run github.com/toml-lang/toml-test/cmd/toml-test@master -copy ./tests
//go:generate go run ./cmd/tomltestgen/main.go -o toml_testgen_test.go

// This is a support file for toml_testgen_test.go
package toml_test

import (
	"encoding/json"
	"testing"

	"github.com/pelletier/go-toml/v2"
	"github.com/pelletier/go-toml/v2/internal/testsuite"
	"github.com/stretchr/testify/require"
)

func testgenInvalid(t *testing.T, input string) {
	t.Helper()
	t.Logf("Input TOML:\n%s", input)

	doc := map[string]interface{}{}
	err := testsuite.Unmarshal([]byte(input), &doc)

	if err == nil {
		out, err := json.Marshal(doc)
		if err != nil {
			panic("could not marshal map to json")
		}
		t.Log("JSON output from unmarshal:", string(out))
		t.Fatalf("test did not fail")
	}
}

func testgenValid(t *testing.T, input string, jsonRef string) {
	t.Helper()
	t.Logf("Input TOML:\n%s", input)

	// TODO: change this to interface{}
	var doc map[string]interface{}

	err := testsuite.Unmarshal([]byte(input), &doc)
	if err != nil {
		if de, ok := err.(*toml.DecodeError); ok {
			t.Logf("%s\n%s", err, de)
		}
		t.Fatalf("failed parsing toml: %s", err)
	}
	j, err := testsuite.ValueToTaggedJSON(doc)
	require.NoError(t, err)

	var ref interface{}
	err = json.Unmarshal([]byte(jsonRef), &ref)
	require.NoError(t, err)

	var actual interface{}
	err = json.Unmarshal([]byte(j), &actual)
	require.NoError(t, err)

	testsuite.CmpJSON(t, "", ref, actual)
}
