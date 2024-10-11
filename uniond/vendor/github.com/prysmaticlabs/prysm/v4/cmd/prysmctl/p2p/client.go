package p2p

import (
	"context"
	"crypto/ecdsa"
	"crypto/rand"
	"net"
	"time"

	"github.com/libp2p/go-libp2p"
	"github.com/libp2p/go-libp2p/core/crypto"
	"github.com/libp2p/go-libp2p/core/host"
	corenet "github.com/libp2p/go-libp2p/core/network"
	"github.com/libp2p/go-libp2p/core/peer"
	"github.com/libp2p/go-libp2p/core/protocol"
	"github.com/libp2p/go-libp2p/p2p/security/noise"
	"github.com/libp2p/go-libp2p/p2p/transport/tcp"
	"github.com/pkg/errors"
	ssz "github.com/prysmaticlabs/fastssz"
	"github.com/prysmaticlabs/go-bitfield"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/encoder"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/wrapper"
	ecdsaprysm "github.com/prysmaticlabs/prysm/v4/crypto/ecdsa"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	"github.com/prysmaticlabs/prysm/v4/monitoring/tracing"
	"github.com/prysmaticlabs/prysm/v4/network"
	"github.com/prysmaticlabs/prysm/v4/network/forks"
	pb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1/metadata"
	"github.com/prysmaticlabs/prysm/v4/runtime/version"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
	"go.opencensus.io/trace"
	"google.golang.org/grpc"
	"google.golang.org/protobuf/types/known/emptypb"
)

// A minimal client for peering with beacon nodes over libp2p and sending p2p RPC requests for data.
type client struct {
	host         host.Host
	meta         metadata.Metadata
	beaconClient pb.BeaconChainClient
	nodeClient   pb.NodeClient
}

func newClient(beaconEndpoints []string, clientPort uint) (*client, error) {
	ipAdd := ipAddr()
	priv, err := privKey()
	if err != nil {
		return nil, errors.Wrap(err, "could not set up p2p private key")
	}
	meta, err := readMetadata()
	if err != nil {
		return nil, errors.Wrap(err, "could not set up p2p metadata")
	}
	listen, err := p2p.MultiAddressBuilder(ipAdd.String(), clientPort)
	if err != nil {
		return nil, errors.Wrap(err, "could not set up listening multiaddr")
	}
	options := []libp2p.Option{
		privKeyOption(priv),
		libp2p.ListenAddrs(listen),
		libp2p.UserAgent(version.BuildData()),
		libp2p.Transport(tcp.NewTCPTransport),
	}
	options = append(options, libp2p.Security(noise.ID, noise.New))
	options = append(options, libp2p.Ping(false))
	h, err := libp2p.New(options...)
	if err != nil {
		return nil, errors.Wrap(err, "could not start libp2p")
	}
	if len(beaconEndpoints) == 0 {
		return nil, errors.New("no specified beacon API endpoints")
	}
	conn, err := grpc.Dial(beaconEndpoints[0], grpc.WithInsecure())
	if err != nil {
		return nil, err
	}
	beaconClient := pb.NewBeaconChainClient(conn)
	nodeClient := pb.NewNodeClient(conn)
	return &client{
		host:         h,
		meta:         meta,
		beaconClient: beaconClient,
		nodeClient:   nodeClient,
	}, nil
}

func (c *client) Close() {
	if err := c.host.Close(); err != nil {
		panic(err)
	}
}

func (c *client) Encoding() encoder.NetworkEncoding {
	return &encoder.SszNetworkEncoder{}
}

func (c *client) MetadataSeq() uint64 {
	return c.meta.SequenceNumber()
}

