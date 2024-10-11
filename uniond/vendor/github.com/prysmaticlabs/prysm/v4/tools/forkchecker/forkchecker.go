/**
 * Fork choice checker
 *
 * A gRPC client that polls beacon node at every slot to log or compare nodes current head.
 *
 * Example: 2 beacon nodes with 2 gRPC end points, 127.0.0.1:4000 and 127.0.0.1:4001
 * For logging heads: forkchecker --endpoint 127.0.0.1:4000 --endpoint 127.0.0.1:4001
 * For comparing heads: forkchecker --endpoint 127.0.0.1:4000 --endpoint 127.0.0.1:4001 --compare
 */
package main

import (
	"context"
	"encoding/hex"
	"flag"
	"reflect"
	"time"

	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	pb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/sirupsen/logrus"
	"google.golang.org/grpc"
	"google.golang.org/protobuf/types/known/emptypb"
)

var log = logrus.WithField("prefix", "forkchoice_checker")

type endpoint []string

func (_ *endpoint) String() string {
	return "gRPC endpoints"
}

// Set adds endpoint value to list.
func (e *endpoint) Set(value string) error {
	*e = append(*e, value)
	return nil
}

func main() {
	var endpts endpoint
	clients := make(map[string]pb.BeaconChainClient)

	flag.Var(&endpts, "endpoint", "Specify gRPC end points for beacon node")
	compare := flag.Bool("compare", false, "Enable head comparisons between all end points")
	flag.Parse()

	for _, endpt := range endpts {
		conn, err := grpc.Dial(endpt, grpc.WithInsecure())
		if err != nil {
			log.WithError(err).Fatal("fail to dial")
		}
		clients[endpt] = pb.NewBeaconChainClient(conn)
	}

	ticker := time.NewTicker(time.Duration(params.BeaconConfig().SecondsPerSlot) * time.Second)
	go func() {
		for range ticker.C {
			if *compare {
				compareHeads(clients)
			} else {
				displayHeads(clients)
			}
		}
	}()
	select {}
}

// log heads for all RPC end points
func displayHeads(clients map[string]pb.BeaconChainClient) {
	for endpt, client := range clients {
		head, err := client.GetChainHead(context.Background(), &emptypb.Empty{})
		if err != nil {
			log.Fatal(err)
		}
		logHead(endpt, head)
	}
}

// compare heads between all RPC end points, log the mismatch if there's one.
func compareHeads(clients map[string]pb.BeaconChainClient) {
	endpt1 := randomEndpt(clients)
	head1, err := clients[endpt1].GetChainHead(context.Background(), &emptypb.Empty{})
	if err != nil {
		log.Fatal(err)
	}

	log.Infof("Comparing all heads for head slot :%d", head1.HeadSlot)
	if (head1.HeadSlot+1)%params.BeaconConfig().SlotsPerEpoch == 0 {
		p, err := clients[endpt1].GetValidatorParticipation(context.Background(), &pb.GetValidatorParticipationRequest{})
		if err != nil {
			log.Fatal(err)
		}
		logParticipation(endpt1, p.Participation)
	}

	for endpt2, client := range clients {
		head2, err := client.GetChainHead(context.Background(), &emptypb.Empty{})
		if err != nil {
			log.Fatal(err)
		}
		if !reflect.DeepEqual(head1, head2) {
			log.Error("Uh oh! Heads mismatched!")
			logHead(endpt1, head1)
			logHead(endpt2, head2)

			if (head1.HeadSlot+1)%params.BeaconConfig().SlotsPerEpoch == 0 {
				p, err := clients[endpt2].GetValidatorParticipation(context.Background(), &pb.GetValidatorParticipationRequest{
					QueryFilter: &pb.GetValidatorParticipationRequest_Epoch{
						Epoch: primitives.Epoch(head2.HeadSlot / params.BeaconConfig().SlotsPerEpoch),
					},
				})
				if err != nil {
					log.Fatal(err)
				}
				logParticipation(endpt2, p.Participation)
			}
		}
	}
}

func logHead(endpt string, head *pb.ChainHead) {
	log.WithFields(
		logrus.Fields{
			"HeadSlot":       head.HeadSlot,
			"HeadRoot":       hex.EncodeToString(head.HeadBlockRoot),
			"JustifiedEpoch": head.JustifiedEpoch,
			"JustifiedRoot":  hex.EncodeToString(head.JustifiedBlockRoot),
			"FinalizedEpoch": head.FinalizedEpoch,
			"FinalizedRoot":  hex.EncodeToString(head.FinalizedBlockRoot),
		}).Info("Head from beacon node ", endpt)
}

func logParticipation(endpt string, p *pb.ValidatorParticipation) {
	log.WithFields(
		logrus.Fields{
			"VotedEther":        p.VotedEther,
			"TotalEther":        p.EligibleEther,
			"ParticipationRate": p.GlobalParticipationRate,
		}).Info("Participation rate from beacon node ", endpt)
}

func randomEndpt(clients map[string]pb.BeaconChainClient) string {
	for endpt := range clients {
		return endpt
	}
	return ""
}
