package benchmark_test

import (
	"compress/gzip"
	"encoding/json"
	"io/ioutil"
	"os"
	"path/filepath"
	"testing"

	"github.com/pelletier/go-toml/v2"
	"github.com/stretchr/testify/require"
)

var bench_inputs = []struct {
	name    string
	jsonLen int
}{
	// from https://gist.githubusercontent.com/feeeper/2197d6d734729625a037af1df14cf2aa/raw/2f22b120e476d897179be3c1e2483d18067aa7df/config.toml
	{"config", 806507},

	// converted from https://github.com/miloyip/nativejson-benchmark
	{"canada", 2090234},
	{"citm_catalog", 479897},
	{"twitter", 428778},
	{"code", 1940472},

	// converted from https://raw.githubusercontent.com/mailru/easyjson/master/benchmark/example.json
	{"example", 7779},
}

func TestUnmarshalDatasetCode(t *testing.T) {
	for _, tc := range bench_inputs {
		t.Run(tc.name, func(t *testing.T) {
			buf := fixture(t, tc.name)

			var v interface{}
			require.NoError(t, toml.Unmarshal(buf, &v))

			b, err := json.Marshal(v)
			require.NoError(t, err)
			require.Equal(t, len(b), tc.jsonLen)
		})
	}
}

func BenchmarkUnmarshalDataset(b *testing.B) {
	for _, tc := range bench_inputs {
		b.Run(tc.name, func(b *testing.B) {
			buf := fixture(b, tc.name)
			b.SetBytes(int64(len(buf)))
			b.ReportAllocs()
			b.ResetTimer()
			for i := 0; i < b.N; i++ {
				var v interface{}
				require.NoError(b, toml.Unmarshal(buf, &v))
			}
		})
	}
}

// fixture returns the uncompressed contents of path.
func fixture(tb testing.TB, path string) []byte {
	tb.Helper()

	file := path + ".toml.gz"
	f, err := os.Open(filepath.Join("testdata", file))
	if os.IsNotExist(err) {
		tb.Skip("benchmark fixture not found:", file)
	}
	require.NoError(tb, err)
	defer f.Close()

	gz, err := gzip.NewReader(f)
	require.NoError(tb, err)

	buf, err := ioutil.ReadAll(gz)
	require.NoError(tb, err)
	return buf
}
