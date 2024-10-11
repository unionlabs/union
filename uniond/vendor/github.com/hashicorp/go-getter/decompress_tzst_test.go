package getter

import (
	"path/filepath"
	"testing"
)

func TestTarZstdDecompressor(t *testing.T) {

	multiplePaths := []string{"dir/", "dir/test2", "test1"}
	orderingPaths := []string{"workers/", "workers/mq/", "workers/mq/__init__.py"}

	cases := []TestDecompressCase{
		{
			"empty.tar.zst",
			false,
			true,
			nil,
			"",
			nil,
		},

		{
			"single.tar.zst",
			false,
			false,
			nil,
			"d3b07384d113edec49eaa6238ad5ff00",
			nil,
		},

		{
			"single.tar.zst",
			true,
			false,
			[]string{"file"},
			"",
			nil,
		},

		{
			"multiple.tar.zst",
			true,
			false,
			[]string{"file1", "file2"},
			"",
			nil,
		},

		{
			"multiple.tar.zst",
			false,
			true,
			nil,
			"",
			nil,
		},

		{
			"multiple_dir.tar.zst",
			true,
			false,
			multiplePaths,
			"",
			nil,
		},

		// Tests when the file is listed before the parent folder
		{
			"ordering.tar.zst",
			true,
			false,
			orderingPaths,
			"",
			nil,
		},

		// Tests that a tar.zst can't contain references with "..".
		// GNU `tar` also disallows this.
		{
			"outside_parent.tar.zst",
			true,
			true,
			nil,
			"",
			nil,
		},
	}

	for i, tc := range cases {
		cases[i].Input = filepath.Join("./testdata", "decompress-tzst", tc.Input)
	}

	TestDecompressor(t, new(TarZstdDecompressor), cases)
}
