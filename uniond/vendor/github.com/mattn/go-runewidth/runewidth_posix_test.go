//go:build !windows && !js && !appengine
// +build !windows,!js,!appengine

package runewidth

import (
	"os"
	"testing"
)

type envVars struct {
	lang    string
	lcall   string
	lcctype string
}

func saveEnv() envVars {
	return envVars{
		lang:    os.Getenv("LANG"),
		lcall:   os.Getenv("LC_ALL"),
		lcctype: os.Getenv("LC_CTYPE"),
	}
}
func restoreEnv(env *envVars) {
	os.Setenv("LANG", env.lang)
	os.Setenv("LC_ALL", env.lcall)
	os.Setenv("LC_CTYPE", env.lcctype)
}

func TestIsEastAsian(t *testing.T) {
	testcases := []struct {
		locale string
		want   bool
	}{
		{"foo@cjk_narrow", false},
		{"foo@cjk", false},
		{"utf-8@cjk", false},
		{"ja_JP.CP932", true},
	}

	for _, tt := range testcases {
		got := isEastAsian(tt.locale)
		if got != tt.want {
			t.Fatalf("isEastAsian(%q) should be %v", tt.locale, tt.want)
		}
	}
}

func TestIsEastAsianLCCTYPE(t *testing.T) {
	env := saveEnv()
	defer restoreEnv(&env)
	os.Setenv("LC_ALL", "")

	testcases := []struct {
		lcctype string
		want    bool
	}{
		{"ja_JP.UTF-8", true},
		{"C", false},
		{"POSIX", false},
		{"en_US.UTF-8", false},
	}

	for _, tt := range testcases {
		os.Setenv("LC_CTYPE", tt.lcctype)
		got := IsEastAsian()
		if got != tt.want {
			t.Fatalf("IsEastAsian() for LC_CTYPE=%v should be %v", tt.lcctype, tt.want)
		}
	}
}

func TestIsEastAsianLANG(t *testing.T) {
	env := saveEnv()
	defer restoreEnv(&env)
	os.Setenv("LC_ALL", "")
	os.Setenv("LC_CTYPE", "")

	testcases := []struct {
		lcctype string
		want    bool
	}{
		{"ja_JP.UTF-8", true},
		{"C", false},
		{"POSIX", false},
		{"en_US.UTF-8", false},
		{"C.UTF-8", false},
	}

	for _, tt := range testcases {
		os.Setenv("LANG", tt.lcctype)
		got := IsEastAsian()
		if got != tt.want {
			t.Fatalf("IsEastAsian() for LANG=%v should be %v", tt.lcctype, tt.want)
		}
	}
}
