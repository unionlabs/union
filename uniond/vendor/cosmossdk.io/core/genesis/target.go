package genesis

import (
	"bytes"
	"encoding/json"
	"io"

	"cosmossdk.io/core/appmodule"
)

// RawJSONTarget returns a struct which encapsulates a genesis target that is
// backed by raw JSON messages. Its Target method should be used to retrieve
// an actual genesis target function. When genesis writing is done, the JSON
// method should be called to retrieve the raw message that has been written.
type RawJSONTarget struct {
	m map[string]json.RawMessage
}

// Target returns the actual genesis target function.
func (r *RawJSONTarget) Target() appmodule.GenesisTarget {
	return func(field string) (io.WriteCloser, error) {
		if r.m == nil {
			r.m = map[string]json.RawMessage{}
		}

		return &genesisWriter{Buffer: &bytes.Buffer{}, sink: r, field: field}, nil
	}
}

// JSON returns the raw JSON message that has been written.
func (r *RawJSONTarget) JSON() (json.RawMessage, error) {
	return json.MarshalIndent(r.m, "", "  ")
}

type genesisWriter struct {
	*bytes.Buffer
	field string
	sink  *RawJSONTarget
}

func (r genesisWriter) Close() error {
	r.sink.m[r.field] = r.Buffer.Bytes()
	return nil
}
