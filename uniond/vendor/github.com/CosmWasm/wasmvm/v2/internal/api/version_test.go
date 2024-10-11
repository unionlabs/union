package api

import (
	"regexp"
	"testing"

	"github.com/stretchr/testify/require"
)

func TestLibwasmvmVersion(t *testing.T) {
	version, err := LibwasmvmVersion()
	require.NoError(t, err)
	require.Regexp(t, regexp.MustCompile(`^([0-9]+)\.([0-9]+)\.([0-9]+)(-[a-z0-9.]+)?$`), version)
}
