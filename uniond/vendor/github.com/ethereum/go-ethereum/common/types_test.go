// Copyright 2015 The go-ethereum Authors
// This file is part of the go-ethereum library.
//
// The go-ethereum library is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// The go-ethereum library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with the go-ethereum library. If not, see <http://www.gnu.org/licenses/>.

package common

import (
	"bytes"
	"database/sql/driver"
	"encoding/json"
	"fmt"
	"math/big"
	"reflect"
	"strings"
	"testing"
	"time"
)

func TestBytesConversion(t *testing.T) {
	bytes := []byte{5}
	hash := BytesToHash(bytes)

	var exp Hash
	exp[31] = 5

	if hash != exp {
		t.Errorf("expected %x got %x", exp, hash)
	}
}

func TestIsHexAddress(t *testing.T) {
	tests := []struct {
		str string
		exp bool
	}{
		{"0x5aaeb6053f3e94c9b9a09f33669435e7ef1beaed", true},
		{"5aaeb6053f3e94c9b9a09f33669435e7ef1beaed", true},
		{"0X5aaeb6053f3e94c9b9a09f33669435e7ef1beaed", true},
		{"0XAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA", true},
		{"0xAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA", true},
		{"0x5aaeb6053f3e94c9b9a09f33669435e7ef1beaed1", false},
		{"0x5aaeb6053f3e94c9b9a09f33669435e7ef1beae", false},
		{"5aaeb6053f3e94c9b9a09f33669435e7ef1beaed11", false},
		{"0xxaaeb6053f3e94c9b9a09f33669435e7ef1beaed", false},
	}

	for _, test := range tests {
		if result := IsHexAddress(test.str); result != test.exp {
			t.Errorf("IsHexAddress(%s) == %v; expected %v",
				test.str, result, test.exp)
		}
	}
}

func TestHashJsonValidation(t *testing.T) {
	var tests = []struct {
		Prefix string
		Size   int
		Error  string
	}{
		{"", 62, "json: cannot unmarshal hex string without 0x prefix into Go value of type common.Hash"},
		{"0x", 66, "hex string has length 66, want 64 for common.Hash"},
		{"0x", 63, "json: cannot unmarshal hex string of odd length into Go value of type common.Hash"},
		{"0x", 0, "hex string has length 0, want 64 for common.Hash"},
		{"0x", 64, ""},
		{"0X", 64, ""},
	}
	for _, test := range tests {
		input := `"` + test.Prefix + strings.Repeat("0", test.Size) + `"`
		var v Hash
		err := json.Unmarshal([]byte(input), &v)
		if err == nil {
			if test.Error != "" {
				t.Errorf("%s: error mismatch: have nil, want %q", input, test.Error)
			}
		} else {
			if err.Error() != test.Error {
				t.Errorf("%s: error mismatch: have %q, want %q", input, err, test.Error)
			}
		}
	}
}

func TestAddressUnmarshalJSON(t *testing.T) {
	var tests = []struct {
		Input     string
		ShouldErr bool
		Output    *big.Int
	}{
		{"", true, nil},
		{`""`, true, nil},
		{`"0x"`, true, nil},
		{`"0x00"`, true, nil},
		{`"0xG000000000000000000000000000000000000000"`, true, nil},
		{`"0x0000000000000000000000000000000000000000"`, false, big.NewInt(0)},
		{`"0x0000000000000000000000000000000000000010"`, false, big.NewInt(16)},
	}
	for i, test := range tests {
		var v Address
		err := json.Unmarshal([]byte(test.Input), &v)
		if err != nil && !test.ShouldErr {
			t.Errorf("test #%d: unexpected error: %v", i, err)
		}
		if err == nil {
			if test.ShouldErr {
				t.Errorf("test #%d: expected error, got none", i)
			}
			if got := new(big.Int).SetBytes(v.Bytes()); got.Cmp(test.Output) != 0 {
				t.Errorf("test #%d: address mismatch: have %v, want %v", i, got, test.Output)
			}
		}
	}
}

