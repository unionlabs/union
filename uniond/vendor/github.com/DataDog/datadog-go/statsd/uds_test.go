// +build !windows

package statsd

import (
	"io/ioutil"
	"net"
	"os"
	"path/filepath"
	"reflect"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"

	"github.com/stretchr/testify/suite"
)

var dogstatsdTests = []struct {
	GlobalNamespace string
	GlobalTags      []string
	Method          string
	Metric          string
	Value           interface{}
	Tags            []string
	Rate            float64
	Expected        string
}{
	{"", nil, "Gauge", "test.gauge", 1.0, nil, 1.0, "test.gauge:1|g\n"},
	{"", nil, "Gauge", "test.gauge", 1.0, nil, 0.999999, "test.gauge:1|g|@0.999999\n"},
	{"", nil, "Gauge", "test.gauge", 1.0, []string{"tagA"}, 1.0, "test.gauge:1|g|#tagA\n"},
	{"", nil, "Gauge", "test.gauge", 1.0, []string{"tagA", "tagB"}, 1.0, "test.gauge:1|g|#tagA,tagB\n"},
	{"", nil, "Gauge", "test.gauge", 1.0, []string{"tagA"}, 0.999999, "test.gauge:1|g|@0.999999|#tagA\n"},
	{"", nil, "Count", "test.count", int64(1), []string{"tagA"}, 1.0, "test.count:1|c|#tagA\n"},
	{"", nil, "Count", "test.count", int64(-1), []string{"tagA"}, 1.0, "test.count:-1|c|#tagA\n"},
	{"", nil, "Histogram", "test.histogram", 2.3, []string{"tagA"}, 1.0, "test.histogram:2.3|h|#tagA\n"},
	{"", nil, "Distribution", "test.distribution", 2.3, []string{"tagA"}, 1.0, "test.distribution:2.3|d|#tagA\n"},
	{"", nil, "Set", "test.set", "uuid", []string{"tagA"}, 1.0, "test.set:uuid|s|#tagA\n"},
	{"flubber.", nil, "Set", "test.set", "uuid", []string{"tagA"}, 1.0, "flubber.test.set:uuid|s|#tagA\n"},
	{"", []string{"tagC"}, "Set", "test.set", "uuid", []string{"tagA"}, 1.0, "test.set:uuid|s|#tagC,tagA\n"},
	{"", nil, "Count", "test.count", int64(1), []string{"hello\nworld"}, 1.0, "test.count:1|c|#helloworld\n"},
}

type testUnixgramServer struct {
	tmpDir string
	*net.UnixConn
}

// newUnixgramServer returns a unix:// addr as well as a *net.UnixConn
// server. Any error will fail the test given in param.
func newTestUnixgramServer(t *testing.T) *testUnixgramServer {
	dir, err := ioutil.TempDir("", "socket")
	if err != nil {
		t.Fatal(err)
	}

	addr := filepath.Join(dir, "dsd.socket")

	udsAddr, err := net.ResolveUnixAddr("unixgram", addr)
	if err != nil {
		t.Fatal(err)
	}

	server, err := net.ListenUnixgram("unixgram", udsAddr)
	if err != nil {
		t.Fatal(err)
	}

	return &testUnixgramServer{dir, server}
}

func (ts *testUnixgramServer) Cleanup() {
	os.RemoveAll(ts.tmpDir) // clean up
	ts.Close()
}

func (ts *testUnixgramServer) AddrString() string {
	return UnixAddressPrefix + ts.LocalAddr().String()
}

// UdsTestSuite contains generic tests valid for both UDS implementations
type UdsTestSuite struct {
	suite.Suite
	options []Option
}

func TestUdsBlocking(t *testing.T) {
	suite.Run(t, &UdsTestSuite{})
}

func (suite *UdsTestSuite) TestClientUDS() {
	server := newTestUnixgramServer(suite.T())
	defer server.Cleanup()

	client, err := New(server.AddrString(), suite.options...)
	if err != nil {
		suite.T().Fatal(err)
	}

	for _, tt := range dogstatsdTests {
		client.Namespace = tt.GlobalNamespace
		client.Tags = tt.GlobalTags
		method := reflect.ValueOf(client).MethodByName(tt.Method)
		e := method.Call([]reflect.Value{
			reflect.ValueOf(tt.Metric),
			reflect.ValueOf(tt.Value),
			reflect.ValueOf(tt.Tags),
			reflect.ValueOf(tt.Rate)})[0]
		errInter := e.Interface()
		if errInter != nil {
			suite.T().Fatal(errInter.(error))
		}

		bytes := make([]byte, 1024)
		n, err := server.Read(bytes)
		if err != nil {
			suite.T().Fatal(err)
		}
		message := bytes[:n]
		if string(message) != tt.Expected {
			suite.T().Errorf("Expected: %s. Actual: %s", tt.Expected, string(message))
		}
	}
}

func (suite *UdsTestSuite) TestClientUDSConcurrent() {
	server := newTestUnixgramServer(suite.T())
	defer server.Cleanup()

	client, err := New(server.AddrString(), WithMaxMessagesPerPayload(1))
	if err != nil {
		suite.T().Fatal(err)
	}

	numWrites := 10
	for i := 0; i < numWrites; i++ {
		go func() {
			err := client.Gauge("test.gauge", 1.0, []string{}, 1)
			if err != nil {
				suite.T().Errorf("error outputting gauge %s", err)
			}
		}()
	}

	expected := "test.gauge:1|g\n"
	var msgs []string
	for {
		bytes := make([]byte, 1024)
		server.SetReadDeadline(time.Now().Add(200 * time.Millisecond))
		n, err := server.Read(bytes)
		if err, ok := err.(net.Error); ok && err.Timeout() {
			break
		}
		if err != nil {
			suite.T().Fatal(err)
		}
		message := string(bytes[:n])
		msgs = append(msgs, message)
		if message != expected {
			suite.T().Errorf("Got %s, expected %s", message, expected)
		}
	}

	if len(msgs) != numWrites {
		suite.T().Errorf("Got %d writes, expected %d. Data: %v", len(msgs), numWrites, msgs)
	}
}

func (suite *UdsTestSuite) TestClientUDSClose() {
	ts := newTestUnixgramServer(suite.T())
	defer ts.Cleanup()

	// close the server ourselves and test code when nobody's listening
	ts.Close()
	client, err := New(ts.AddrString())
	if err != nil {
		suite.T().Fatal(err)
	}

	assert.NotPanics(suite.T(), func() { client.Close() })
}

func TestConnectionNotUnset(t *testing.T) {
	ts := newTestUnixgramServer(t)
	defer ts.Cleanup()

	writer, err := newUDSWriter(ts.LocalAddr().String())
	assert.NoError(t, err)

	_, err = writer.Write([]byte("test.gauge:1|g"))
	assert.NoError(t, err)
	assert.NotNil(t, writer.conn)
}
