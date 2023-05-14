//go:build viper_toml1
// +build viper_toml1

package toml

import (
	"github.com/pelletier/go-toml"
)

// Codec implements the encoding.Encoder and encoding.Decoder interfaces for TOML encoding.
type Codec struct{}

func (Codec) Encode(v map[string]interface{}) ([]byte, error) {
	t, err := toml.TreeFromMap(v)
	if err != nil {
		return nil, err
	}

	s, err := t.ToTomlString()
	if err != nil {
		return nil, err
	}

	return []byte(s), nil
}

func (Codec) Decode(b []byte, v map[string]interface{}) error {
	tree, err := toml.LoadBytes(b)
	if err != nil {
		return err
	}

	tmap := tree.ToMap()
	for key, value := range tmap {
		v[key] = value
	}

	return nil
}
