package levels_test

import (
	"bytes"
	"os"
	"testing"

	levels "github.com/go-kit/kit/log/deprecated_levels"
	"github.com/go-kit/log"
)

func TestDefaultLevels(t *testing.T) {
	buf := bytes.Buffer{}
	logger := levels.New(log.NewLogfmtLogger(&buf))

	logger.Debug().Log("msg", "résumé") // of course you'd want to do this
	if want, have := "level=debug msg=résumé\n", buf.String(); want != have {
		t.Errorf("want %#v, have %#v", want, have)
	}

	buf.Reset()
	logger.Info().Log("msg", "Åhus")
	if want, have := "level=info msg=Åhus\n", buf.String(); want != have {
		t.Errorf("want %#v, have %#v", want, have)
	}

	buf.Reset()
	logger.Error().Log("msg", "© violation")
	if want, have := "level=error msg=\"© violation\"\n", buf.String(); want != have {
		t.Errorf("want %#v, have %#v", want, have)
	}

	buf.Reset()
	logger.Crit().Log("msg", "	")
	if want, have := "level=crit msg=\"\\t\"\n", buf.String(); want != have {
		t.Errorf("want %#v, have %#v", want, have)
	}
}

func TestModifiedLevels(t *testing.T) {
	buf := bytes.Buffer{}
	logger := levels.New(
		log.NewJSONLogger(&buf),
		levels.Key("l"),
		levels.DebugValue("dbg"),
		levels.InfoValue("nfo"),
		levels.WarnValue("wrn"),
		levels.ErrorValue("err"),
		levels.CritValue("crt"),
	)
	logger.With("easter_island", "176°").Debug().Log("msg", "moai")
	if want, have := `{"easter_island":"176°","l":"dbg","msg":"moai"}`+"\n", buf.String(); want != have {
		t.Errorf("want %#v, have %#v", want, have)
	}
}

func ExampleLevels() {
	logger := levels.New(log.NewLogfmtLogger(os.Stdout))
	logger.Debug().Log("msg", "hello")
	logger.With("context", "foo").Warn().Log("err", "error")

	// Output:
	// level=debug msg=hello
	// level=warn context=foo err=error
}
