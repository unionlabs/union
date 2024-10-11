package confix_test

import (
	"os"
	"testing"

	"gotest.tools/v3/assert"

	"cosmossdk.io/tools/confix"
)

func mustReadConfig(t *testing.T, path string) []byte {
	f, err := os.ReadFile(path)
	if err != nil {
		t.Fatalf("failed to open file: %v", err)
	}

	return f
}

func TestCheckValid(t *testing.T) {
	err := confix.CheckValid("foo", []byte{})
	assert.ErrorContains(t, err, "unknown config")

	err = confix.CheckValid("client", mustReadConfig(t, "data/v0.45-app.toml"))
	assert.ErrorContains(t, err, "unknown config")

	err = confix.CheckValid("config.toml", mustReadConfig(t, "data/v0.45-app.toml"))
	assert.Error(t, err, "cometbft config is not supported")

	err = confix.CheckValid("client.toml", mustReadConfig(t, "data/v0.45-app.toml"))
	assert.Error(t, err, "client config invalid: chain-id is empty")

	err = confix.CheckValid("client.toml", []byte{})
	assert.Error(t, err, "client config invalid: chain-id is empty")

	err = confix.CheckValid("app.toml", []byte{})
	assert.ErrorContains(t, err, "server config invalid")

	err = confix.CheckValid("app.toml", mustReadConfig(t, "data/v0.45-app.toml"))
	assert.NilError(t, err)

	err = confix.CheckValid("client.toml", mustReadConfig(t, "testdata/client.toml"))
	assert.NilError(t, err)
}
