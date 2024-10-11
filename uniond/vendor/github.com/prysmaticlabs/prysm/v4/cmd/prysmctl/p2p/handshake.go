package p2p

import (
	"context"

	libp2pcore "github.com/libp2p/go-libp2p/core"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/network/forks"
	pb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
	"github.com/sirupsen/logrus"
	"google.golang.org/protobuf/types/known/emptypb"
)

var responseCodeSuccess = byte(0x00)

func (c *client) registerHandshakeHandlers() {
	c.registerRPCHandler(p2p.RPCPingTopicV1, c.pingHandler)
	c.registerRPCHandler(p2p.RPCStatusTopicV1, c.statusRPCHandler)
	c.registerRPCHandler(p2p.RPCGoodByeTopicV1, c.goodbyeHandler)
}

// pingHandler reads the incoming ping rpc message from the peer.
func (c *client) pingHandler(_ context.Context, _ interface{}, stream libp2pcore.Stream) error {
	defer closeStream(stream)
	if _, err := stream.Write([]byte{responseCodeSuccess}); err != nil {
		return err
	}
	sq := primitives.SSZUint64(c.MetadataSeq())
	if _, err := c.Encoding().EncodeWithMaxLength(stream, &sq); err != nil {
		return err
	}
	return nil
}

func (c *client) goodbyeHandler(_ context.Context, _ interface{}, _ libp2pcore.Stream) error {
	return nil
}

// statusRPCHandler reads the incoming Status RPC from the peer and responds with our version of a status message.
// This handler will disconnect any peer that does not match our fork version.
func (c *client) statusRPCHandler(ctx context.Context, _ interface{}, stream libp2pcore.Stream) error {
	defer closeStream(stream)
	chainHead, err := c.beaconClient.GetChainHead(ctx, &emptypb.Empty{})
	if err != nil {
		return err
	}
	resp, err := c.nodeClient.GetGenesis(ctx, &emptypb.Empty{})
	if err != nil {
		return err
	}
	digest, err := forks.CreateForkDigest(resp.GenesisTime.AsTime(), resp.GenesisValidatorsRoot)
	if err != nil {
		return err
	}
	kindOfFork, err := forks.Fork(slots.ToEpoch(chainHead.HeadSlot))
	if err != nil {
		return err
	}
	log.WithFields(logrus.Fields{
		"genesisTime":  resp.GenesisTime.AsTime(),
		"forkDigest":   digest,
		"currentFork":  kindOfFork.CurrentVersion,
		"previousFork": kindOfFork.PreviousVersion,
	}).Info("Responding to status RPC handler")
	status := &pb.Status{
		ForkDigest:     digest[:],
		FinalizedRoot:  chainHead.FinalizedBlockRoot,
		FinalizedEpoch: chainHead.FinalizedEpoch,
		HeadRoot:       chainHead.HeadBlockRoot,
		HeadSlot:       chainHead.HeadSlot,
	}

	if _, err := stream.Write([]byte{responseCodeSuccess}); err != nil {
		log.WithError(err).Debug("Could not write to stream")
		return err
	}
	_, err = c.Encoding().EncodeWithMaxLength(stream, status)
	return err
}
