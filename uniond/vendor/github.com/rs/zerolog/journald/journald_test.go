// +build linux

package journald_test

import (
	"bytes"
	"io"
	"testing"

	"github.com/rs/zerolog"
	"github.com/rs/zerolog/journald"
)

func ExampleNewJournalDWriter() {
	log := zerolog.New(journald.NewJournalDWriter())
	log.Info().Str("foo", "bar").Uint64("small", 123).Float64("float", 3.14).Uint64("big", 1152921504606846976).Msg("Journal Test")
	// Output:
}

/*

There is no automated way to verify the output - since the output is sent
to journald process and method to retrieve is journalctl. Will find a way
to automate the process and fix this test.

$ journalctl -o verbose -f

Thu 2018-04-26 22:30:20.768136 PDT [s=3284d695bde946e4b5017c77a399237f;i=329f0;b=98c0dca0debc4b98a5b9534e910e7dd6;m=7a702e35dd4;t=56acdccd2ed0a;x=4690034cf0348614]
    PRIORITY=6
    _AUDIT_LOGINUID=1000
    _BOOT_ID=98c0dca0debc4b98a5b9534e910e7dd6
    _MACHINE_ID=926ed67eb4744580948de70fb474975e
    _HOSTNAME=sprint
    _UID=1000
    _GID=1000
    _CAP_EFFECTIVE=0
    _SYSTEMD_SLICE=-.slice
    _TRANSPORT=journal
    _SYSTEMD_CGROUP=/
    _AUDIT_SESSION=2945
    MESSAGE=Journal Test
    FOO=bar
    BIG=1152921504606846976
    _COMM=journald.test
    SMALL=123
    FLOAT=3.14
    JSON={"level":"info","foo":"bar","small":123,"float":3.14,"big":1152921504606846976,"message":"Journal Test"}
    _PID=27103
    _SOURCE_REALTIME_TIMESTAMP=1524807020768136
*/

func TestWriteReturnsNoOfWrittenBytes(t *testing.T) {
	input := []byte(`{"level":"info","time":1570912626,"message":"Starting..."}`)
	wr := journald.NewJournalDWriter()
	want := len(input)
	got, err := wr.Write(input)

	if err != nil {
		t.Errorf("Unexpected error %v", err)
	}

	if want != got {
		t.Errorf("Expected %d bytes to be written got %d", want, got)
	}
}

func TestMultiWrite(t *testing.T) {
	var (
		w1 = new(bytes.Buffer)
		w2 = new(bytes.Buffer)
		w3 = journald.NewJournalDWriter()
	)

	zerolog.ErrorHandler = func(err error) {
		if err == io.ErrShortWrite {
			t.Errorf("Unexpected ShortWriteError")
			t.FailNow()
		}
	}

	log := zerolog.New(io.MultiWriter(w1, w2, w3)).With().Logger()

	for i := 0; i < 10; i++ {
		log.Info().Msg("Tick!")
	}
}
