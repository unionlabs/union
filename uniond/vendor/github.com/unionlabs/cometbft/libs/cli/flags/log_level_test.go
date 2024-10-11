package flags_test

import (
	"bytes"
	"strings"
	"testing"

	cmtflags "github.com/cometbft/cometbft/libs/cli/flags"
	"github.com/cometbft/cometbft/libs/log"
)

const (
	defaultLogLevelValue = "info"
)

func TestParseLogLevel(t *testing.T) {
	var buf bytes.Buffer
	jsonLogger := log.NewTMJSONLoggerNoTS(&buf)

	correctLogLevels := []struct {
		lvl              string
		expectedLogLines []string
	}{
		{"mempool:error", []string{
			``, // if no default is given, assume info
			``,
			`{"_msg":"Mesmero","level":"error","module":"mempool"}`,
			`{"_msg":"Mind","level":"info","module":"state"}`, // if no default is given, assume info
			``,
		}},

		{"mempool:error,*:debug", []string{
			`{"_msg":"Kingpin","level":"debug","module":"wire"}`,
			``,
			`{"_msg":"Mesmero","level":"error","module":"mempool"}`,
			`{"_msg":"Mind","level":"info","module":"state"}`,
			`{"_msg":"Gideon","level":"debug"}`,
		}},

		{"*:debug,wire:none", []string{
			``,
			`{"_msg":"Kitty Pryde","level":"info","module":"mempool"}`,
			`{"_msg":"Mesmero","level":"error","module":"mempool"}`,
			`{"_msg":"Mind","level":"info","module":"state"}`,
			`{"_msg":"Gideon","level":"debug"}`,
		}},
	}

	for _, c := range correctLogLevels {
		logger, err := cmtflags.ParseLogLevel(c.lvl, jsonLogger, defaultLogLevelValue)
		if err != nil {
			t.Fatal(err)
		}

		buf.Reset()

		logger.With("module", "mempool").With("module", "wire").Debug("Kingpin")
		if have := strings.TrimSpace(buf.String()); c.expectedLogLines[0] != have {
			t.Errorf("\nwant '%s'\nhave '%s'\nlevel '%s'", c.expectedLogLines[0], have, c.lvl)
		}

		buf.Reset()

		logger.With("module", "mempool").Info("Kitty Pryde")
		if have := strings.TrimSpace(buf.String()); c.expectedLogLines[1] != have {
			t.Errorf("\nwant '%s'\nhave '%s'\nlevel '%s'", c.expectedLogLines[1], have, c.lvl)
		}

		buf.Reset()

		logger.With("module", "mempool").Error("Mesmero")
		if have := strings.TrimSpace(buf.String()); c.expectedLogLines[2] != have {
			t.Errorf("\nwant '%s'\nhave '%s'\nlevel '%s'", c.expectedLogLines[2], have, c.lvl)
		}

		buf.Reset()

		logger.With("module", "state").Info("Mind")
		if have := strings.TrimSpace(buf.String()); c.expectedLogLines[3] != have {
			t.Errorf("\nwant '%s'\nhave '%s'\nlevel '%s'", c.expectedLogLines[3], have, c.lvl)
		}

		buf.Reset()

		logger.Debug("Gideon")
		if have := strings.TrimSpace(buf.String()); c.expectedLogLines[4] != have {
			t.Errorf("\nwant '%s'\nhave '%s'\nlevel '%s'", c.expectedLogLines[4], have, c.lvl)
		}
	}

	incorrectLogLevel := []string{"some", "mempool:some", "*:some,mempool:error"}
	for _, lvl := range incorrectLogLevel {
		if _, err := cmtflags.ParseLogLevel(lvl, jsonLogger, defaultLogLevelValue); err == nil {
			t.Fatalf("Expected %s to produce error", lvl)
		}
	}
}
