package validator_helpers

import (
	"time"

	"google.golang.org/grpc"
)

// Use an interface with a private dummy function to force all other packages to call NewNodeConnection
type NodeConnection interface {
	GetGrpcClientConn() *grpc.ClientConn
	GetBeaconApiUrl() string
	GetBeaconApiTimeout() time.Duration
	dummy()
}

type nodeConnection struct {
	grpcClientConn   *grpc.ClientConn
	beaconApiUrl     string
	beaconApiTimeout time.Duration
}

func (c *nodeConnection) GetGrpcClientConn() *grpc.ClientConn {
	return c.grpcClientConn
}

func (c *nodeConnection) GetBeaconApiUrl() string {
	return c.beaconApiUrl
}

func (c *nodeConnection) GetBeaconApiTimeout() time.Duration {
	return c.beaconApiTimeout
}

func (*nodeConnection) dummy() {}

func NewNodeConnection(grpcConn *grpc.ClientConn, beaconApiUrl string, beaconApiTimeout time.Duration) NodeConnection {
	conn := &nodeConnection{}
	conn.grpcClientConn = grpcConn
	conn.beaconApiUrl = beaconApiUrl
	conn.beaconApiTimeout = beaconApiTimeout
	return conn
}
