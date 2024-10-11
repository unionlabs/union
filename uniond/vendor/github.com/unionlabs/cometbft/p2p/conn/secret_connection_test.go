package conn

import (
	"bufio"
	"bytes"
	"encoding/hex"
	"errors"
	"flag"
	"fmt"
	"io"
	"log"
	"os"
	"path/filepath"
	"strconv"
	"strings"
	"sync"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"github.com/cometbft/cometbft/crypto"
	"github.com/cometbft/cometbft/crypto/ed25519"
	"github.com/cometbft/cometbft/internal/async"
	cmtos "github.com/cometbft/cometbft/internal/os"
	cmtrand "github.com/cometbft/cometbft/internal/rand"
)

// Run go test -update from within this module
// to update the golden test vector file.
var update = flag.Bool("update", false, "update .golden files")

type kvstoreConn struct {
	*io.PipeReader
	*io.PipeWriter
}

func (drw kvstoreConn) Close() (err error) {
	err2 := drw.PipeWriter.CloseWithError(io.EOF)
	err1 := drw.PipeReader.Close()
	if err2 != nil {
		return err2
	}
	return err1
}

type privKeyWithNilPubKey struct {
	orig crypto.PrivKey
}

func (pk privKeyWithNilPubKey) Bytes() []byte                   { return pk.orig.Bytes() }
func (pk privKeyWithNilPubKey) Sign(msg []byte) ([]byte, error) { return pk.orig.Sign(msg) }
func (privKeyWithNilPubKey) PubKey() crypto.PubKey              { return nil }
func (privKeyWithNilPubKey) Type() string                       { return "privKeyWithNilPubKey" }

func TestSecretConnectionHandshake(t *testing.T) {
	fooSecConn, barSecConn := makeSecretConnPair(t)
	if err := fooSecConn.Close(); err != nil {
		t.Error(err)
	}
	if err := barSecConn.Close(); err != nil {
		t.Error(err)
	}
}

func TestConcurrentWrite(t *testing.T) {
	fooSecConn, barSecConn := makeSecretConnPair(t)
	fooWriteText := cmtrand.Str(dataMaxSize)

	// write from two routines.
	// should be safe from race according to net.Conn:
	// https://golang.org/pkg/net/#Conn
	n := 100
	wg := new(sync.WaitGroup)
	wg.Add(3)
	go writeLots(t, wg, fooSecConn, fooWriteText, n)
	go writeLots(t, wg, fooSecConn, fooWriteText, n)

	// Consume reads from bar's reader
	readLots(t, wg, barSecConn, n*2)
	wg.Wait()

	if err := fooSecConn.Close(); err != nil {
		t.Error(err)
	}
}

func TestConcurrentRead(t *testing.T) {
	fooSecConn, barSecConn := makeSecretConnPair(t)
	fooWriteText := cmtrand.Str(dataMaxSize)
	n := 100

	// read from two routines.
	// should be safe from race according to net.Conn:
	// https://golang.org/pkg/net/#Conn
	wg := new(sync.WaitGroup)
	wg.Add(3)
	go readLots(t, wg, fooSecConn, n/2)
	go readLots(t, wg, fooSecConn, n/2)

	// write to bar
	writeLots(t, wg, barSecConn, fooWriteText, n)
	wg.Wait()

	if err := fooSecConn.Close(); err != nil {
		t.Error(err)
	}
}

