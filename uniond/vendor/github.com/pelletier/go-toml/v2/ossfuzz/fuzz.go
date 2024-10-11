package ossfuzz

import (
	"fmt"
	"reflect"
	"strings"

	"github.com/pelletier/go-toml/v2"
)

func FuzzToml(data []byte) int {
	if len(data) >= 2048 {
		return 0
	}

	if strings.Contains(string(data), "nan") {
		return 0
	}

	var v interface{}
	err := toml.Unmarshal(data, &v)
	if err != nil {
		return 0
	}

	encoded, err := toml.Marshal(v)
	if err != nil {
		panic(fmt.Sprintf("failed to marshal unmarshaled document: %s", err))
	}

	var v2 interface{}
	err = toml.Unmarshal(encoded, &v2)
	if err != nil {
		panic(fmt.Sprintf("failed round trip: %s", err))
	}

	if !reflect.DeepEqual(v, v2) {
		panic(fmt.Sprintf("not equal: %#+v %#+v", v, v2))
	}

	return 1
}
