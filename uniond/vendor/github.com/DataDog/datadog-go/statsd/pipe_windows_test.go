// +build windows

package statsd

import (
	"io/ioutil"
	"net"
	"os"
	"testing"
	"time"

	"github.com/Microsoft/go-winio"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func createNamedPipe(t *testing.T) (string, *os.File, net.Listener) {
	f, err := ioutil.TempFile("", "test-pipe-")
	require.Nil(t, err)

	pipepath := WindowsPipeAddressPrefix + f.Name()
	ln, err := winio.ListenPipe(pipepath, &winio.PipeConfig{
		SecurityDescriptor: "D:AI(A;;GA;;;WD)",
		InputBufferSize:    1_000_000,
	})
	if err != nil {
		os.Remove(f.Name())
		t.Fatal(err)
	}
	return pipepath, f, ln
}

// acceptOne accepts one single connection from ln, reads 512 bytes from it
// and sends it to the out channel, afterwards closing the connection.
func acceptOne(t *testing.T, ln net.Listener, out chan string) {
	conn, err := ln.Accept()
	require.Nil(t, err)

	buf := make([]byte, 512)
	n, err := conn.Read(buf)
	require.Nil(t, err)

	conn.Close()
	out <- string(buf[:n])
}

func TestPipeWriter(t *testing.T) {
	pipepath, f, ln := createNamedPipe(t)
	defer os.Remove(f.Name())

	out := make(chan string)
	go acceptOne(t, ln, out)

	client, err := New(pipepath)
	require.Nil(t, err)

	err = client.Gauge("metric", 1, []string{"key:val"}, 1)
	require.Nil(t, err)

	got := <-out
	assert.Equal(t, got, "metric:1|g|#key:val\n")
}

func TestPipeWriterEnv(t *testing.T) {
	pipepath, f, ln := createNamedPipe(t)
	defer os.Remove(f.Name())

	out := make(chan string)
	go acceptOne(t, ln, out)

	os.Setenv(agentHostEnvVarName, pipepath)
	defer os.Unsetenv(agentHostEnvVarName)

	client, err := New("")
	require.Nil(t, err)

	err = client.Gauge("metric", 1, []string{"key:val"}, 1)
	require.Nil(t, err)

	got := <-out
	assert.Equal(t, got, "metric:1|g|#key:val\n")
}

func TestPipeWriterReconnect(t *testing.T) {
	pipepath, f, ln := createNamedPipe(t)
	defer os.Remove(f.Name())

	out := make(chan string)
	go acceptOne(t, ln, out)
	client, err := New(pipepath)
	require.Nil(t, err)

	// first attempt works, then connection closes
	err = client.Gauge("metric", 1, []string{"key:val"}, 1)
	require.Nil(t, err, "Failed to send gauge: %s", err)

	timeout := time.After(1 * time.Second)
	select {
	case got := <-out:
		assert.Equal(t, got, "metric:1|g|#key:val\n")
	case <-timeout:
		t.Fatal("timeout receiving the first metric")
	}

	// second attempt fails by attempting the same connection
	go acceptOne(t, ln, out)
	err = client.Gauge("metric", 2, []string{"key:val"}, 1)
	require.Nil(t, err, "Failed to send second gauge: %s", err)

	timeout = time.After(100 * time.Millisecond)
	select {
	case <-out:
		t.Fatal("Second attempt should have timed out")
	case <-timeout:
		// ok
	}

	// subsequent attempts succeed with new connection
	for n := 0; n < 3; n++ {
		err = client.Gauge("metric", 3, []string{"key:val"}, 1)
		require.Nil(t, err, "Failed to send second gauge: %s", err)

		timeout = time.After(500 * time.Millisecond)
		select {
		case got := <-out:
			assert.Equal(t, got, "metric:3|g|#key:val\n")
			return
		case <-timeout:
			continue
		}
	}
	t.Fatal("failed to reconnect")
}
