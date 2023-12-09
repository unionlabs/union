package genesis

import (
	"bytes"
	"encoding/json"
	"io"

	"cosmossdk.io/core/appmodule"
)

// SourceFromRawJSON returns a genesis source based on a raw JSON message.
func SourceFromRawJSON(message json.RawMessage) (appmodule.GenesisSource, error) {
	var m map[string]json.RawMessage
	err := json.Unmarshal(message, &m)
	if err != nil {
		return nil, err
	}
	return func(field string) (io.ReadCloser, error) {
		j, ok := m[field]
		if !ok {
			return nil, nil
		}
		return readCloserWrapper{bytes.NewReader(j)}, nil
	}, nil
}

type readCloserWrapper struct {
	io.Reader
}

func (r readCloserWrapper) Close() error { return nil }