func TestSecretConnectionReadWrite(t *testing.T) {
	fooConn, barConn := makeKVStoreConnPair()
	fooWrites, barWrites := []string{}, []string{}
	fooReads, barReads := []string{}, []string{}

	// Pre-generate the things to write (for foo & bar)
	for i := 0; i < 100; i++ {
		fooWrites = append(fooWrites, cmtrand.Str((cmtrand.Int()%(dataMaxSize*5))+1))
		barWrites = append(barWrites, cmtrand.Str((cmtrand.Int()%(dataMaxSize*5))+1))
	}

	// A helper that will run with (fooConn, fooWrites, fooReads) and vice versa
	genNodeRunner := func(nodeConn kvstoreConn, nodeWrites []string, nodeReads *[]string) async.Task {
		return func(_ int) (any, bool, error) {
			// Initiate cryptographic private key and secret connection through nodeConn.
			nodePrvKey := ed25519.GenPrivKey()
			nodeSecretConn, err := MakeSecretConnection(nodeConn, nodePrvKey)
			if err != nil {
				t.Errorf("failed to establish SecretConnection for node: %v", err)
				return nil, true, err
			}
			// In parallel, handle some reads and writes.
			trs, ok := async.Parallel(
				func(_ int) (any, bool, error) {
					// Node writes:
					for _, nodeWrite := range nodeWrites {
						n, err := nodeSecretConn.Write([]byte(nodeWrite))
						if err != nil {
							t.Errorf("failed to write to nodeSecretConn: %v", err)
							return nil, true, err
						}
						if n != len(nodeWrite) {
							err = fmt.Errorf("failed to write all bytes. Expected %v, wrote %v", len(nodeWrite), n)
							t.Error(err)
							return nil, true, err
						}
					}
					if err := nodeConn.PipeWriter.Close(); err != nil {
						t.Error(err)
						return nil, true, err
					}
					return nil, false, nil
				},
				func(_ int) (any, bool, error) {
					// Node reads:
					readBuffer := make([]byte, dataMaxSize)
					for {
						n, err := nodeSecretConn.Read(readBuffer)
						if err == io.EOF {
							if err := nodeConn.PipeReader.Close(); err != nil {
								t.Error(err)
								return nil, true, err
							}
							return nil, false, nil
						} else if err != nil {
							t.Errorf("failed to read from nodeSecretConn: %v", err)
							return nil, true, err
						}
						*nodeReads = append(*nodeReads, string(readBuffer[:n]))
					}
				},
			)
			assert.True(t, ok, "Unexpected task abortion")

			// If error:
			if trs.FirstError() != nil {
				return nil, true, trs.FirstError()
			}

			// Otherwise:
			return nil, false, nil
		}
	}

	// Run foo & bar in parallel
	trs, ok := async.Parallel(
		genNodeRunner(fooConn, fooWrites, &fooReads),
		genNodeRunner(barConn, barWrites, &barReads),
	)
	require.NoError(t, trs.FirstError())
	require.True(t, ok, "unexpected task abortion")

	// A helper to ensure that the writes and reads match.
	// Additionally, small writes (<= dataMaxSize) must be atomically read.
	compareWritesReads := func(writes []string, reads []string) {
		for {
			// Pop next write & corresponding reads
			read := ""
			write := writes[0]
			readCount := 0
			for _, readChunk := range reads {
				read += readChunk
				readCount++
				if len(write) <= len(read) {
					break
				}
				if len(write) <= dataMaxSize {
					break // atomicity of small writes
				}
			}
			// Compare
			if write != read {
				t.Errorf("expected to read %X, got %X", write, read)
			}
			// Iterate
			writes = writes[1:]
			reads = reads[readCount:]
			if len(writes) == 0 {
				break
			}
		}
	}

	compareWritesReads(fooWrites, barReads)
	compareWritesReads(barWrites, fooReads)
}

func TestDeriveSecretsAndChallengeGolden(t *testing.T) {
	goldenFilepath := filepath.Join("testdata", t.Name()+".golden")
	if *update {
		t.Logf("Updating golden test vector file %s", goldenFilepath)
		data := createGoldenTestVectors(t)
		err := cmtos.WriteFile(goldenFilepath, []byte(data), 0o644)
		require.NoError(t, err)
	}
	f, err := os.Open(goldenFilepath)
	if err != nil {
		log.Fatal(err)
	}
	defer f.Close()
	scanner := bufio.NewScanner(f)
	for scanner.Scan() {
		line := scanner.Text()
		params := strings.Split(line, ",")
		randSecretVector, err := hex.DecodeString(params[0])
		require.NoError(t, err)
		randSecret := new([32]byte)
		copy((*randSecret)[:], randSecretVector)
		locIsLeast, err := strconv.ParseBool(params[1])
		require.NoError(t, err)
		expectedRecvSecret, err := hex.DecodeString(params[2])
		require.NoError(t, err)
		expectedSendSecret, err := hex.DecodeString(params[3])
		require.NoError(t, err)

		recvSecret, sendSecret := deriveSecrets(randSecret, locIsLeast)
		require.Equal(t, expectedRecvSecret, (*recvSecret)[:], "Recv Secrets aren't equal")
		require.Equal(t, expectedSendSecret, (*sendSecret)[:], "Send Secrets aren't equal")
	}
}

