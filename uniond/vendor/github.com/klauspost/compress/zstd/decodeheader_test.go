package zstd

import (
	"bytes"
	"encoding/json"
	"io"
	"os"
	"strings"
	"testing"
)

func TestHeader_Decode(t *testing.T) {
	zr := testCreateZipReader("testdata/headers.zip", t)

	// Regenerate golden data...
	const regen = false
	golden := make(map[string]Header)
	if !regen {
		b, err := os.ReadFile("testdata/headers-want.json.zst")
		if err != nil {
			t.Fatal(err)
		}
		dec, err := NewReader(nil)
		if err != nil {
			t.Fatal(err)
		}
		defer dec.Close()
		b, err = dec.DecodeAll(b, nil)
		if err != nil {
			t.Fatal(err)
		}
		err = json.Unmarshal(b, &golden)
		if err != nil {
			t.Fatal(err)
		}
	}

	for i, tt := range zr.File {
		if !strings.HasSuffix(t.Name(), "") {
			continue
		}
		if testing.Short() && i > 100 {
			break
		}

		t.Run(tt.Name, func(t *testing.T) {
			r, err := tt.Open()
			if err != nil {
				t.Error(err)
				return
			}
			defer r.Close()
			b, err := io.ReadAll(r)
			if err != nil {
				t.Error(err)
				return
			}
			want, ok := golden[tt.Name]
			var got Header
			err = got.Decode(b)
			if err != nil {
				if ok {
					t.Errorf("got unexpected error %v", err)
				}
				return
			}
			if regen {
				// errored entries are not set
				golden[tt.Name] = got
				return
			}
			if !ok {
				t.Errorf("want error, got result: %v", got)
			}
			if want != got {
				t.Errorf("header mismatch:\nwant %#v\ngot  %#v", want, got)
			}
		})
	}
	if regen {
		w, err := os.Create("testdata/headers-want.json.zst")
		if err != nil {
			t.Fatal(err)
		}
		defer w.Close()
		enc, err := NewWriter(w, WithEncoderLevel(SpeedBestCompression))
		if err != nil {
			t.Fatal(err)
		}
		b, err := json.Marshal(golden)
		if err != nil {
			t.Fatal(err)
		}
		enc.ReadFrom(bytes.NewBuffer(b))
		enc.Close()
		t.SkipNow()
	}
}