// Send a request to specific peer. The returned stream may be used for reading,
// but has been closed for writing.
// When done, the caller must Close() or Reset() on the stream.
func (c *client) Send(
	ctx context.Context,
	message interface{},
	baseTopic string,
	pid peer.ID,
) (corenet.Stream, error) {
	ctx, span := trace.StartSpan(ctx, "p2p.Send")
	defer span.End()
	topic := baseTopic + c.Encoding().ProtocolSuffix()
	span.AddAttributes(trace.StringAttribute("topic", topic))

	// Apply max dial timeout when opening a new stream.
	ctx, cancel := context.WithTimeout(ctx, 10*time.Minute)
	defer cancel()

	stream, err := c.host.NewStream(ctx, pid, protocol.ID(topic))
	if err != nil {
		tracing.AnnotateError(span, err)
		return nil, errors.Wrap(err, "could not open new stream")
	}
	// do not encode anything if we are sending a metadata request
	if baseTopic != p2p.RPCMetaDataTopicV1 && baseTopic != p2p.RPCMetaDataTopicV2 {
		castedMsg, ok := message.(ssz.Marshaler)
		if !ok {
			return nil, errors.Errorf("%T does not support the ssz marshaller interface", message)
		}
		if _, err := c.Encoding().EncodeWithMaxLength(stream, castedMsg); err != nil {
			tracing.AnnotateError(span, err)
			_err := stream.Reset()
			_ = _err
			return nil, err
		}
	}
	// Close stream for writing.
	if err := stream.CloseWrite(); err != nil {
		tracing.AnnotateError(span, err)
		_err := stream.Reset()
		_ = _err
		return nil, errors.Wrap(err, "could not close write")
	}

	return stream, nil
}

func (c *client) retrievePeerAddressesViaRPC(ctx context.Context, beaconEndpoints []string) ([]string, error) {
	if len(beaconEndpoints) == 0 {
		return nil, errors.New("no beacon RPC endpoints specified")
	}
	peers := make([]string, 0)
	for i := 0; i < len(beaconEndpoints); i++ {
		conn, err := grpc.Dial(beaconEndpoints[i], grpc.WithInsecure())
		if err != nil {
			return nil, err
		}
		nodeClient := pb.NewNodeClient(conn)
		hostData, err := nodeClient.GetHost(ctx, &emptypb.Empty{})
		if err != nil {
			return nil, err
		}
		if len(hostData.Addresses) == 0 {
			continue
		}
		peers = append(peers, hostData.Addresses[0]+"/p2p/"+hostData.PeerId)
	}
	return peers, nil
}

func (c *client) initializeMockChainService(ctx context.Context) (*mockChain, error) {
	genesisResp, err := c.nodeClient.GetGenesis(ctx, &emptypb.Empty{})
	if err != nil {
		return nil, err
	}
	currEpoch := slots.ToEpoch(slots.SinceGenesis(genesisResp.GenesisTime.AsTime()))
	currFork, err := forks.Fork(currEpoch)
	if err != nil {
		return nil, err
	}
	return &mockChain{
		genesisTime:     genesisResp.GenesisTime.AsTime(),
		currentFork:     currFork,
		genesisValsRoot: bytesutil.ToBytes32(genesisResp.GenesisValidatorsRoot),
	}, nil
}

// Retrieves an external ipv4 address and converts into a libp2p formatted value.
func ipAddr() net.IP {
	ip, err := network.ExternalIP()
	if err != nil {
		panic(err)
	}
	return net.ParseIP(ip)
}

// Determines a private key for p2p networking from the p2p service's
// configuration struct. If no key is found, it generates a new one.
func privKey() (*ecdsa.PrivateKey, error) {
	priv, _, err := crypto.GenerateSecp256k1Key(rand.Reader)
	if err != nil {
		return nil, err
	}
	return ecdsaprysm.ConvertFromInterfacePrivKey(priv)
}

// Adds a private key to the libp2p option if the option was provided.
// If the private key file is missing or cannot be read, or if the
// private key contents cannot be marshaled, an exception is thrown.
func privKeyOption(privkey *ecdsa.PrivateKey) libp2p.Option {
	return func(cfg *libp2p.Config) error {
		ifaceKey, err := ecdsaprysm.ConvertToInterfacePrivkey(privkey)
		if err != nil {
			return err
		}
		return cfg.Apply(libp2p.Identity(ifaceKey))
	}
}

func readMetadata() (metadata.Metadata, error) {
	metaData := &pb.MetaDataV1{
		SeqNumber: 0,
		Attnets:   bitfield.NewBitvector64(),
	}
	return wrapper.WrappedMetadataV1(metaData), nil
}
