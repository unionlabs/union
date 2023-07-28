package ini

import (
	"bytes"
	"sort"
	"strings"

	"github.com/spf13/cast"
	"gopkg.in/ini.v1"
)

// LoadOptions contains all customized options used for load data source(s).
// This type is added here for convenience: this way consumers can import a single package called "ini".
type LoadOptions = ini.LoadOptions

// Codec implements the encoding.Encoder and encoding.Decoder interfaces for INI encoding.
type Codec struct {
	KeyDelimiter string
	LoadOptions  LoadOptions
}

func (c Codec) Encode(v map[string]interface{}) ([]byte, error) {
	cfg := ini.Empty()
	ini.PrettyFormat = false

	flattened := map[string]interface{}{}

	flattened = flattenAndMergeMap(flattened, v, "", c.keyDelimiter())

	keys := make([]string, 0, len(flattened))

	for key := range flattened {
		keys = append(keys, key)
	}

	sort.Strings(keys)

	for _, key := range keys {
		sectionName, keyName := "", key

		lastSep := strings.LastIndex(key, ".")
		if lastSep != -1 {
			sectionName = key[:(lastSep)]
			keyName = key[(lastSep + 1):]
		}

		// TODO: is this a good idea?
		if sectionName == "default" {
			sectionName = ""
		}

		cfg.Section(sectionName).Key(keyName).SetValue(cast.ToString(flattened[key]))
	}

	var buf bytes.Buffer

	_, err := cfg.WriteTo(&buf)
	if err != nil {
		return nil, err
	}

	return buf.Bytes(), nil
}

func (c Codec) Decode(b []byte, v map[string]interface{}) error {
	cfg := ini.Empty(c.LoadOptions)

	err := cfg.Append(b)
	if err != nil {
		return err
	}

	sections := cfg.Sections()

	for i := 0; i < len(sections); i++ {
		section := sections[i]
		keys := section.Keys()

		for j := 0; j < len(keys); j++ {
			key := keys[j]
			value := cfg.Section(section.Name()).Key(key.Name()).String()

			deepestMap := deepSearch(v, strings.Split(section.Name(), c.keyDelimiter()))

			// set innermost value
			deepestMap[key.Name()] = value
		}
	}

	return nil
}

func (c Codec) keyDelimiter() string {
	if c.KeyDelimiter == "" {
		return "."
	}

	return c.KeyDelimiter
}
