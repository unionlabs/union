package log_test

import (
	"bytes"
	"strings"
	"testing"

	"gotest.tools/v3/assert"

	"cosmossdk.io/log"
)

func TestFilteredWriter(t *testing.T) {
	buf := new(bytes.Buffer)

	level := "consensus:debug,mempool:debug,*:error"
	filter, err := log.ParseLogLevel(level)
	assert.NilError(t, err)

	logger := log.NewLogger(buf, log.FilterOption(filter))
	logger.Debug("this log line should be displayed", log.ModuleKey, "consensus")
	assert.Check(t, strings.Contains(buf.String(), "this log line should be displayed"))
	buf.Reset()

	logger.Debug("this log line should be filtered", log.ModuleKey, "server")
	assert.Check(t, buf.Len() == 0)
}