func TestNilPubkey(t *testing.T) {
	fooConn, barConn := makeKVStoreConnPair()
	defer fooConn.Close()
	defer barConn.Close()
	fooPrvKey := ed25519.GenPrivKey()
	barPrvKey := privKeyWithNilPubKey{ed25519.GenPrivKey()}

	go MakeSecretConnection(fooConn, fooPrvKey) //nolint:errcheck // ignore for tests

	_, err := MakeSecretConnection(barConn, barPrvKey)
	require.Error(t, err)
	assert.Equal(t, "encoding: unsupported key <nil>", err.Error())
}

func writeLots(t *testing.T, wg *sync.WaitGroup, conn io.Writer, txt string, n int) {
	t.Helper()
	defer wg.Done()
	for i := 0; i < n; i++ {
		_, err := conn.Write([]byte(txt))
		if err != nil {
			t.Errorf("failed to write to fooSecConn: %v", err)
			return
		}
	}
}

func readLots(t *testing.T, wg *sync.WaitGroup, conn io.Reader, n int) {
	t.Helper()
	readBuffer := make([]byte, dataMaxSize)
	for i := 0; i < n; i++ {
		_, err := conn.Read(readBuffer)
		require.NoError(t, err)
	}
	wg.Done()
}

// Creates the data for a test vector file.
// The file format is:
// Hex(diffie_hellman_secret), loc_is_least, Hex(recvSecret), Hex(sendSecret), Hex(challenge).
func createGoldenTestVectors(*testing.T) string {
	data := ""
	for i := 0; i < 32; i++ {
		randSecretVector := cmtrand.Bytes(32)
		randSecret := new([32]byte)
		copy((*randSecret)[:], randSecretVector)
		data += hex.EncodeToString((*randSecret)[:]) + ","
		locIsLeast := cmtrand.Bool()
		data += strconv.FormatBool(locIsLeast) + ","
		recvSecret, sendSecret := deriveSecrets(randSecret, locIsLeast)
		data += hex.EncodeToString((*recvSecret)[:]) + ","
		data += hex.EncodeToString((*sendSecret)[:]) + ","
	}
	return data
}

// Each returned ReadWriteCloser is akin to a net.Connection.
func makeKVStoreConnPair() (fooConn, barConn kvstoreConn) {
	barReader, fooWriter := io.Pipe()
	fooReader, barWriter := io.Pipe()
	return kvstoreConn{fooReader, fooWriter}, kvstoreConn{barReader, barWriter}
}

