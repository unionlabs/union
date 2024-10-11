package main

import (
	"context"
	"flag"
	"fmt"
	"math"
	"time"

	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/interfaces"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	v1alpha1 "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
	"golang.org/x/sync/errgroup"
	"google.golang.org/grpc"
)

var (
	beacon  = flag.String("beacon", "127.0.0.1:4000", "gRPC address of the Prysm beacon node")
	genesis = flag.Uint64("genesis", 1606824023, "Genesis time. mainnet=1606824023, prater=1616508000")
)

func main() {
	flag.Parse()
	ctx := context.Background()

	cc, err := grpc.DialContext(ctx, *beacon, grpc.WithInsecure(), grpc.WithDefaultCallOptions(grpc.MaxCallRecvMsgSize(math.MaxInt64)))
	if err != nil {
		panic(err)
	}
	c := v1alpha1.NewBeaconChainClient(cc)
	g, ctx := errgroup.WithContext(ctx)
	v := newVotes()

	current := slots.ToEpoch(slots.CurrentSlot(*genesis))
	start := current.Div(uint64(params.BeaconConfig().EpochsPerEth1VotingPeriod)).Mul(uint64(params.BeaconConfig().EpochsPerEth1VotingPeriod))
	nextStart := start.AddEpoch(params.BeaconConfig().EpochsPerEth1VotingPeriod)

	fmt.Printf("Looking back from current epoch %d back to %d\n", current, start)
	nextStartSlot, err := slots.EpochStart(nextStart)
	if err != nil {
		panic(err)
	}
	nextStartTime, err := slots.ToTime(*genesis, nextStartSlot)
	if err != nil {
		panic(err)
	}
	fmt.Printf("Next period starts at epoch %d (%s)\n", nextStart, time.Until(nextStartTime))

	for i := primitives.Epoch(0); i < current.Sub(uint64(start)); i++ {
		j := i
		g.Go(func() error {
			resp, err := c.ListBeaconBlocks(ctx, &v1alpha1.ListBlocksRequest{
				QueryFilter: &v1alpha1.ListBlocksRequest_Epoch{Epoch: current.Sub(uint64(j))},
			})
			if err != nil {
				return err
			}
			for _, c := range resp.GetBlockContainers() {
				v.Insert(wrapBlock(c))
			}

			return nil
		})
	}

	if err := g.Wait(); err != nil {
		panic(err)
	}

	fmt.Println(v.Report())
}

func wrapBlock(b *v1alpha1.BeaconBlockContainer) interfaces.ReadOnlyBeaconBlock {
	var err error
	var wb interfaces.ReadOnlySignedBeaconBlock
	switch bb := b.Block.(type) {
	case *v1alpha1.BeaconBlockContainer_Phase0Block:
		wb, err = blocks.NewSignedBeaconBlock(bb.Phase0Block)
	case *v1alpha1.BeaconBlockContainer_AltairBlock:
		wb, err = blocks.NewSignedBeaconBlock(bb.AltairBlock)
	case *v1alpha1.BeaconBlockContainer_BellatrixBlock:
		wb, err = blocks.NewSignedBeaconBlock(bb.BellatrixBlock)
	case *v1alpha1.BeaconBlockContainer_CapellaBlock:
		wb, err = blocks.NewSignedBeaconBlock(bb.CapellaBlock)
	case *v1alpha1.BeaconBlockContainer_BlindedCapellaBlock:
		wb, err = blocks.NewSignedBeaconBlock(bb.BlindedCapellaBlock)
	}
	if err != nil {
		panic("no block")
	}
	return wb.Block()
}
