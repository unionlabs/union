// Package node defines a gRPC node service implementation, providing
// useful endpoints for checking a node's sync status, peer info,
// genesis data, and version information.
package node

import (
	"context"
	"fmt"
	"sort"
	"time"

	"github.com/golang/protobuf/ptypes/empty"
	"github.com/golang/protobuf/ptypes/timestamp"
	"github.com/libp2p/go-libp2p/core/network"
	"github.com/libp2p/go-libp2p/core/peer"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/blockchain"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/db"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/execution"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/sync"
	"github.com/prysmaticlabs/prysm/v4/io/logs"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/runtime/version"
	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
	"google.golang.org/protobuf/types/known/timestamppb"
)

// Server defines a server implementation of the gRPC Node service,
// providing RPC endpoints for verifying a beacon node's sync status, genesis and
// version information, and services the node implements and runs.
type Server struct {
	LogsStreamer         logs.Streamer
	StreamLogsBufferSize int
	SyncChecker          sync.Checker
	Server               *grpc.Server
	BeaconDB             db.ReadOnlyDatabase
	PeersFetcher         p2p.PeersProvider
	PeerManager          p2p.PeerManager
	GenesisTimeFetcher   blockchain.TimeFetcher
	GenesisFetcher       blockchain.GenesisFetcher
	POWChainInfoFetcher  execution.ChainInfoFetcher
	BeaconMonitoringHost string
	BeaconMonitoringPort int
}

// GetSyncStatus checks the current network sync status of the node.
func (ns *Server) GetSyncStatus(_ context.Context, _ *empty.Empty) (*ethpb.SyncStatus, error) {
	return &ethpb.SyncStatus{
		Syncing: ns.SyncChecker.Syncing(),
	}, nil
}

// GetGenesis fetches genesis chain information of Ethereum. Returns unix timestamp 0
// if a genesis time has yet to be determined.
func (ns *Server) GetGenesis(ctx context.Context, _ *empty.Empty) (*ethpb.Genesis, error) {
	contractAddr, err := ns.BeaconDB.DepositContractAddress(ctx)
	if err != nil {
		return nil, status.Errorf(codes.Internal, "Could not retrieve contract address from db: %v", err)
	}
	genesisTime := ns.GenesisTimeFetcher.GenesisTime()
	var defaultGenesisTime time.Time
	var gt *timestamp.Timestamp
	if genesisTime == defaultGenesisTime {
		gt = timestamppb.New(time.Unix(0, 0))
	} else {
		gt = timestamppb.New(genesisTime)
	}

	genValRoot := ns.GenesisFetcher.GenesisValidatorsRoot()
	return &ethpb.Genesis{
		GenesisTime:            gt,
		DepositContractAddress: contractAddr,
		GenesisValidatorsRoot:  genValRoot[:],
	}, nil
}

// GetVersion checks the version information of the beacon node.
func (_ *Server) GetVersion(_ context.Context, _ *empty.Empty) (*ethpb.Version, error) {
	return &ethpb.Version{
		Version: version.Version(),
	}, nil
}

// ListImplementedServices lists the services implemented and enabled by this node.
//
// Any service not present in this list may return UNIMPLEMENTED or
// PERMISSION_DENIED. The server may also support fetching services by grpc
// reflection.
func (ns *Server) ListImplementedServices(_ context.Context, _ *empty.Empty) (*ethpb.ImplementedServices, error) {
	serviceInfo := ns.Server.GetServiceInfo()
	serviceNames := make([]string, 0, len(serviceInfo))
	for svc := range serviceInfo {
		serviceNames = append(serviceNames, svc)
	}
	sort.Strings(serviceNames)
	return &ethpb.ImplementedServices{
		Services: serviceNames,
	}, nil
}

// GetHost returns the p2p data on the current local and host peer.
func (ns *Server) GetHost(_ context.Context, _ *empty.Empty) (*ethpb.HostData, error) {
	var stringAddr []string
	for _, addr := range ns.PeerManager.Host().Addrs() {
		stringAddr = append(stringAddr, addr.String())
	}
	record := ns.PeerManager.ENR()
	enr := ""
	var err error
	if record != nil {
		enr, err = p2p.SerializeENR(record)
		if err != nil {
			return nil, status.Errorf(codes.Internal, "Unable to serialize enr: %v", err)
		}
	}

	return &ethpb.HostData{
		Addresses: stringAddr,
		PeerId:    ns.PeerManager.PeerID().String(),
		Enr:       enr,
	}, nil
}

