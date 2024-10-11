package log_test

import (
	"testing"

	"gotest.tools/v3/assert"

	"cosmossdk.io/log"
)

func TestParseLogLevel(t *testing.T) {
	_, err := log.ParseLogLevel("")
	assert.Error(t, err, "empty log level")

	level := "consensus:foo,mempool:debug,*:error"
	_, err = log.ParseLogLevel(level)
	assert.Error(t, err, "invalid log level foo in log level list [consensus:foo mempool:debug *:error]")

	level = "consensus:debug,mempool:debug,*:error"
	filter, err := log.ParseLogLevel(level)
	assert.NilError(t, err)
	assert.Assert(t, filter != nil)

	assert.Assert(t, !filter("consensus", "debug"))
	assert.Assert(t, !filter("consensus", "info"))
	assert.Assert(t, !filter("consensus", "error"))
	assert.Assert(t, !filter("mempool", "debug"))
	assert.Assert(t, !filter("mempool", "info"))
	assert.Assert(t, !filter("mempool", "error"))
	assert.Assert(t, !filter("state", "error"))
	assert.Assert(t, !filter("server", "panic"))

	assert.Assert(t, filter("server", "debug"))
	assert.Assert(t, filter("state", "debug"))
	assert.Assert(t, filter("state", "info"))

	level = "error"
	filter, err = log.ParseLogLevel(level)
	assert.NilError(t, err)
	assert.Assert(t, filter != nil)

	assert.Assert(t, !filter("state", "error"))
	assert.Assert(t, !filter("consensus", "error"))

	assert.Assert(t, filter("consensus", "debug"))
	assert.Assert(t, filter("consensus", "info"))
	assert.Assert(t, filter("state", "debug"))
}
