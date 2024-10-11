package bech32

import (
	"crypto/sha256"
	"encoding/json"
	"testing"
)

type hrpAndData struct {
	HRP  string `json:"h"`
	Data []byte `json:"d"`
}

func FuzzConvertAndEncode(f *testing.F) {
	// 1. Add seeds
	seeds := []*hrpAndData{
		{
			"shasum",
			func() []byte {
				sum := sha256.Sum256([]byte("hello world\n"))
				return sum[:]
			}(),
		},
		{
			"shasum",
			[]byte("49yfqne0parehrupja55kvqcfvxja5wpe54pas8mshffngvj53rs93fk75"),
		},
		{
			"bech32",
			[]byte("er8m900ayvv9rg5r6ush4nzvqhj4p9tqnxqkxaaxrs4ueuvhurcs4x3j4j"),
		},
	}

	for _, seed := range seeds {
		seedJSONBlob, err := json.Marshal(seed)
		if err != nil {
			continue
		}
		f.Add(seedJSONBlob)
	}

	// 2. Now run the fuzzer.
	f.Fuzz(func(t *testing.T, inputJSON []byte) {
		had := new(hrpAndData)
		if err := json.Unmarshal(inputJSON, had); err != nil {
			return
		}
		_, _ = ConvertAndEncode(had.HRP, had.Data)
	})
}