// GetPeer returns the data known about the peer defined by the provided peer id.
func (ns *Server) GetPeer(_ context.Context, peerReq *ethpb.PeerRequest) (*ethpb.Peer, error) {
	pid, err := peer.Decode(peerReq.PeerId)
	if err != nil {
		return nil, status.Errorf(codes.InvalidArgument, "Unable to parse provided peer id: %v", err)
	}
	addr, err := ns.PeersFetcher.Peers().Address(pid)
	if err != nil {
		return nil, status.Errorf(codes.NotFound, "Requested peer does not exist: %v", err)
	}
	dir, err := ns.PeersFetcher.Peers().Direction(pid)
	if err != nil {
		return nil, status.Errorf(codes.NotFound, "Requested peer does not exist: %v", err)
	}
	pbDirection := ethpb.PeerDirection_UNKNOWN
	switch dir {
	case network.DirInbound:
		pbDirection = ethpb.PeerDirection_INBOUND
	case network.DirOutbound:
		pbDirection = ethpb.PeerDirection_OUTBOUND
	}
	connState, err := ns.PeersFetcher.Peers().ConnectionState(pid)
	if err != nil {
		return nil, status.Errorf(codes.NotFound, "Requested peer does not exist: %v", err)
	}
	record, err := ns.PeersFetcher.Peers().ENR(pid)
	if err != nil {
		return nil, status.Errorf(codes.NotFound, "Requested peer does not exist: %v", err)
	}
	enr := ""
	if record != nil {
		enr, err = p2p.SerializeENR(record)
		if err != nil {
			return nil, status.Errorf(codes.Internal, "Unable to serialize enr: %v", err)
		}
	}
	return &ethpb.Peer{
		Address:         addr.String(),
		Direction:       pbDirection,
		ConnectionState: ethpb.ConnectionState(connState),
		PeerId:          peerReq.PeerId,
		Enr:             enr,
	}, nil
}

// ListPeers lists the peers connected to this node.
func (ns *Server) ListPeers(ctx context.Context, _ *empty.Empty) (*ethpb.Peers, error) {
	peers := ns.PeersFetcher.Peers().Connected()
	res := make([]*ethpb.Peer, 0, len(peers))
	for _, pid := range peers {
		if ctx.Err() != nil {
			return nil, ctx.Err()
		}
		multiaddr, err := ns.PeersFetcher.Peers().Address(pid)
		if err != nil {
			continue
		}
		direction, err := ns.PeersFetcher.Peers().Direction(pid)
		if err != nil {
			continue
		}
		record, err := ns.PeersFetcher.Peers().ENR(pid)
		if err != nil {
			continue
		}
		enr := ""
		if record != nil {
			enr, err = p2p.SerializeENR(record)
			if err != nil {
				continue
			}
		}
		multiAddrStr := "unknown"
		if multiaddr != nil {
			multiAddrStr = multiaddr.String()
		}
		address := fmt.Sprintf("%s/p2p/%s", multiAddrStr, pid.String())
		pbDirection := ethpb.PeerDirection_UNKNOWN
		switch direction {
		case network.DirInbound:
			pbDirection = ethpb.PeerDirection_INBOUND
		case network.DirOutbound:
			pbDirection = ethpb.PeerDirection_OUTBOUND
		}
		res = append(res, &ethpb.Peer{
			Address:         address,
			Direction:       pbDirection,
			ConnectionState: ethpb.ConnectionState_CONNECTED,
			PeerId:          pid.String(),
			Enr:             enr,
		})
	}

	return &ethpb.Peers{
		Peers: res,
	}, nil
}

// GetETH1ConnectionStatus gets data about the ETH1 endpoints.
func (ns *Server) GetETH1ConnectionStatus(_ context.Context, _ *empty.Empty) (*ethpb.ETH1ConnectionStatus, error) {
	var currErr string
	err := ns.POWChainInfoFetcher.ExecutionClientConnectionErr()
	if err != nil {
		currErr = err.Error()
	}
	return &ethpb.ETH1ConnectionStatus{
		CurrentAddress:         ns.POWChainInfoFetcher.ExecutionClientEndpoint(),
		CurrentConnectionError: currErr,
		Addresses:              []string{ns.POWChainInfoFetcher.ExecutionClientEndpoint()},
	}, nil
}

// StreamBeaconLogs from the beacon node via a gRPC server-side stream.
// DEPRECATED: This endpoint doesn't appear to be used and have been marked for deprecation.
func (ns *Server) StreamBeaconLogs(_ *empty.Empty, stream ethpb.Health_StreamBeaconLogsServer) error {
	ch := make(chan []byte, ns.StreamLogsBufferSize)
	sub := ns.LogsStreamer.LogsFeed().Subscribe(ch)
	defer func() {
		sub.Unsubscribe()
		close(ch)
	}()

	recentLogs := ns.LogsStreamer.GetLastFewLogs()
	logStrings := make([]string, len(recentLogs))
	for i, log := range recentLogs {
		logStrings[i] = string(log)
	}
	if err := stream.Send(&ethpb.LogsResponse{
		Logs: logStrings,
	}); err != nil {
		return status.Errorf(codes.Unavailable, "Could not send over stream: %v", err)
	}
	for {
		select {
		case log := <-ch:
			resp := &ethpb.LogsResponse{
				Logs: []string{string(log)},
			}
			if err := stream.Send(resp); err != nil {
				return status.Errorf(codes.Unavailable, "Could not send over stream: %v", err)
			}
		case err := <-sub.Err():
			return status.Errorf(codes.Canceled, "Subscriber error, closing: %v", err)
		case <-stream.Context().Done():
			return status.Error(codes.Canceled, "Context canceled")
		}
	}
}
