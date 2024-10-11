package prefixed_test

import (
	"fmt"
	"regexp"
	"testing"

	"github.com/pkg/errors"
	prefixed "github.com/prysmaticlabs/prysm/v4/runtime/logging/logrus-prefixed-formatter"
	"github.com/prysmaticlabs/prysm/v4/testing/require"

	. "github.com/onsi/ginkgo"
	. "github.com/onsi/gomega"
	"github.com/sirupsen/logrus"
)

var _ = Describe("Formatter", func() {
	var formatter *prefixed.TextFormatter
	var log *logrus.Logger
	var output *LogOutput

	BeforeEach(func() {
		output = new(LogOutput)
		formatter = new(prefixed.TextFormatter)
		log = logrus.New()
		log.Out = output
		log.Formatter = formatter
		log.Level = logrus.DebugLevel
	})

	Describe("logfmt output", func() {
		It("should output simple message", func() {
			formatter.DisableTimestamp = true
			log.Debug("test")
			Ω(output.GetValue()).Should(Equal("level=debug msg=test\n"))
		})

		It("should output message with additional field", func() {
			formatter.DisableTimestamp = true
			log.WithFields(logrus.Fields{"animal": "walrus"}).Debug("test")
			Ω(output.GetValue()).Should(Equal("level=debug msg=test animal=walrus\n"))
		})
	})

	Describe("Formatted output", func() {
		It("should output formatted message", func() {
			formatter.DisableTimestamp = true
			formatter.ForceFormatting = true
			log.Debug("test")
			Ω(output.GetValue()).Should(Equal("DEBUG test\n"))
		})
	})

	Describe("Theming support", func() {

	})
})

func TestFormatter_SuppressErrorStackTraces(t *testing.T) {
	formatter := new(prefixed.TextFormatter)
	formatter.ForceFormatting = true
	log := logrus.New()
	log.Formatter = formatter
	output := new(LogOutput)
	log.Out = output

	errFn := func() error {
		return errors.New("inner")
	}

	log.WithError(errors.Wrap(errFn(), "outer")).Error("test")
	require.Equal(t, true, regexp.MustCompile(`test error=outer: inner\n\s*$`).MatchString(output.GetValue()), fmt.Sprintf("wrong log output: %s", output.GetValue()))
}

func TestFormatter_EscapesControlCharacters(t *testing.T) {
	formatter := new(prefixed.TextFormatter)
	formatter.ForceFormatting = true
	log := logrus.New()
	log.Formatter = formatter
	output := new(LogOutput)
	log.Out = output

	log.WithField("test", "foo\nbar").Error("testing things")
	require.Equal(t, "[0000] ERROR testing things test=foobar"+"\n", output.GetValue())
}
