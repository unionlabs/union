package genopenapi

import (
	"encoding/json"
	"errors"
	"io"

	"gopkg.in/yaml.v3"
)

type Format string

const (
	FormatJSON Format = "json"
	FormatYAML Format = "yaml"
)

type ContentEncoder interface {
	Encode(v interface{}) (err error)
}

func (f Format) Validate() error {
	switch f {
	case FormatJSON, FormatYAML:
		return nil
	default:
		return errors.New("unknown format: " + string(f))
	}
}

func (f Format) NewEncoder(w io.Writer) (ContentEncoder, error) {
	switch f {
	case FormatYAML:
		enc := yaml.NewEncoder(w)
		enc.SetIndent(2)

		return enc, nil
	case FormatJSON:
		enc := json.NewEncoder(w)
		enc.SetIndent("", "  ")

		return enc, nil
	default:
		return nil, errors.New("unknown format: " + string(f))
	}
}