func TestAddressHexChecksum(t *testing.T) {
	var tests = []struct {
		Input  string
		Output string
	}{
		// Test cases from https://github.com/ethereum/EIPs/blob/master/EIPS/eip-55.md#specification
		{"0x5aaeb6053f3e94c9b9a09f33669435e7ef1beaed", "0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAed"},
		{"0xfb6916095ca1df60bb79ce92ce3ea74c37c5d359", "0xfB6916095ca1df60bB79Ce92cE3Ea74c37c5d359"},
		{"0xdbf03b407c01e7cd3cbea99509d93f8dddc8c6fb", "0xdbF03B407c01E7cD3CBea99509d93f8DDDC8C6FB"},
		{"0xd1220a0cf47c7b9be7a2e6ba89f429762e7b9adb", "0xD1220A0cf47c7B9Be7A2E6BA89F429762e7b9aDb"},
		// Ensure that non-standard length input values are handled correctly
		{"0xa", "0x000000000000000000000000000000000000000A"},
		{"0x0a", "0x000000000000000000000000000000000000000A"},
		{"0x00a", "0x000000000000000000000000000000000000000A"},
		{"0x000000000000000000000000000000000000000a", "0x000000000000000000000000000000000000000A"},
	}
	for i, test := range tests {
		output := HexToAddress(test.Input).Hex()
		if output != test.Output {
			t.Errorf("test #%d: failed to match when it should (%s != %s)", i, output, test.Output)
		}
	}
}

func BenchmarkAddressHex(b *testing.B) {
	testAddr := HexToAddress("0x5aaeb6053f3e94c9b9a09f33669435e7ef1beaed")
	for n := 0; n < b.N; n++ {
		testAddr.Hex()
	}
}

// Test checks if the customized json marshaller of MixedcaseAddress object
// is invoked correctly. In golang the struct pointer will inherit the
// non-pointer receiver methods, the reverse is not true. In the case of
// MixedcaseAddress, it must define the MarshalJSON method in the object
// but not the pointer level, so that this customized marshalled can be used
// for both MixedcaseAddress object and pointer.
func TestMixedcaseAddressMarshal(t *testing.T) {
	var (
		output string
		input  = "0xae967917c465db8578ca9024c205720b1a3651A9"
	)
	addr, err := NewMixedcaseAddressFromString(input)
	if err != nil {
		t.Fatal(err)
	}
	blob, err := json.Marshal(*addr)
	if err != nil {
		t.Fatal(err)
	}
	json.Unmarshal(blob, &output)
	if output != input {
		t.Fatal("Failed to marshal/unmarshal MixedcaseAddress object")
	}
}

func TestMixedcaseAccount_Address(t *testing.T) {
	// https://github.com/ethereum/EIPs/blob/master/EIPS/eip-55.md
	// Note: 0X{checksum_addr} is not valid according to spec above

	var res []struct {
		A     MixedcaseAddress
		Valid bool
	}
	if err := json.Unmarshal([]byte(`[
		{"A" : "0xae967917c465db8578ca9024c205720b1a3651A9", "Valid": false},
		{"A" : "0xAe967917c465db8578ca9024c205720b1a3651A9", "Valid": true},
		{"A" : "0XAe967917c465db8578ca9024c205720b1a3651A9", "Valid": false},
		{"A" : "0x1111111111111111111112222222222223333323", "Valid": true}
		]`), &res); err != nil {
		t.Fatal(err)
	}

	for _, r := range res {
		if got := r.A.ValidChecksum(); got != r.Valid {
			t.Errorf("Expected checksum %v, got checksum %v, input %v", r.Valid, got, r.A.String())
		}
	}

	// These should throw exceptions:
	var r2 []MixedcaseAddress
	for _, r := range []string{
		`["0x11111111111111111111122222222222233333"]`,     // Too short
		`["0x111111111111111111111222222222222333332"]`,    // Too short
		`["0x11111111111111111111122222222222233333234"]`,  // Too long
		`["0x111111111111111111111222222222222333332344"]`, // Too long
		`["1111111111111111111112222222222223333323"]`,     // Missing 0x
		`["x1111111111111111111112222222222223333323"]`,    // Missing 0
		`["0xG111111111111111111112222222222223333323"]`,   //Non-hex
	} {
		if err := json.Unmarshal([]byte(r), &r2); err == nil {
			t.Errorf("Expected failure, input %v", r)
		}
	}
}

