package aminojson

import (
	"encoding/base64"
	"encoding/json"
	"fmt"
	"io"
	"sort"

	"github.com/pkg/errors"
	"google.golang.org/protobuf/reflect/protoreflect"

	authapi "cosmossdk.io/api/cosmos/auth/v1beta1"
	"cosmossdk.io/api/cosmos/crypto/multisig"
	"cosmossdk.io/math"
)

// cosmosIntEncoder provides legacy compatible encoding for cosmos.Int types. In gogo messages these are sometimes
// represented by a `cosmos-sdk/types.Int` through the usage of the option:
//
//	(gogoproto.customtype) = "github.com/cosmos/cosmos-sdk/types.Int"
//
// In pulsar message they represented as strings, which is the only format this encoder supports.
func cosmosIntEncoder(_ *Encoder, v protoreflect.Value, w io.Writer) error {
	switch val := v.Interface().(type) {
	case string:
		if val == "" {
			return jsonMarshal(w, "0")
		}
		return jsonMarshal(w, val)
	case []byte:
		if len(val) == 0 {
			return jsonMarshal(w, "0")
		}
		var i math.Int
		err := i.Unmarshal(val)
		if err != nil {
			return err
		}
		return jsonMarshal(w, i.String())
	default:
		return fmt.Errorf("unsupported type %T", val)
	}
}

// cosmosDecEncoder provides legacy compatible encoding for cosmos.Dec and cosmos.Int types. These are sometimes
// represented as strings in pulsar messages and sometimes as bytes.  This encoder handles both cases.
func cosmosDecEncoder(_ *Encoder, v protoreflect.Value, w io.Writer) error {
	switch val := v.Interface().(type) {
	case string:
		if val == "" {
			return jsonMarshal(w, "0")
		}
		return jsonMarshal(w, val)
	case []byte:
		if len(val) == 0 {
			return jsonMarshal(w, "0")
		}
		var dec math.LegacyDec
		err := dec.Unmarshal(val)
		if err != nil {
			return err
		}
		return jsonMarshal(w, dec.String())
	default:
		return fmt.Errorf("unsupported type %T", val)
	}
}

// nullSliceAsEmptyEncoder replicates the behavior at:
// https://github.com/cosmos/cosmos-sdk/blob/be9bd7a8c1b41b115d58f4e76ee358e18a52c0af/types/coin.go#L199-L205
func nullSliceAsEmptyEncoder(enc *Encoder, v protoreflect.Value, w io.Writer) error {
	switch list := v.Interface().(type) {
	case protoreflect.List:
		if list.Len() == 0 {
			_, err := io.WriteString(w, "[]")
			return err
		}
		return enc.marshalList(list, nil /* no field descriptor available here */, w)
	default:
		return fmt.Errorf("unsupported type %T", list)
	}
}

// cosmosInlineJSON takes bytes and inlines them into a JSON document.
//
// This requires the bytes contain valid JSON since otherwise the resulting document would be invalid.
// Invalid JSON will result in an error.
//
// This replicates the behavior of JSON messages embedded in protobuf bytes
// required for CosmWasm, e.g.:
// https://github.com/CosmWasm/wasmd/blob/08567ff20e372e4f4204a91ca64a371538742bed/x/wasm/types/tx.go#L20-L22
func cosmosInlineJSON(_ *Encoder, v protoreflect.Value, w io.Writer) error {
	switch bz := v.Interface().(type) {
	case []byte:
		json, err := sortedJsonStringify(bz)
		if err != nil {
			return errors.Wrap(err, "could not normalize JSON")
		}
		_, err = w.Write(json)
		return err
	default:
		return fmt.Errorf("unsupported type %T", bz)
	}
}

// keyFieldEncoder replicates the behavior at described at:
// https://github.com/cosmos/cosmos-sdk/blob/b49f948b36bc991db5be431607b475633aed697e/proto/cosmos/crypto/secp256k1/keys.proto#L16
// The message is treated if it were bytes directly without the key field specified.
func keyFieldEncoder(_ *Encoder, msg protoreflect.Message, w io.Writer) error {
	keyField := msg.Descriptor().Fields().ByName("key")
	if keyField == nil {
		return errors.New(`message encoder for key_field: no field named "key" found`)
	}

	bz := msg.Get(keyField).Bytes()

	if len(bz) == 0 {
		_, err := fmt.Fprint(w, "null")
		return err
	}

	_, err := fmt.Fprintf(w, `"%s"`, base64.StdEncoding.EncodeToString(bz))
	return err
}

