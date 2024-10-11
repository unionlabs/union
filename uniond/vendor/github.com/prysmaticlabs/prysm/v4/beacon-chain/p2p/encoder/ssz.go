package encoder

import (
	"fmt"
	"io"
	"sync"

	"github.com/gogo/protobuf/proto"
	"github.com/golang/snappy"
	"github.com/pkg/errors"
	fastssz "github.com/prysmaticlabs/fastssz"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/math"
)

var _ NetworkEncoding = (*SszNetworkEncoder)(nil)

// MaxGossipSize allowed for gossip messages.
var MaxGossipSize = params.BeaconConfig().GossipMaxSize // 10 Mib.
var MaxChunkSize = params.BeaconConfig().MaxChunkSize   // 10 Mib.

// This pool defines the sync pool for our buffered snappy writers, so that they
// can be constantly reused.
var bufWriterPool = new(sync.Pool)

// This pool defines the sync pool for our buffered snappy readers, so that they
// can be constantly reused.
var bufReaderPool = new(sync.Pool)

// SszNetworkEncoder supports p2p networking encoding using SimpleSerialize
// with snappy compression (if enabled).
type SszNetworkEncoder struct{}

// ProtocolSuffixSSZSnappy is the last part of the topic string to identify the encoding protocol.
const ProtocolSuffixSSZSnappy = "ssz_snappy"

// EncodeGossip the proto gossip message to the io.Writer.
func (_ SszNetworkEncoder) EncodeGossip(w io.Writer, msg fastssz.Marshaler) (int, error) {
	if msg == nil {
		return 0, nil
	}
	b, err := msg.MarshalSSZ()
	if err != nil {
		return 0, err
	}
	if uint64(len(b)) > MaxGossipSize {
		return 0, errors.Errorf("gossip message exceeds max gossip size: %d bytes > %d bytes", len(b), MaxGossipSize)
	}
	b = snappy.Encode(nil /*dst*/, b)
	return w.Write(b)
}

// EncodeWithMaxLength the proto message to the io.Writer. This encoding prefixes the byte slice with a protobuf varint
// to indicate the size of the message. This checks that the encoded message isn't larger than the provided max limit.
func (_ SszNetworkEncoder) EncodeWithMaxLength(w io.Writer, msg fastssz.Marshaler) (int, error) {
	if msg == nil {
		return 0, nil
	}
	b, err := msg.MarshalSSZ()
	if err != nil {
		return 0, err
	}
	if uint64(len(b)) > MaxChunkSize {
		return 0, fmt.Errorf(
			"size of encoded message is %d which is larger than the provided max limit of %d",
			len(b),
			MaxChunkSize,
		)
	}
	// write varint first
	_, err = w.Write(proto.EncodeVarint(uint64(len(b))))
	if err != nil {
		return 0, err
	}
	return writeSnappyBuffer(w, b)
}

func doDecode(b []byte, to fastssz.Unmarshaler) error {
	return to.UnmarshalSSZ(b)
}

// DecodeGossip decodes the bytes to the protobuf gossip message provided.
func (_ SszNetworkEncoder) DecodeGossip(b []byte, to fastssz.Unmarshaler) error {
	b, err := DecodeSnappy(b, MaxGossipSize)
	if err != nil {
		return err
	}
	return doDecode(b, to)
}

// DecodeSnappy decodes a snappy compressed message.
func DecodeSnappy(msg []byte, maxSize uint64) ([]byte, error) {
	size, err := snappy.DecodedLen(msg)
	if err != nil {
		return nil, err
	}
	if uint64(size) > maxSize {
		return nil, errors.Errorf("snappy message exceeds max size: %d bytes > %d bytes", size, maxSize)
	}
	msg, err = snappy.Decode(nil /*dst*/, msg)
	if err != nil {
		return nil, err
	}
	return msg, nil
}

// DecodeWithMaxLength the bytes from io.Reader to the protobuf message provided.
// This checks that the decoded message isn't larger than the provided max limit.
func (e SszNetworkEncoder) DecodeWithMaxLength(r io.Reader, to fastssz.Unmarshaler) error {
	msgLen, err := readVarint(r)
	if err != nil {
		return err
	}
	if msgLen > MaxChunkSize {
		return fmt.Errorf(
			"remaining bytes %d goes over the provided max limit of %d",
			msgLen,
			MaxChunkSize,
		)
	}
	msgMax, err := e.MaxLength(msgLen)
	if err != nil {
		return err
	}
	limitedRdr := io.LimitReader(r, int64(msgMax))
	r = newBufferedReader(limitedRdr)
	defer bufReaderPool.Put(r)

	buf := make([]byte, msgLen)
	// Returns an error if less than msgLen bytes
	// are read. This ensures we read exactly the
	// required amount.
	_, err = io.ReadFull(r, buf)
	if err != nil {
		return err
	}
	return doDecode(buf, to)
}

// ProtocolSuffix returns the appropriate suffix for protocol IDs.
func (_ SszNetworkEncoder) ProtocolSuffix() string {
	return "/" + ProtocolSuffixSSZSnappy
}

// MaxLength specifies the maximum possible length of an encoded
// chunk of data.
func (_ SszNetworkEncoder) MaxLength(length uint64) (int, error) {
	il, err := math.Int(length)
	if err != nil {
		return 0, errors.Wrap(err, "invalid length provided")
	}
	maxLen := snappy.MaxEncodedLen(il)
	if maxLen < 0 {
		return 0, errors.Errorf("max encoded length is negative: %d", maxLen)
	}
	return maxLen, nil
}

// Writes a bytes value through a snappy buffered writer.
func writeSnappyBuffer(w io.Writer, b []byte) (int, error) {
	bufWriter := newBufferedWriter(w)
	defer bufWriterPool.Put(bufWriter)
	num, err := bufWriter.Write(b)
	if err != nil {
		// Close buf writer in the event of an error.
		if err := bufWriter.Close(); err != nil {
			return 0, err
		}
		return 0, err
	}
	return num, bufWriter.Close()
}

// Instantiates a new instance of the snappy buffered reader
// using our sync pool.
func newBufferedReader(r io.Reader) *snappy.Reader {
	rawReader := bufReaderPool.Get()
	if rawReader == nil {
		return snappy.NewReader(r)
	}
	bufR, ok := rawReader.(*snappy.Reader)
	if !ok {
		return snappy.NewReader(r)
	}
	bufR.Reset(r)
	return bufR
}

// Instantiates a new instance of the snappy buffered writer
// using our sync pool.
func newBufferedWriter(w io.Writer) *snappy.Writer {
	rawBufWriter := bufWriterPool.Get()
	if rawBufWriter == nil {
		return snappy.NewBufferedWriter(w)
	}
	bufW, ok := rawBufWriter.(*snappy.Writer)
	if !ok {
		return snappy.NewBufferedWriter(w)
	}
	bufW.Reset(w)
	return bufW
}
