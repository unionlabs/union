package ssz

import (
	"encoding/hex"
	"fmt"
	"reflect"
	"strings"

	"github.com/mitchellh/mapstructure"
	"gopkg.in/yaml.v2"
)

func isByteSlice(t reflect.Type) bool {
	return t.Kind() == reflect.Slice && t.Elem().Kind() == reflect.Uint8
}

func isByteArray(t reflect.Type) bool {
	return t.Kind() == reflect.Array && t.Elem().Kind() == reflect.Uint8
}

func customHook(f reflect.Type, t reflect.Type, data interface{}) (interface{}, error) {
	if f.Kind() != reflect.String {
		return data, nil
	}

	raw := data.(string)
	if !strings.HasPrefix(raw, "0x") {
		return nil, fmt.Errorf("0x prefix not found")
	}
	elem, err := hex.DecodeString(raw[2:])
	if err != nil {
		return nil, err
	}
	if isByteSlice(t) {
		// []byte
		return elem, nil
	}
	if isByteArray(t) {
		// [n]byte
		if t.Len() != len(elem) {
			return nil, fmt.Errorf("incorrect array length: %d %d", t.Len(), len(elem))
		}

		v := reflect.New(t)
		reflect.Copy(v.Elem(), reflect.ValueOf(elem))
		return v.Interface(), nil
	}

	var v reflect.Value
	if t.Kind() == reflect.Ptr {
		v = reflect.New(t.Elem())
	} else {
		v = reflect.New(t)
	}
	if vv, ok := v.Interface().(Unmarshaler); ok {
		if err := vv.UnmarshalSSZ(elem); err != nil {
			return nil, err
		}
		return vv, nil
	}
	return nil, fmt.Errorf("type not found")
}

func UnmarshalSSZTest(content []byte, result interface{}) error {
	var source map[string]interface{}
	if err := yaml.Unmarshal(content, &source); err != nil {
		return err
	}

	dc := &mapstructure.DecoderConfig{
		Result:     result,
		DecodeHook: customHook,
		TagName:    "json",
	}
	ms, err := mapstructure.NewDecoder(dc)
	if err != nil {
		return err
	}
	if err = ms.Decode(source); err != nil {
		return err
	}
	return nil
}