type moduleAccountPretty struct {
	Address       string   `json:"address"`
	PubKey        string   `json:"public_key"`
	AccountNumber uint64   `json:"account_number"`
	Sequence      uint64   `json:"sequence"`
	Name          string   `json:"name"`
	Permissions   []string `json:"permissions"`
}

// moduleAccountEncoder replicates the behavior in
// https://github.com/cosmos/cosmos-sdk/blob/41a3dfeced2953beba3a7d11ec798d17ee19f506/x/auth/types/account.go#L230-L254
func moduleAccountEncoder(_ *Encoder, msg protoreflect.Message, w io.Writer) error {
	ma := msg.Interface().(*authapi.ModuleAccount)
	pretty := moduleAccountPretty{
		PubKey:      "",
		Name:        ma.Name,
		Permissions: ma.Permissions,
	}
	if ma.BaseAccount != nil {
		pretty.Address = ma.BaseAccount.Address
		pretty.AccountNumber = ma.BaseAccount.AccountNumber
		pretty.Sequence = ma.BaseAccount.Sequence
	} else {
		pretty.Address = ""
		pretty.AccountNumber = 0
		pretty.Sequence = 0
	}

	// we do not want to use the json encoder here because it adds a newline
	bz, err := json.Marshal(pretty)
	if err != nil {
		return err
	}
	_, err = w.Write(bz)
	return err
}

// thresholdStringEncoder replicates the behavior at:
// https://github.com/cosmos/cosmos-sdk/blob/4a6a1e3cb8de459891cb0495052589673d14ef51/crypto/keys/multisig/amino.go#L35
// also see:
// https://github.com/cosmos/cosmos-sdk/blob/b49f948b36bc991db5be431607b475633aed697e/proto/cosmos/crypto/multisig/keys.proto#L15/
func thresholdStringEncoder(enc *Encoder, msg protoreflect.Message, w io.Writer) error {
	pk, ok := msg.Interface().(*multisig.LegacyAminoPubKey)
	if !ok {
		return errors.New("thresholdStringEncoder: msg not a multisig.LegacyAminoPubKey")
	}
	_, err := fmt.Fprintf(w, `{"threshold":"%d","pubkeys":`, pk.Threshold)
	if err != nil {
		return err
	}

	if len(pk.PublicKeys) == 0 {
		_, err = io.WriteString(w, `[]}`)
		return err
	}

	fields := msg.Descriptor().Fields()
	pubkeysField := fields.ByName("public_keys")
	pubkeys := msg.Get(pubkeysField).List()

	err = enc.marshalList(pubkeys, pubkeysField, w)
	if err != nil {
		return err
	}
	_, err = io.WriteString(w, `}`)
	return err
}

// sortedObject returns a new object that mirrors the structure of the original
// but with all maps having their keys sorted.
func sortedObject(obj interface{}) interface{} {
	switch v := obj.(type) {
	case map[string]interface{}:
		sortedKeys := make([]string, 0, len(v))
		for key := range v {
			sortedKeys = append(sortedKeys, key)
		}
		sort.Strings(sortedKeys)
		result := make(map[string]interface{})
		for _, key := range sortedKeys {
			result[key] = sortedObject(v[key])
		}
		return result
	case []interface{}:
		for i, val := range v {
			v[i] = sortedObject(val)
		}
		return v
	default:
		return obj
	}
}

// sortedJsonStringify returns a JSON with objects sorted by key.
func sortedJsonStringify(jsonBytes []byte) ([]byte, error) {
	var obj interface{}
	if err := json.Unmarshal(jsonBytes, &obj); err != nil {
		return nil, errors.New("invalid JSON bytes")
	}
	sorted := sortedObject(obj)
	jsonData, err := json.Marshal(sorted)
	if err != nil {
		return nil, err
	}
	return jsonData, nil
}
