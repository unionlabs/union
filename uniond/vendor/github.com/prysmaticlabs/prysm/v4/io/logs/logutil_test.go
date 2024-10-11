package logs

import (
	"testing"

	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

var urltests = []struct {
	url       string
	maskedUrl string
}{
	{"https://a:b@xyz.net", "https://***@xyz.net"},
	{"https://eth-goerli.alchemyapi.io/v2/tOZG5mjl3.zl_nZdZTNIBUzsDq62R_dkOtY",
		"https://eth-goerli.alchemyapi.io/***"},
	{"https://google.com/search?q=golang", "https://google.com/***"},
	{"https://user@example.com/foo%2fbar", "https://***@example.com/***"},
	{"http://john@example.com/#x/y%2Fz", "http://***@example.com/#***"},
	{"https://me:pass@example.com/foo/bar?x=1&y=2", "https://***@example.com/***"},
}

func TestMaskCredentialsLogging(t *testing.T) {
	for _, test := range urltests {
		require.Equal(t, MaskCredentialsLogging(test.url), test.maskedUrl)
	}
}