func TestHash_Scan(t *testing.T) {
	type args struct {
		src interface{}
	}
	tests := []struct {
		name    string
		args    args
		wantErr bool
	}{
		{
			name: "working scan",
			args: args{src: []byte{
				0xb2, 0x6f, 0x2b, 0x34, 0x2a, 0xab, 0x24, 0xbc, 0xf6, 0x3e,
				0xa2, 0x18, 0xc6, 0xa9, 0x27, 0x4d, 0x30, 0xab, 0x9a, 0x15,
				0xa2, 0x18, 0xc6, 0xa9, 0x27, 0x4d, 0x30, 0xab, 0x9a, 0x15,
				0x10, 0x00,
			}},
			wantErr: false,
		},
		{
			name:    "non working scan",
			args:    args{src: int64(1234567890)},
			wantErr: true,
		},
		{
			name: "invalid length scan",
			args: args{src: []byte{
				0xb2, 0x6f, 0x2b, 0x34, 0x2a, 0xab, 0x24, 0xbc, 0xf6, 0x3e,
				0xa2, 0x18, 0xc6, 0xa9, 0x27, 0x4d, 0x30, 0xab, 0x9a, 0x15,
				0xa2, 0x18, 0xc6, 0xa9, 0x27, 0x4d, 0x30, 0xab, 0x9a, 0x15,
			}},
			wantErr: true,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			h := &Hash{}
			if err := h.Scan(tt.args.src); (err != nil) != tt.wantErr {
				t.Errorf("Hash.Scan() error = %v, wantErr %v", err, tt.wantErr)
			}

			if !tt.wantErr {
				for i := range h {
					if h[i] != tt.args.src.([]byte)[i] {
						t.Errorf(
							"Hash.Scan() didn't scan the %d src correctly (have %X, want %X)",
							i, h[i], tt.args.src.([]byte)[i],
						)
					}
				}
			}
		})
	}
}

