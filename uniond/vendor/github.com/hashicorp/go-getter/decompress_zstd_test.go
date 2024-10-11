package getter

import (
	"path/filepath"
	"testing"
)

func TestZstdDecompressor(t *testing.T) {
	cases := []TestDecompressCase{
		{
			"single.zst",
			false,
			false,
			nil,
			"d3b07384d113edec49eaa6238ad5ff00",
			nil,
		},

		{
			"single.zst",
			true,
			true,
			nil,
			"",
			nil,
		},
	}

	for i, tc := range cases {
		cases[i].Input = filepath.Join("./testdata", "decompress-zst", tc.Input)
	}

	TestDecompressor(t, new(ZstdDecompressor), cases)
}
