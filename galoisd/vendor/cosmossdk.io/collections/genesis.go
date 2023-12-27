package collections

import (
	"context"
	"encoding/json"
	"fmt"
	"io"
)

type genesisHandler interface {
	validateGenesis(r io.Reader) error
	importGenesis(ctx context.Context, r io.Reader) error
	exportGenesis(ctx context.Context, w io.Writer) error
	defaultGenesis(w io.Writer) error
}

type jsonMapEntry struct {
	Key   json.RawMessage `json:"key"`
	Value json.RawMessage `json:"value,omitempty"`
}

func (m Map[K, V]) validateGenesis(reader io.Reader) error {
	return m.doDecodeJSON(reader, func(key K, value V) error {
		return nil
	})
}

func (m Map[K, V]) importGenesis(ctx context.Context, reader io.Reader) error {
	return m.doDecodeJSON(reader, func(key K, value V) error {
		return m.Set(ctx, key, value)
	})
}

func (m Map[K, V]) exportGenesis(ctx context.Context, writer io.Writer) error {
	_, err := io.WriteString(writer, "[")
	if err != nil {
		return err
	}

	it, err := m.Iterate(ctx, nil)
	if err != nil {
		return err
	}
	defer it.Close()

	first := true
	for ; it.Valid(); it.Next() {
		// add a comma before encoding the object
		// for all objects besides the first one.
		if !first {
			_, err = io.WriteString(writer, ",")
			if err != nil {
				return err
			}
		}
		first = false

		key, err := it.Key()
		if err != nil {
			return err
		}

		keyBz, err := m.kc.EncodeJSON(key)
		if err != nil {
			return err
		}

		value, err := it.Value()
		if err != nil {
			return err
		}

		valueBz, err := m.vc.EncodeJSON(value)
		if err != nil {
			return err
		}

		entry := jsonMapEntry{
			Key:   keyBz,
			Value: valueBz,
		}

		bz, err := json.Marshal(entry)
		if err != nil {
			return err
		}

		_, err = writer.Write(bz)
		if err != nil {
			return err
		}
	}

	_, err = io.WriteString(writer, "]")
	return err
}

func (m Map[K, V]) doDecodeJSON(reader io.Reader, onEntry func(key K, value V) error) error {
	decoder := json.NewDecoder(reader)
	token, err := decoder.Token()
	if err != nil {
		return err
	}

	if token != json.Delim('[') {
		return fmt.Errorf("expected [ got %s", token)
	}

	for decoder.More() {
		var rawJSON json.RawMessage
		err := decoder.Decode(&rawJSON)
		if err != nil {
			return err
		}

		var mapEntry jsonMapEntry
		err = json.Unmarshal(rawJSON, &mapEntry)
		if err != nil {
			return err
		}

		key, err := m.kc.DecodeJSON(mapEntry.Key)
		if err != nil {
			return err
		}

		value, err := m.vc.DecodeJSON(mapEntry.Value)
		if err != nil {
			return err
		}

		err = onEntry(key, value)
		if err != nil {
			return err
		}
	}

	token, err = decoder.Token()
	if err != nil {
		return err
	}

	if token != json.Delim(']') {
		return fmt.Errorf("expected ] got %s", token)
	}

	return nil
}

func (m Map[K, V]) defaultGenesis(writer io.Writer) error {
	_, err := io.WriteString(writer, `[]`)
	return err
}