func TestHash_Value(t *testing.T) {
	b := []byte{
		0xb2, 0x6f, 0x2b, 0x34, 0x2a, 0xab, 0x24, 0xbc, 0xf6, 0x3e,
		0xa2, 0x18, 0xc6, 0xa9, 0x27, 0x4d, 0x30, 0xab, 0x9a, 0x15,
		0xa2, 0x18, 0xc6, 0xa9, 0x27, 0x4d, 0x30, 0xab, 0x9a, 0x15,
		0x10, 0x00,
	}
	var usedH Hash
	usedH.SetBytes(b)
	tests := []struct {
		name    string
		h       Hash
		want    driver.Value
		wantErr bool
	}{
		{
			name:    "Working value",
			h:       usedH,
			want:    b,
			wantErr: false,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, err := tt.h.Value()
			if (err != nil) != tt.wantErr {
				t.Errorf("Hash.Value() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			if !reflect.DeepEqual(got, tt.want) {
				t.Errorf("Hash.Value() = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestAddress_Scan(t *testing.T) {
	type args struct {
		src interface{}
	}
	tests := []struct {
		name    string
		args    args
		wantErr bool
	}{
		{
			name: "working scan",
			args: args{src: []byte{
				0xb2, 0x6f, 0x2b, 0x34, 0x2a, 0xab, 0x24, 0xbc, 0xf6, 0x3e,
				0xa2, 0x18, 0xc6, 0xa9, 0x27, 0x4d, 0x30, 0xab, 0x9a, 0x15,
			}},
			wantErr: false,
		},
		{
			name:    "non working scan",
			args:    args{src: int64(1234567890)},
			wantErr: true,
		},
		{
			name: "invalid length scan",
			args: args{src: []byte{
				0xb2, 0x6f, 0x2b, 0x34, 0x2a, 0xab, 0x24, 0xbc, 0xf6, 0x3e,
				0xa2, 0x18, 0xc6, 0xa9, 0x27, 0x4d, 0x30, 0xab, 0x9a,
			}},
			wantErr: true,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			a := &Address{}
			if err := a.Scan(tt.args.src); (err != nil) != tt.wantErr {
				t.Errorf("Address.Scan() error = %v, wantErr %v", err, tt.wantErr)
			}

			if !tt.wantErr {
				for i := range a {
					if a[i] != tt.args.src.([]byte)[i] {
						t.Errorf(
							"Address.Scan() didn't scan the %d src correctly (have %X, want %X)",
							i, a[i], tt.args.src.([]byte)[i],
						)
					}
				}
			}
		})
	}
}

func TestAddress_Value(t *testing.T) {
	b := []byte{
		0xb2, 0x6f, 0x2b, 0x34, 0x2a, 0xab, 0x24, 0xbc, 0xf6, 0x3e,
		0xa2, 0x18, 0xc6, 0xa9, 0x27, 0x4d, 0x30, 0xab, 0x9a, 0x15,
	}
	var usedA Address
	usedA.SetBytes(b)
	tests := []struct {
		name    string
		a       Address
		want    driver.Value
		wantErr bool
	}{
		{
			name:    "Working value",
			a:       usedA,
			want:    b,
			wantErr: false,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, err := tt.a.Value()
			if (err != nil) != tt.wantErr {
				t.Errorf("Address.Value() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			if !reflect.DeepEqual(got, tt.want) {
				t.Errorf("Address.Value() = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestAddress_Format(t *testing.T) {
	b := []byte{
		0xb2, 0x6f, 0x2b, 0x34, 0x2a, 0xab, 0x24, 0xbc, 0xf6, 0x3e,
		0xa2, 0x18, 0xc6, 0xa9, 0x27, 0x4d, 0x30, 0xab, 0x9a, 0x15,
	}
	var addr Address
	addr.SetBytes(b)

	tests := []struct {
		name string
		out  string
		want string
	}{
		{
			name: "println",
			out:  fmt.Sprintln(addr),
			want: "0xB26f2b342AAb24BCF63ea218c6A9274D30Ab9A15\n",
		},
		{
			name: "print",
			out:  fmt.Sprint(addr),
			want: "0xB26f2b342AAb24BCF63ea218c6A9274D30Ab9A15",
		},
		{
			name: "printf-s",
			out: func() string {
				buf := new(bytes.Buffer)
				fmt.Fprintf(buf, "%s", addr)
				return buf.String()
			}(),
			want: "0xB26f2b342AAb24BCF63ea218c6A9274D30Ab9A15",
		},
		{
			name: "printf-q",
			out:  fmt.Sprintf("%q", addr),
			want: `"0xB26f2b342AAb24BCF63ea218c6A9274D30Ab9A15"`,
		},
		{
			name: "printf-x",
			out:  fmt.Sprintf("%x", addr),
			want: "b26f2b342aab24bcf63ea218c6a9274d30ab9a15",
		},
		{
			name: "printf-X",
			out:  fmt.Sprintf("%X", addr),
			want: "B26F2B342AAB24BCF63EA218C6A9274D30AB9A15",
		},
		{
			name: "printf-#x",
			out:  fmt.Sprintf("%#x", addr),
			want: "0xb26f2b342aab24bcf63ea218c6a9274d30ab9a15",
		},
		{
			name: "printf-v",
			out:  fmt.Sprintf("%v", addr),
			want: "0xB26f2b342AAb24BCF63ea218c6A9274D30Ab9A15",
		},
		// The original default formatter for byte slice
		{
			name: "printf-d",
			out:  fmt.Sprintf("%d", addr),
			want: "[178 111 43 52 42 171 36 188 246 62 162 24 198 169 39 77 48 171 154 21]",
		},
		// Invalid format char.
		{
			name: "printf-t",
			out:  fmt.Sprintf("%t", addr),
			want: "%!t(address=b26f2b342aab24bcf63ea218c6a9274d30ab9a15)",
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if tt.out != tt.want {
				t.Errorf("%s does not render as expected:\n got %s\nwant %s", tt.name, tt.out, tt.want)
			}
		})
	}
}

func TestHash_Format(t *testing.T) {
	var hash Hash
	hash.SetBytes([]byte{
		0xb2, 0x6f, 0x2b, 0x34, 0x2a, 0xab, 0x24, 0xbc, 0xf6, 0x3e,
		0xa2, 0x18, 0xc6, 0xa9, 0x27, 0x4d, 0x30, 0xab, 0x9a, 0x15,
		0xa2, 0x18, 0xc6, 0xa9, 0x27, 0x4d, 0x30, 0xab, 0x9a, 0x15,
		0x10, 0x00,
	})

	tests := []struct {
		name string
		out  string
		want string
	}{
		{
			name: "println",
			out:  fmt.Sprintln(hash),
			want: "0xb26f2b342aab24bcf63ea218c6a9274d30ab9a15a218c6a9274d30ab9a151000\n",
		},
		{
			name: "print",
			out:  fmt.Sprint(hash),
			want: "0xb26f2b342aab24bcf63ea218c6a9274d30ab9a15a218c6a9274d30ab9a151000",
		},
		{
			name: "printf-s",
			out: func() string {
				buf := new(bytes.Buffer)
				fmt.Fprintf(buf, "%s", hash)
				return buf.String()
			}(),
			want: "0xb26f2b342aab24bcf63ea218c6a9274d30ab9a15a218c6a9274d30ab9a151000",
		},
		{
			name: "printf-q",
			out:  fmt.Sprintf("%q", hash),
			want: `"0xb26f2b342aab24bcf63ea218c6a9274d30ab9a15a218c6a9274d30ab9a151000"`,
		},
		{
			name: "printf-x",
			out:  fmt.Sprintf("%x", hash),
			want: "b26f2b342aab24bcf63ea218c6a9274d30ab9a15a218c6a9274d30ab9a151000",
		},
		{
			name: "printf-X",
			out:  fmt.Sprintf("%X", hash),
			want: "B26F2B342AAB24BCF63EA218C6A9274D30AB9A15A218C6A9274D30AB9A151000",
		},
		{
			name: "printf-#x",
			out:  fmt.Sprintf("%#x", hash),
			want: "0xb26f2b342aab24bcf63ea218c6a9274d30ab9a15a218c6a9274d30ab9a151000",
		},
		{
			name: "printf-#X",
			out:  fmt.Sprintf("%#X", hash),
			want: "0XB26F2B342AAB24BCF63EA218C6A9274D30AB9A15A218C6A9274D30AB9A151000",
		},
		{
			name: "printf-v",
			out:  fmt.Sprintf("%v", hash),
			want: "0xb26f2b342aab24bcf63ea218c6a9274d30ab9a15a218c6a9274d30ab9a151000",
		},
		// The original default formatter for byte slice
		{
			name: "printf-d",
			out:  fmt.Sprintf("%d", hash),
			want: "[178 111 43 52 42 171 36 188 246 62 162 24 198 169 39 77 48 171 154 21 162 24 198 169 39 77 48 171 154 21 16 0]",
		},
		// Invalid format char.
		{
			name: "printf-t",
			out:  fmt.Sprintf("%t", hash),
			want: "%!t(hash=b26f2b342aab24bcf63ea218c6a9274d30ab9a15a218c6a9274d30ab9a151000)",
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if tt.out != tt.want {
				t.Errorf("%s does not render as expected:\n got %s\nwant %s", tt.name, tt.out, tt.want)
			}
		})
	}
}

func TestAddressEIP55(t *testing.T) {
	addr := HexToAddress("0x5aaeb6053f3e94c9b9a09f33669435e7ef1beaed")
	addrEIP55 := AddressEIP55(addr)

	if addr.Hex() != addrEIP55.String() {
		t.Fatal("AddressEIP55 should match original address hex")
	}

	blob, err := addrEIP55.MarshalJSON()
	if err != nil {
		t.Fatal("Failed to marshal AddressEIP55", err)
	}
	if strings.Trim(string(blob), "\"") != addr.Hex() {
		t.Fatal("Address with checksum is expected")
	}
	var dec Address
	if err := json.Unmarshal(blob, &dec); err != nil {
		t.Fatal("Failed to unmarshal AddressEIP55", err)
	}
	if addr != dec {
		t.Fatal("Unexpected address after unmarshal")
	}
}

func BenchmarkPrettyDuration(b *testing.B) {
	var x = PrettyDuration(time.Duration(int64(1203123912312)))
	b.Logf("Pre %s", time.Duration(x).String())
	var a string
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		a = x.String()
	}
	b.Logf("Post %s", a)
}
