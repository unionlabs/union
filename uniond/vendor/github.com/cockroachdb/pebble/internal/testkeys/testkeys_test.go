// Copyright 2021 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package testkeys

import (
	"bytes"
	"fmt"
	"sort"
	"testing"

	"github.com/cockroachdb/datadriven"
	"github.com/stretchr/testify/require"
	"golang.org/x/exp/rand"
)

func TestGenerateAlphabetKey(t *testing.T) {
	testCases := []struct {
		i     int64
		depth int
		want  string
	}{
		{0, 1, "a"},
		{0, 2, "a"},
		{0, 3, "a"},

		{1, 1, "b"},
		{2, 1, "c"},

		{0, 2, "a"},
		{1, 2, "aa"},
		{2, 2, "ab"},
		{3, 2, "ac"},
		{4, 2, "b"},
		{5, 2, "ba"},
		{6, 2, "bb"},
		{7, 2, "bc"},
		{8, 2, "c"},
		{9, 2, "ca"},
		{10, 2, "cb"},
		{11, 2, "cc"},
	}
	testAlphabet := []byte{byte('a'), byte('b'), byte('c')}
	testInverseAlphabet := map[byte]int64{byte('a'): 0, byte('b'): 1, byte('c'): 2}

	buf := make([]byte, 10)
	for _, tc := range testCases {
		kc := keyCount(len(testAlphabet), tc.depth)
		n := generateAlphabetKey(buf, testAlphabet, tc.i, kc)
		got := string(buf[:n])
		if got != tc.want {
			t.Errorf("generateAlphabetKey(%q, %d, %d) = %q, want %q", testAlphabet, tc.i, kc, got, tc.want)
		}
		i := computeAlphabetKeyIndex([]byte(got), testInverseAlphabet, tc.depth)
		if i != tc.i {
			t.Errorf("computeAlphabetKeyIndex(%q, %d) = %d, want %d", got, tc.depth, i, tc.i)
		}
	}
}

func TestKeyCount(t *testing.T) {
	type params struct {
		n, l int
	}
	testCases := map[params]int64{
		{26, 1}: 26,
		{52, 1}: 52,
		{2, 2}:  6,
		{2, 3}:  14,
		{2, 4}:  30,
		{3, 2}:  12,
	}
	for p, want := range testCases {
		got := keyCount(p.n, p.l)
		if got != want {
			t.Errorf("keyCount(%d, %d) = %d, want %d", p.n, p.l, got, want)
		}
	}
}

func TestFullKeyspaces(t *testing.T) {
	testCases := []struct {
		ks   Keyspace
		want string
	}{
		{
			Alpha(1),
			"a b c d e f g h i j k l m n o p q r s t u v w x y z",
		},
		{
			alphabet{[]byte("abc"), 2, 0, 0, 1},
			"a aa ab ac b ba bb bc c ca cb cc",
		},
		{
			alphabet{[]byte("abc"), 2, 0, 0, 2},
			"a ab b bb c cb",
		},
		{
			alphabet{[]byte("abc"), 3, 0, 0, 1},
			"a aa aaa aab aac ab aba abb abc ac aca acb acc b ba baa bab bac bb bba bbb bbc bc bca bcb bcc c ca caa cab cac cb cba cbb cbc cc cca ccb ccc",
		},
		{
			alphabet{[]byte("abc"), 3, 7, 10, 1},
			"abb abc ac aca acb acc b ba baa bab bac bb bba bbb bbc bc bca bcb bcc c ca caa",
		},
	}
	for _, tc := range testCases {
		require.Equal(t, tc.want, keyspaceToString(tc.ks))
	}
}

func TestSlice(t *testing.T) {
	testCases := []struct {
		orig Keyspace
		i, j int64
		want string
	}{
		{Alpha(1), 1, 25, "b c d e f g h i j k l m n o p q r s t u v w x y"},
		{Alpha(1).Slice(1, 25), 1, 23, "c d e f g h i j k l m n o p q r s t u v w x"},
		{Alpha(1).Slice(1, 25).Slice(1, 23), 10, 22, "m n o p q r s t u v w x"},
	}
	for _, tc := range testCases {
		got := keyspaceToString(tc.orig.Slice(tc.i, tc.j))
		if got != tc.want {
			t.Errorf("(%q).Slice(%d, %d) = %q, want %q",
				keyspaceToString(tc.orig), tc.i, tc.j, got, tc.want)
		}
	}
}

