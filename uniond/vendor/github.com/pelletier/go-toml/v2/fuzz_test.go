package toml_test

import (
	"io/ioutil"
	"strings"
	"testing"

	"github.com/pelletier/go-toml/v2"
	"github.com/stretchr/testify/require"
)

func FuzzUnmarshal(f *testing.F) {
	file, err := ioutil.ReadFile("benchmark/benchmark.toml")
	if err != nil {
		panic(err)
	}
	f.Add(file)

	f.Fuzz(func(t *testing.T, b []byte) {
		if strings.Contains(string(b), "nan") {
			// Current limitation of testify.
			// https://github.com/stretchr/testify/issues/624
			t.Skip("can't compare NaNs")
		}

		t.Log("INITIAL DOCUMENT ===========================")
		t.Log(string(b))

		var v interface{}
		err := toml.Unmarshal(b, &v)
		if err != nil {
			return
		}

		t.Log("DECODED VALUE ===========================")
		t.Logf("%#+v", v)

		encoded, err := toml.Marshal(v)
		if err != nil {
			t.Fatalf("cannot marshal unmarshaled document: %s", err)
		}

		t.Log("ENCODED DOCUMENT ===========================")
		t.Log(string(encoded))

		var v2 interface{}
		err = toml.Unmarshal(encoded, &v2)
		if err != nil {
			t.Fatalf("failed round trip: %s", err)
		}
		require.Equal(t, v, v2)
	})
}