func makeSecretConnPair(tb testing.TB) (fooSecConn, barSecConn *SecretConnection) {
	tb.Helper()
	var (
		fooConn, barConn = makeKVStoreConnPair()
		fooPrvKey        = ed25519.GenPrivKey()
		fooPubKey        = fooPrvKey.PubKey()
		barPrvKey        = ed25519.GenPrivKey()
		barPubKey        = barPrvKey.PubKey()
	)

	// Make connections from both sides in parallel.
	trs, ok := async.Parallel(
		func(_ int) (val any, abort bool, err error) {
			fooSecConn, err = MakeSecretConnection(fooConn, fooPrvKey)
			if err != nil {
				tb.Errorf("failed to establish SecretConnection for foo: %v", err)
				return nil, true, err
			}
			remotePubBytes := fooSecConn.RemotePubKey()
			if !bytes.Equal(remotePubBytes.Bytes(), barPubKey.Bytes()) {
				err = fmt.Errorf("unexpected fooSecConn.RemotePubKey.  Expected %v, got %v",
					barPubKey, fooSecConn.RemotePubKey())
				tb.Error(err)
				return nil, true, err
			}
			return nil, false, nil
		},
		func(_ int) (val any, abort bool, err error) {
			barSecConn, err = MakeSecretConnection(barConn, barPrvKey)
			if barSecConn == nil {
				tb.Errorf("failed to establish SecretConnection for bar: %v", err)
				return nil, true, err
			}
			remotePubBytes := barSecConn.RemotePubKey()
			if !bytes.Equal(remotePubBytes.Bytes(), fooPubKey.Bytes()) {
				err = fmt.Errorf("unexpected barSecConn.RemotePubKey.  Expected %v, got %v",
					fooPubKey, barSecConn.RemotePubKey())
				tb.Error(err)
				return nil, true, err
			}
			return nil, false, nil
		},
	)

	require.NoError(tb, trs.FirstError())
	require.True(tb, ok, "Unexpected task abortion")

	return fooSecConn, barSecConn
}

// Benchmarks

func BenchmarkWriteSecretConnection(b *testing.B) {
	b.StopTimer()
	b.ReportAllocs()
	fooSecConn, barSecConn := makeSecretConnPair(b)
	randomMsgSizes := []int{
		dataMaxSize / 10,
		dataMaxSize / 3,
		dataMaxSize / 2,
		dataMaxSize,
		dataMaxSize * 3 / 2,
		dataMaxSize * 2,
		dataMaxSize * 7 / 2,
	}
	fooWriteBytes := make([][]byte, 0, len(randomMsgSizes))
	for _, size := range randomMsgSizes {
		fooWriteBytes = append(fooWriteBytes, cmtrand.Bytes(size))
	}
	// Consume reads from bar's reader
	go func() {
		readBuffer := make([]byte, dataMaxSize)
		for {
			_, err := barSecConn.Read(readBuffer)
			if errors.Is(err, io.EOF) {
				return
			} else if err != nil {
				b.Errorf("failed to read from barSecConn: %v", err)
				return
			}
		}
	}()

	b.StartTimer()
	for i := 0; i < b.N; i++ {
		idx := cmtrand.Intn(len(fooWriteBytes))
		_, err := fooSecConn.Write(fooWriteBytes[idx])
		if err != nil {
			b.Errorf("failed to write to fooSecConn: %v", err)
			return
		}
	}
	b.StopTimer()

	if err := fooSecConn.Close(); err != nil {
		b.Error(err)
	}
	// barSecConn.Close() race condition
}

func BenchmarkReadSecretConnection(b *testing.B) {
	b.StopTimer()
	b.ReportAllocs()
	fooSecConn, barSecConn := makeSecretConnPair(b)
	randomMsgSizes := []int{
		dataMaxSize / 10,
		dataMaxSize / 3,
		dataMaxSize / 2,
		dataMaxSize,
		dataMaxSize * 3 / 2,
		dataMaxSize * 2,
		dataMaxSize * 7 / 2,
	}
	fooWriteBytes := make([][]byte, 0, len(randomMsgSizes))
	for _, size := range randomMsgSizes {
		fooWriteBytes = append(fooWriteBytes, cmtrand.Bytes(size))
	}
	go func() {
		for i := 0; i < b.N; i++ {
			idx := cmtrand.Intn(len(fooWriteBytes))
			_, err := fooSecConn.Write(fooWriteBytes[idx])
			if err != nil {
				b.Errorf("failed to write to fooSecConn: %v, %v,%v", err, i, b.N)
				return
			}
		}
	}()

	b.StartTimer()
	for i := 0; i < b.N; i++ {
		readBuffer := make([]byte, dataMaxSize)
		_, err := barSecConn.Read(readBuffer)

		if errors.Is(err, io.EOF) {
			return
		} else if err != nil {
			b.Fatalf("Failed to read from barSecConn: %v", err)
		}
	}
	b.StopTimer()
}