func TestSuffix(t *testing.T) {
	ks := Alpha(3)
	require.Equal(t, "a@1", string(KeyAt(ks, 0, 1)))
	require.Equal(t, "a@10", string(KeyAt(ks, 0, 10)))
	require.Equal(t, "aab@5", string(KeyAt(ks, 3, 5)))

	assertCmp := func(want int, a, b []byte) {
		got := Comparer.Compare(a, b)
		if got != want {
			t.Helper()
			t.Errorf("Compare(%q, %q) = %d, want %d", a, b, got, want)
		}
	}

	for i := int64(1); i < ks.Count(); i++ {
		assertCmp(-1, KeyAt(ks, i-1, 1), KeyAt(ks, i, 1))
		assertCmp(-1, Key(ks, i-1), Key(ks, i))
		assertCmp(0, Key(ks, i), Key(ks, i))
		for ts := int64(2); ts < 11; ts++ {
			assertCmp(+1, KeyAt(ks, i, ts-1), KeyAt(ks, i, ts))
			assertCmp(-1, KeyAt(ks, i-1, ts-1), KeyAt(ks, i, ts))
		}
	}

	// Suffixes should be comparable on their own too.
	a, b := make([]byte, MaxSuffixLen), make([]byte, MaxSuffixLen)
	for ts := int64(2); ts < 150; ts++ {
		an := WriteSuffix(a, ts-1)
		bn := WriteSuffix(b, ts)
		assertCmp(+1, a[:an], b[:bn])
	}
}

func TestSuffixLen(t *testing.T) {
	testCases := map[int64]int{
		0:    2,
		1:    2,
		5:    2,
		9:    2,
		10:   3,
		17:   3,
		20:   3,
		99:   3,
		100:  4,
		101:  4,
		999:  4,
		1000: 5,
	}
	for ts, want := range testCases {
		if got := SuffixLen(ts); got != want {
			t.Errorf("SuffixLen(%d) = %d, want %d", ts, got, want)
		}
	}
}

func TestDivvy(t *testing.T) {
	var buf bytes.Buffer
	datadriven.RunTest(t, "testdata/divvy", func(t *testing.T, d *datadriven.TestData) string {
		buf.Reset()
		switch d.Cmd {
		case "divvy":
			var alphaLen int
			var portions int64
			d.ScanArgs(t, "alpha", &alphaLen)
			d.ScanArgs(t, "portions", &portions)

			input := Alpha(alphaLen)
			for _, ks := range Divvy(input, portions) {
				fmt.Fprintln(&buf, keyspaceToString(ks))
			}
			return buf.String()
		default:
			return fmt.Sprintf("unrecognized command %q", d.Cmd)
		}
	})
}

func keyspaceToString(ks Keyspace) string {
	var buf bytes.Buffer
	b := make([]byte, ks.MaxLen())
	for i := int64(0); i < ks.Count(); i++ {
		n := ks.key(b, i)
		if i > 0 {
			buf.WriteRune(' ')
		}
		buf.Write(b[:n])
	}
	return buf.String()
}

func TestRandomSeparator(t *testing.T) {
	rng := rand.New(rand.NewSource(0))
	keys := []string{"a", "zzz@9"}
	for n := 0; n < 1000; n++ {
		i := rng.Intn(len(keys))
		j := rng.Intn(len(keys))
		for i == j {
			j = rng.Intn(len(keys))
		}
		if i > j {
			i, j = j, i
		}

		a := []byte(keys[i])
		b := []byte(keys[j])
		suffix := rng.Int63n(10)
		sep := RandomSeparator(nil, a, b, suffix, 3, rng)
		t.Logf("RandomSeparator(%q, %q, %d) = %q\n", a, b, suffix, sep)
		if sep == nil {
			continue
		}
		for k := 0; k < len(keys); k++ {
			v := Comparer.Compare(sep, []byte(keys[k]))
			if k <= i && v <= 0 || k >= j && v >= 0 {
				t.Fatalf("RandomSeparator(%q, %q, %d) = %q; but Compare(%q,%q) = %d\n", a, b, suffix, sep, sep, keys[k], v)
			}
		}
		keys = append(keys, string(sep))
		sort.Slice(keys, func(i, j int) bool {
			return Comparer.Compare([]byte(keys[i]), []byte(keys[j])) < 0
		})
	}
}
