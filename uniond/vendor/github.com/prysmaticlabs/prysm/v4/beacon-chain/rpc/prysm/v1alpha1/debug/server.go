// Package debug defines a gRPC server implementation of a debugging service
// which allows for helpful endpoints to debug a beacon node at runtime, this server is
// gated behind the feature flag --enable-debug-rpc-endpoints.
package debug

import (
	"context"
	"os"

	gethlog "github.com/ethereum/go-ethereum/log"
	"github.com/golang/protobuf/ptypes/empty"
	golog "github.com/ipfs/go-log/v2"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/blockchain"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/db"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state/stategen"
	pbrpc "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/sirupsen/logrus"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
)

// Server defines a server implementation of the gRPC Debug service,
// providing RPC endpoints for runtime debugging of a node, this server is
// gated behind the feature flag --enable-debug-rpc-endpoints.
type Server struct {
	BeaconDB           db.NoHeadAccessDatabase
	GenesisTimeFetcher blockchain.TimeFetcher
	StateGen           *stategen.State
	HeadFetcher        blockchain.HeadFetcher
	PeerManager        p2p.PeerManager
	PeersFetcher       p2p.PeersProvider
	ReplayerBuilder    stategen.ReplayerBuilder
}

// SetLoggingLevel of a beacon node according to a request type,
// either INFO, DEBUG, or TRACE.
func (_ *Server) SetLoggingLevel(_ context.Context, req *pbrpc.LoggingLevelRequest) (*empty.Empty, error) {
	var verbosity string
	switch req.Level {
	case pbrpc.LoggingLevelRequest_INFO:
		verbosity = "info"
	case pbrpc.LoggingLevelRequest_DEBUG:
		verbosity = "debug"
	case pbrpc.LoggingLevelRequest_TRACE:
		verbosity = "trace"
	default:
		return nil, status.Error(codes.InvalidArgument, "Expected valid verbosity level as argument")
	}
	level, err := logrus.ParseLevel(verbosity)
	if err != nil {
		return nil, status.Error(codes.Internal, "Could not parse verbosity level")
	}
	logrus.SetLevel(level)
	if level == logrus.TraceLevel {
		// Libp2p specific logging.
		golog.SetAllLoggers(golog.LevelDebug)
		// Geth specific logging.
		glogger := gethlog.NewGlogHandler(gethlog.StreamHandler(os.Stderr, gethlog.TerminalFormat(true)))
		glogger.Verbosity(gethlog.LvlTrace)
		gethlog.Root().SetHandler(glogger)
	}
	return &empty.Empty{}, nil
}
