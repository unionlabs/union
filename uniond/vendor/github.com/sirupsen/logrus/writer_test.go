package logrus_test

import (
	"bufio"
	"bytes"
	"log"
	"net/http"
	"strings"
	"testing"
	"time"

	"github.com/sirupsen/logrus"
	"github.com/stretchr/testify/assert"
)

func ExampleLogger_Writer_httpServer() {
	logger := logrus.New()
	w := logger.Writer()
	defer w.Close()

	srv := http.Server{
		// create a stdlib log.Logger that writes to
		// logrus.Logger.
		ErrorLog: log.New(w, "", 0),
	}

	if err := srv.ListenAndServe(); err != nil {
		logger.Fatal(err)
	}
}

func ExampleLogger_Writer_stdlib() {
	logger := logrus.New()
	logger.Formatter = &logrus.JSONFormatter{}

	// Use logrus for standard log output
	// Note that `log` here references stdlib's log
	// Not logrus imported under the name `log`.
	log.SetOutput(logger.Writer())
}

func TestWriterSplitNewlines(t *testing.T) {
	buf := bytes.NewBuffer(nil)
	logger := logrus.New()
	logger.Formatter = &logrus.TextFormatter{
		DisableColors:    true,
		DisableTimestamp: true,
	}
	logger.SetOutput(buf)
	writer := logger.Writer()

	const logNum = 10

	for i := 0; i < logNum; i++ {
		_, err := writer.Write([]byte("bar\nfoo\n"))
		assert.NoError(t, err, "writer.Write failed")
	}
	writer.Close()
	// Test is flaky because it writes in another goroutine,
	// we need to make sure to wait a bit so all write are done.
	time.Sleep(500 * time.Millisecond)

	lines := strings.Split(strings.TrimRight(buf.String(), "\n"), "\n")
	assert.Len(t, lines, logNum*2, "logger printed incorrect number of lines")
}

func TestWriterSplitsMax64KB(t *testing.T) {
	buf := bytes.NewBuffer(nil)
	logger := logrus.New()
	logger.Formatter = &logrus.TextFormatter{
		DisableColors:    true,
		DisableTimestamp: true,
	}
	logger.SetOutput(buf)
	writer := logger.Writer()

	// write more than 64KB
	const bigWriteLen = bufio.MaxScanTokenSize + 100
	output := make([]byte, bigWriteLen)
	// lets not write zero bytes
	for i := 0; i < bigWriteLen; i++ {
		output[i] = 'A'
	}

	for i := 0; i < 3; i++ {
		len, err := writer.Write(output)
		assert.NoError(t, err, "writer.Write failed")
		assert.Equal(t, bigWriteLen, len, "bytes written")
	}
	writer.Close()
	// Test is flaky because it writes in another goroutine,
	// we need to make sure to wait a bit so all write are done.
	time.Sleep(500 * time.Millisecond)

	lines := strings.Split(strings.TrimRight(buf.String(), "\n"), "\n")
	// we should have 4 lines because we wrote more than 64 KB each time
	assert.Len(t, lines, 4, "logger printed incorrect number of lines")
}
