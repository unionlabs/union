package p2p

import (
	"context"
	"fmt"
	"strings"
	"time"

	libp2pcore "github.com/libp2p/go-libp2p/core"
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p"
	p2ptypes "github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/types"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/sync"
	"github.com/prysmaticlabs/prysm/v4/cmd"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	pb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
	"github.com/sirupsen/logrus"
	"github.com/urfave/cli/v2"
	"google.golang.org/protobuf/types/known/emptypb"
)

var requestBlobsFlags = struct {
	Peers        string
	ClientPort   uint
	APIEndpoints string
	StartSlot    uint64
	Count        uint64
}{}

var requestBlobsCmd = &cli.Command{
	Name:  "blobs-by-range",
	Usage: "Request a range of blobs from a beacon node via a p2p connection",
	Action: func(cliCtx *cli.Context) error {
		if err := cliActionRequestBlobs(cliCtx); err != nil {
			log.WithError(err).Fatal("Could not request blobs by range")
		}
		return nil
	},
	Flags: []cli.Flag{
		cmd.ChainConfigFileFlag,
		&cli.StringFlag{
			Name:        "peer-multiaddrs",
			Usage:       "comma-separated, peer multiaddr(s) to connect to for p2p requests",
			Destination: &requestBlobsFlags.Peers,
			Value:       "",
		},
		&cli.UintFlag{
			Name:        "client-port",
			Usage:       "port to use for the client as a libp2p host",
			Destination: &requestBlobsFlags.ClientPort,
			Value:       13001,
		},
		&cli.StringFlag{
			Name:        "prysm-api-endpoints",
			Usage:       "comma-separated, gRPC API endpoint(s) for Prysm beacon node(s)",
			Destination: &requestBlobsFlags.APIEndpoints,
			Value:       "localhost:4000",
		},
		&cli.Uint64Flag{
			Name:        "start-slot",
			Usage:       "start slot for blocks by range request. If unset, will use start_slot(current_epoch-1)",
			Destination: &requestBlobsFlags.StartSlot,
			Value:       0,
		},
		&cli.Uint64Flag{
			Name:        "count",
			Usage:       "number of blocks to request, (default 32)",
			Destination: &requestBlobsFlags.Count,
			Value:       32,
		},
	},
}

func cliActionRequestBlobs(cliCtx *cli.Context) error {
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
	if requestBlobsFlags.APIEndpoints != "" {
		allAPIEndpoints = strings.Split(requestBlobsFlags.APIEndpoints, ",")
	}
	var err error
	c, err := newClient(allAPIEndpoints, requestBlobsFlags.ClientPort)
	if err != nil {
		return err
	}
	defer c.Close()

	allPeers := make([]string, 0)
	if requestBlobsFlags.Peers != "" {
		allPeers = strings.Split(requestBlobsFlags.Peers, ",")
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

	c.registerRPCHandler(p2p.RPCBlobSidecarsByRangeTopicV1, func(
		ctx context.Context, i interface{}, stream libp2pcore.Stream,
	) error {
		return nil
	})

	if err := c.connectToPeers(ctx, allPeers...); err != nil {
		return errors.Wrap(err, "could not connect to peers")
	}

	startSlot := primitives.Slot(requestBlobsFlags.StartSlot)
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
		req := &pb.BlobSidecarsByRangeRequest{
			StartSlot: startSlot,
			Count:     requestBlobsFlags.Count,
		}
		fields := logrus.Fields{
			"startSlot": startSlot,
			"count":     requestBlobsFlags.Count,
			"peer":      pr.String(),
		}
		if headSlot != nil {
			fields["headSlot"] = *headSlot
		}

		ctxByte, err := sync.ContextByteVersionsForValRoot(chain.genesisValsRoot)
		if err != nil {
			return err
		}

		log.WithFields(fields).Info("Blobs by range p2p request to peer")
		blobs, err := sync.SendBlobsByRangeRequest(
			ctx,
			chain,
			c,
			pr,
			ctxByte,
			req,
		)
		if err != nil {
			return err
		}
		for _, b := range blobs {
			log.WithFields(logrus.Fields{
				"slot":       b.Slot,
				"index":      b.Index,
				"commitment": fmt.Sprintf("%#x", b.KzgCommitment),
				"kzgProof":   fmt.Sprintf("%#x", b.KzgProof),
			}).Info("Received blob sidecar")
		}
		log.WithFields(logrus.Fields{
			"numBlobs": len(blobs),
			"peer":     pr.String(),
		}).Info("Received blob sidecars from peer")
	}
	return nil
}
