package graffiti

import (
	"os"
	"path/filepath"
	"testing"

	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/crypto/hash"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func TestParseGraffitiFile_Default(t *testing.T) {
	input := []byte(`default: "Mr T was here"`)

	dirName := t.TempDir() + "somedir"
	err := os.MkdirAll(dirName, os.ModePerm)
	require.NoError(t, err)
	someFileName := filepath.Join(dirName, "somefile.txt")
	require.NoError(t, os.WriteFile(someFileName, input, os.ModePerm))

	got, err := ParseGraffitiFile(someFileName)
	require.NoError(t, err)

	wanted := &Graffiti{
		Hash:    hash.Hash(input),
		Default: "Mr T was here",
	}
	require.DeepEqual(t, wanted, got)
}

func TestParseGraffitiFile_Random(t *testing.T) {
	input := []byte(`random:
  - "Mr A was here"
  - "Mr B was here"
  - "Mr C was here"`)

	dirName := t.TempDir() + "somedir"
	err := os.MkdirAll(dirName, os.ModePerm)
	require.NoError(t, err)
	someFileName := filepath.Join(dirName, "somefile.txt")
	require.NoError(t, os.WriteFile(someFileName, input, os.ModePerm))

	got, err := ParseGraffitiFile(someFileName)
	require.NoError(t, err)

	wanted := &Graffiti{
		Hash: hash.Hash(input),
		Random: []string{
			"Mr A was here",
			"Mr B was here",
			"Mr C was here",
		},
	}
	require.DeepEqual(t, wanted, got)
}

func TestParseGraffitiFile_Ordered(t *testing.T) {
	input := []byte(`ordered:
  - "Mr D was here"
  - "Mr E was here"
  - "Mr F was here"`)

	dirName := t.TempDir() + "somedir"
	err := os.MkdirAll(dirName, os.ModePerm)
	require.NoError(t, err)
	someFileName := filepath.Join(dirName, "somefile.txt")
	require.NoError(t, os.WriteFile(someFileName, input, os.ModePerm))

	got, err := ParseGraffitiFile(someFileName)
	require.NoError(t, err)

	wanted := &Graffiti{
		Hash: hash.Hash(input),
		Ordered: []string{
			"Mr D was here",
			"Mr E was here",
			"Mr F was here",
		},
	}
	require.DeepEqual(t, wanted, got)
}

func TestParseGraffitiFile_Validators(t *testing.T) {
	input := []byte(`
specific:
  1234: Yolo
  555: "What's up"
  703727: Meow`)

	dirName := t.TempDir() + "somedir"
	err := os.MkdirAll(dirName, os.ModePerm)
	require.NoError(t, err)
	someFileName := filepath.Join(dirName, "somefile.txt")
	require.NoError(t, os.WriteFile(someFileName, input, os.ModePerm))

	got, err := ParseGraffitiFile(someFileName)
	require.NoError(t, err)

	wanted := &Graffiti{
		Hash: hash.Hash(input),
		Specific: map[primitives.ValidatorIndex]string{
			1234:   "Yolo",
			555:    "What's up",
			703727: "Meow",
		},
	}
	require.DeepEqual(t, wanted, got)
}

func TestParseGraffitiFile_AllFields(t *testing.T) {
	input := []byte(`default: "Mr T was here"

random:
  - "Mr A was here"
  - "Mr B was here"
  - "Mr C was here"

ordered:
  - "Mr D was here"
  - "Mr E was here"
  - "Mr F was here"

specific:
  1234: Yolo
  555: "What's up"
  703727: Meow`)

	dirName := t.TempDir() + "somedir"
	err := os.MkdirAll(dirName, os.ModePerm)
	require.NoError(t, err)
	someFileName := filepath.Join(dirName, "somefile.txt")
	require.NoError(t, os.WriteFile(someFileName, input, os.ModePerm))

	got, err := ParseGraffitiFile(someFileName)
	require.NoError(t, err)

	wanted := &Graffiti{
		Hash:    hash.Hash(input),
		Default: "Mr T was here",
		Random: []string{
			"Mr A was here",
			"Mr B was here",
			"Mr C was here",
		},
		Ordered: []string{
			"Mr D was here",
			"Mr E was here",
			"Mr F was here",
		},
		Specific: map[primitives.ValidatorIndex]string{
			1234:   "Yolo",
			555:    "What's up",
			703727: "Meow",
		},
	}
	require.DeepEqual(t, wanted, got)
}

func TestParseHexGraffiti(t *testing.T) {
	tests := []struct {
		name  string
		want  string
		input string
	}{
		{
			name:  "standard",
			want:  "hola mundo!",
			input: "hola mundo!",
		},
		{
			name:  "standard with hex tag",
			want:  "hola mundo!",
			input: "hex:686f6c61206d756e646f21",
		},
		{
			name:  "irregularly cased hex tag",
			want:  "hola mundo!",
			input: "HEX:686f6c61206d756e646f21",
		},
		{
			name:  "hex tag without accompanying data",
			want:  "hex:",
			input: "hex:",
		},
		{
			name:  "Passing non-hex data with hex tag",
			want:  "hex:hola mundo!",
			input: "hex:hola mundo!",
		},
		{
			name:  "unmarked hex input",
			want:  "0x686f6c61206d756e646f21",
			input: "0x686f6c61206d756e646f21",
		},
		{
			name:  "Properly tagged hex data with 0x prefix",
			want:  "hola mundo!",
			input: "hex:0x686f6c61206d756e646f21",
		},
		{
			name:  "hex tag with 0x prefix and no other data",
			want:  "hex:0x",
			input: "hex:0x",
		},
		{
			name:  "hex tag with 0x prefix and invalid hex data",
			want:  "hex:0xhola mundo",
			input: "hex:0xhola mundo",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			out := ParseHexGraffiti(tt.input)
			assert.Equal(t, out, tt.want)
		})
	}
}
