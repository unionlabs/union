package genesis

import (
	"encoding/json"
	"io"
	"testing"

	"cosmossdk.io/core/appmodule"
)

func TestSource(t *testing.T) {
	source, err := SourceFromRawJSON(json.RawMessage(testJSON))
	if err != nil {
		t.Errorf("Error creating source: %s", err)
	}

	expectJSON(t, source, "foo", fooContents)
	expectJSON(t, source, "bar", barContents)

	// missing fields just return nil, nil
	r, err := source("baz")
	if err != nil {
		t.Errorf("Error retrieving field: %s", err)
	}
	if r != nil {
		t.Errorf("Expected nil result for missing field, got: %v", r)
	}
}

func expectJSON(t *testing.T, source appmodule.GenesisSource, field, contents string) {
	t.Helper()
	r, err := source(field)
	if err != nil {
		t.Errorf("Error retrieving field: %s", err)
	}
	bz, err := io.ReadAll(r)
	if err != nil {
		t.Errorf("Error reading contents: %s", err)
	}
	if string(bz) != contents {
		t.Errorf("Expected contents: %s, got: %s", contents, string(bz))
	}
}

const (
	testJSON = `
{
	"foo":{"x":1,"y":"abc"},
	"bar":[1,2,3,4]
}
`
	fooContents = `{"x":1,"y":"abc"}`
	barContents = `[1,2,3,4]`
)
