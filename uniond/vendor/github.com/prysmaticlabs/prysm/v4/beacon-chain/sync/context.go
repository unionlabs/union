package sync

import (
	"github.com/libp2p/go-libp2p/core/network"
	"github.com/libp2p/go-libp2p/core/protocol"
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/signing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p"
	"github.com/prysmaticlabs/prysm/v4/config/params"
)

// Specifies the fixed size context length.
const forkDigestLength = 4

// writes peer's current context for the expected payload to the stream.
func writeContextToStream(objCtx []byte, stream network.Stream) error {
	// The rpc context for our v2 methods is the fork-digest of
	// the relevant payload. We write the associated fork-digest(context)
	// into the stream for the payload.
	rpcCtx, err := expectRpcContext(stream)
	if err != nil {
		return err
	}
	// Exit early if an empty context is expected.
	if !rpcCtx {
		return nil
	}
	_, err = stream.Write(objCtx)
	return err
}

// reads any attached context-bytes to the payload.
func readContextFromStream(stream network.Stream) ([]byte, error) {
	hasCtx, err := expectRpcContext(stream)
	if err != nil {
		return nil, err
	}
	if !hasCtx {
		return []byte{}, nil
	}
	// Read context (fork-digest) from stream
	b := make([]byte, forkDigestLength)
	if _, err := stream.Read(b); err != nil {
		return nil, err
	}
	return b, nil
}

func expectRpcContext(stream network.Stream) (bool, error) {
	_, message, version, err := p2p.TopicDeconstructor(string(stream.Protocol()))
	if err != nil {
		return false, err
	}
	// For backwards compatibility, we want to omit context bytes for certain v1 methods that were defined before
	// context bytes were introduced into the protocol.
	if version == p2p.SchemaVersionV1 && p2p.OmitContextBytesV1[message] {
		return false, nil
	}
	return true, nil
}

// Minimal interface for a stream with a protocol.
type withProtocol interface {
	Protocol() protocol.ID
}

// Validates that the rpc topic matches the provided version.
func validateVersion(version string, stream withProtocol) error {
	_, _, streamVersion, err := p2p.TopicDeconstructor(string(stream.Protocol()))
	if err != nil {
		return err
	}
	if streamVersion != version {
		return errors.Errorf("stream version of %s doesn't match provided version %s", streamVersion, version)
	}
	return nil
}

// ContextByteVersions is a mapping between expected values for context bytes
// and the runtime/version identifier they correspond to. This can be used to look up the type
// needed to unmarshal a wire-encoded value.
type ContextByteVersions map[[4]byte]int

// ContextByteVersionsForValRoot computes a mapping between all possible context bytes values
// and the runtime/version identifier for the corresponding fork.
func ContextByteVersionsForValRoot(valRoot [32]byte) (ContextByteVersions, error) {
	m := make(ContextByteVersions)
	for fv, v := range params.ConfigForkVersions(params.BeaconConfig()) {
		digest, err := signing.ComputeForkDigest(fv[:], valRoot[:])
		if err != nil {
			return nil, errors.Wrapf(err, "unable to compute fork digest for fork version %#x", fv)
		}
		m[digest] = v
	}
	return m, nil
}
