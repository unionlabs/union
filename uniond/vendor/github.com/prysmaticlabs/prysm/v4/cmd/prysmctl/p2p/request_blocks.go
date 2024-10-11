package p2p

import (
	"context"
	"strings"
	"time"

	libp2pcore "github.com/libp2p/go-libp2p/core"
	corenet "github.com/libp2p/go-libp2p/core/network"
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p"
	p2ptypes "github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/types"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/sync"
	"github.com/prysmaticlabs/prysm/v4/cmd"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	consensus_types "github.com/prysmaticlabs/prysm/v4/consensus-types"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	pb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
	"github.com/sirupsen/logrus"
	"github.com/urfave/cli/v2"
	"google.golang.org/protobuf/types/known/emptypb"
)

var requestBlocksFlags = struct {
	Peers        string
	ClientPort   uint
	APIEndpoints string
	StartSlot    uint64
	Count        uint64
	Step         uint64
}{}

var requestBlocksCmd = &cli.Command{
	Name:  "beacon-blocks-by-range",
	Usage: "Request a range of blocks from a beacon node via a p2p connection",
	Action: func(cliCtx *cli.Context) error {
		if err := cliActionRequestBlocks(cliCtx); err != nil {
			log.WithError(err).Fatal("Could not request blocks by range")
		}
		return nil
	},
	Flags: []cli.Flag{
		cmd.ChainConfigFileFlag,
		&cli.StringFlag{
			Name:        "peer-multiaddrs",
			Usage:       "comma-separated, peer multiaddr(s) to connect to for p2p requests",
			Destination: &requestBlocksFlags.Peers,
			Value:       "",
		},
		&cli.UintFlag{
			Name:        "client-port",
			Usage:       "port to use for the client as a libp2p host",
			Destination: &requestBlocksFlags.ClientPort,
			Value:       13001,
		},
		&cli.StringFlag{
			Name:        "prysm-api-endpoints",
			Usage:       "comma-separated, gRPC API endpoint(s) for Prysm beacon node(s)",
			Destination: &requestBlocksFlags.APIEndpoints,
			Value:       "localhost:4000",
		},
		&cli.Uint64Flag{
			Name:        "start-slot",
			Usage:       "start slot for blocks by range request. If unset, will use start_slot(current_epoch-1)",
			Destination: &requestBlocksFlags.StartSlot,
			Value:       0,
		},
		&cli.Uint64Flag{
			Name:        "count",
			Usage:       "number of blocks to request, (default 32)",
			Destination: &requestBlocksFlags.Count,
			Value:       32,
		},
		&cli.Uint64Flag{
			Name:        "step",
			Usage:       "number of steps of blocks in the range request, (default 1)",
			Destination: &requestBlocksFlags.Step,
			Value:       1,
		},
	},
}

func cliActionRequestBlocks(cliCtx *cli.Context) error {
	if cliCtx.IsSet(cmd.ChainConfigFileFlag.Name) {
		chainConfigFileName := cliCtx.String(cmd.ChainConfigFileFlag.Name)
		if err := params.LoadChainConfigFile(chainConfigFileName, nil); err != nil {
			return err
		}
	}
	p2ptypes.InitializeDataMaps()

	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Minute)
	defer cancel()

	allAPIEndpoints := make([]string, 0)
	if requestBlocksFlags.APIEndpoints != "" {
		allAPIEndpoints = strings.Split(requestBlocksFlags.APIEndpoints, ",")
	}
	var err error
	c, err := newClient(allAPIEndpoints, requestBlocksFlags.ClientPort)
	if err != nil {
		return err
	}
	defer c.Close()

	allPeers := make([]string, 0)
	if requestBlocksFlags.Peers != "" {
		allPeers = strings.Split(requestBlocksFlags.Peers, ",")
	}
	if len(allPeers) == 0 {
		allPeers, err = c.retrievePeerAddressesViaRPC(ctx, allAPIEndpoints)
		if err != nil {
			return err
		}
	}
	if len(allPeers) == 0 {
		return errors.New("no peers found")
	}
	log.WithField("peers", allPeers).Info("List of peers")
	chain, err := c.initializeMockChainService(ctx)
	if err != nil {
		return err
	}
	c.registerHandshakeHandlers()

	c.registerRPCHandler(p2p.RPCBlocksByRangeTopicV1, func(
		ctx context.Context, i interface{}, stream libp2pcore.Stream,
	) error {
		return nil
	})
	c.registerRPCHandler(p2p.RPCBlocksByRangeTopicV2, func(
		ctx context.Context, i interface{}, stream libp2pcore.Stream,
	) error {
		return nil
	})

	if err := c.connectToPeers(ctx, allPeers...); err != nil {
		return err
	}

	startSlot := primitives.Slot(requestBlocksFlags.StartSlot)
	var headSlot *primitives.Slot
	if startSlot == 0 {
		headResp, err := c.beaconClient.GetChainHead(ctx, &emptypb.Empty{})
		if err != nil {
			return err
		}
		startSlot, err = slots.EpochStart(headResp.HeadEpoch.Sub(1))
		if err != nil {
			return err
		}
		headSlot = &headResp.HeadSlot
	}

	// Submit requests.
	for _, pr := range c.host.Peerstore().Peers() {
		if pr.String() == c.host.ID().String() {
			continue
		}
		req := &pb.BeaconBlocksByRangeRequest{
			StartSlot: startSlot,
			Count:     requestBlocksFlags.Count,
			Step:      requestBlocksFlags.Step,
		}
		fields := logrus.Fields{
			"startSlot": startSlot,
			"count":     requestBlocksFlags.Count,
			"step":      requestBlocksFlags.Step,
			"peer":      pr.String(),
		}
		if headSlot != nil {
			fields["headSlot"] = *headSlot
		}
		log.WithFields(fields).Info("Sending blocks by range p2p request to peer")
		start := time.Now()
		blocks, err := sync.SendBeaconBlocksByRangeRequest(
			ctx,
			chain,
			c,
			pr,
			req,
			nil, /* no extra block processing */
		)
		if err != nil {
			return err
		}
		end := time.Since(start)
		totalExecutionBlocks := 0
		for _, blk := range blocks {
			exec, err := blk.Block().Body().Execution()
			switch {
			case errors.Is(err, consensus_types.ErrUnsupportedField):
				continue
			case err != nil:
				log.WithError(err).Error("Could not read execution data from block body")
				continue
			default:
			}
			_, err = exec.Transactions()
			switch {
			case errors.Is(err, consensus_types.ErrUnsupportedField):
				continue
			case err != nil:
				log.WithError(err).Error("Could not read transactions block execution payload")
				continue
			default:
			}
			totalExecutionBlocks++
		}
		log.WithFields(logrus.Fields{
			"numBlocks":                           len(blocks),
			"peer":                                pr.String(),
			"timeFromSendingToProcessingResponse": end,
			"totalBlocksWithExecutionPayloads":    totalExecutionBlocks,
		}).Info("Received blocks from peer")
	}
	return nil
}

func closeStream(stream corenet.Stream) {
	if err := stream.Close(); err != nil {
		log.Println(err)
	}
}
