// This file is https://github.com/orofarne/gowchar/blob/master/gowchar_test.go
//
// It was vendored inline to work around CGO limitations that don't allow C types
// to directly cross package API boundaries.
//
// The vendored file is licensed under the 3-clause BSD license, according to:
// https://github.com/orofarne/gowchar/blob/master/LICENSE

// +build !ios
// +build linux darwin windows

package hid

import (
	"fmt"
	"testing"
)

func TestGowcharSimple(t *testing.T) {
	str1 := "Привет, 世界. 𪛖"
	wstr, size := stringToWcharT(str1)
	switch sizeofWcharT {
	case 2:
		if size != 15 {
			t.Errorf("size(%v) != 15", size)
		}
	case 4:
		if size != 14 {
			t.Errorf("size(%v) != 14", size)
		}
	default:
		panic(fmt.Sprintf("sizeof(wchar_t) = %v", sizeofWcharT))
	}
	str2, err := wcharTToString(wstr)
	if err != nil {
		t.Errorf("wcharTToString error: %v", err)
	}
	if str1 != str2 {
		t.Errorf("\"%s\" != \"%s\"", str1, str2)
	}
}

func TestGowcharSimpleN(t *testing.T) {
	str1 := "Привет, 世界. 𪛖"
	wstr, size := stringToWcharT(str1)
	switch sizeofWcharT {
	case 2:
		if size != 15 {
			t.Errorf("size(%v) != 15", size)
		}
	case 4:
		if size != 14 {
			t.Errorf("size(%v) != 14", size)
		}
	default:
		panic(fmt.Sprintf("sizeof(wchar_t) = %v", sizeofWcharT))
	}

	str2, err := wcharTNToString(wstr, size-1)
	if err != nil {
		t.Errorf("wcharTToString error: %v", err)
	}
	if str1 != str2 {
		t.Errorf("\"%s\" != \"%s\"", str1, str2)
	}
}
