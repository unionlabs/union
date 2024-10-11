package ibctesting

import (
	"fmt"
	"testing"
	"time"

	channeltypes "github.com/cosmos/ibc-go/v8/modules/core/04-channel/types"
	host "github.com/cosmos/ibc-go/v8/modules/core/24-host"
	ibctesting "github.com/cosmos/ibc-go/v8/testing"
	"github.com/stretchr/testify/require"

	wasmkeeper "github.com/CosmWasm/wasmd/x/wasm/keeper"
)

var (
	TimeIncrement   = time.Second * 5
	globalStartTime = time.Date(2020, 1, 2, 0, 0, 0, 0, time.UTC)
)

// Coordinator is a testing struct which contains N TestChain's. It handles keeping all chains
// in sync with regards to time.
type Coordinator struct {
	t *testing.T

	CurrentTime time.Time
	Chains      map[string]*TestChain
}

// NewCoordinator initializes Coordinator with n default wasm TestChain instances
func NewCoordinator(t *testing.T, n int, opts ...[]wasmkeeper.Option) *Coordinator {
	t.Helper()
	return NewCoordinatorX(t, n, DefaultWasmAppFactory, opts...)
}

// NewCoordinatorX initializes Coordinator with N TestChain instances using the given app factory
func NewCoordinatorX(t *testing.T, n int, appFactory ChainAppFactory, opts ...[]wasmkeeper.Option) *Coordinator {
	t.Helper()
	chains := make(map[string]*TestChain)
	coord := &Coordinator{
		t:           t,
		CurrentTime: globalStartTime,
	}

	for i := 1; i <= n; i++ {
		chainID := GetChainID(i)
		var x []wasmkeeper.Option
		if len(opts) > (i - 1) {
			x = opts[i-1]
		}
		chains[chainID] = NewTestChain(t, coord, appFactory, chainID, x...)
	}
	coord.Chains = chains

	return coord
}

// IncrementTime iterates through all the TestChain's and increments their current header time
// by 5 seconds.
//
// CONTRACT: this function must be called after every Commit on any TestChain.
func (coord *Coordinator) IncrementTime() {
	coord.IncrementTimeBy(TimeIncrement)
}

// IncrementTimeBy iterates through all the TestChain's and increments their current header time
// by specified time.
func (coord *Coordinator) IncrementTimeBy(increment time.Duration) {
	coord.CurrentTime = coord.CurrentTime.Add(increment).UTC()
	coord.UpdateTime()
}

// UpdateTime updates all clocks for the TestChains to the current global time.
func (coord *Coordinator) UpdateTime() {
	for _, chain := range coord.Chains {
		coord.UpdateTimeForChain(chain)
	}
}

// UpdateTimeForChain updates the clock for a specific chain.
func (coord *Coordinator) UpdateTimeForChain(chain *TestChain) {
	chain.CurrentHeader.Time = coord.CurrentTime.UTC()
}

// Setup constructs a TM client, connection, and channel on both chains provided. It will
// fail if any error occurs. The clientID's, TestConnections, and TestChannels are returned
// for both chains. The channels created are connected to the ibc-transfer application.
func (coord *Coordinator) Setup(path *Path) {
	coord.SetupConnections(path)

	// channels can also be referenced through the returned connections
	coord.CreateChannels(path)
}

// SetupClients is a helper function to create clients on both chains. It assumes the
// caller does not anticipate any errors.
func (coord *Coordinator) SetupClients(path *Path) {
	err := path.EndpointA.CreateClient()
	require.NoError(coord.t, err)

	err = path.EndpointB.CreateClient()
	require.NoError(coord.t, err)
}

// SetupClientConnections is a helper function to create clients and the appropriate
// connections on both the source and counterparty chain. It assumes the caller does not
// anticipate any errors.
func (coord *Coordinator) SetupConnections(path *Path) {
	coord.SetupClients(path)

	coord.CreateConnections(path)
}

// CreateConnection constructs and executes connection handshake messages in order to create
// OPEN channels on chainA and chainB. The connection information of for chainA and chainB
// are returned within a TestConnection struct. The function expects the connections to be
// successfully opened otherwise testing will fail.
func (coord *Coordinator) CreateConnections(path *Path) {
	err := path.EndpointA.ConnOpenInit()
	require.NoError(coord.t, err)

	err = path.EndpointB.ConnOpenTry()
	require.NoError(coord.t, err)

	err = path.EndpointA.ConnOpenAck()
	require.NoError(coord.t, err)

	err = path.EndpointB.ConnOpenConfirm()
	require.NoError(coord.t, err)

	// ensure counterparty is up to date
	err = path.EndpointA.UpdateClient()
	require.NoError(coord.t, err)
}

// CreateMockChannels constructs and executes channel handshake messages to create OPEN
// channels that use a mock application module that returns nil on all callbacks. This
// function is expects the channels to be successfully opened otherwise testing will
// fail.
func (coord *Coordinator) CreateMockChannels(path *Path) {
	path.EndpointA.ChannelConfig.PortID = ibctesting.MockPort
	path.EndpointB.ChannelConfig.PortID = ibctesting.MockPort

	coord.CreateChannels(path)
}

// CreateTransferChannels constructs and executes channel handshake messages to create OPEN
// ibc-transfer channels on chainA and chainB. The function expects the channels to be
// successfully opened otherwise testing will fail.
func (coord *Coordinator) CreateTransferChannels(path *Path) {
	path.EndpointA.ChannelConfig.PortID = ibctesting.TransferPort
	path.EndpointB.ChannelConfig.PortID = ibctesting.TransferPort

	coord.CreateChannels(path)
}

// CreateChannel constructs and executes channel handshake messages in order to create
// OPEN channels on chainA and chainB. The function expects the channels to be successfully
// opened otherwise testing will fail.
func (coord *Coordinator) CreateChannels(path *Path) {
	err := path.EndpointA.ChanOpenInit()
	require.NoError(coord.t, err)

	err = path.EndpointB.ChanOpenTry()
	require.NoError(coord.t, err)

	err = path.EndpointA.ChanOpenAck()
	require.NoError(coord.t, err)

	err = path.EndpointB.ChanOpenConfirm()
	require.NoError(coord.t, err)

	// ensure counterparty is up to date
	err = path.EndpointA.UpdateClient()
	require.NoError(coord.t, err)
}

// GetChain returns the TestChain using the given chainID and returns an error if it does
// not exist.
func (coord *Coordinator) GetChain(chainID string) *TestChain {
	chain, found := coord.Chains[chainID]
	require.True(coord.t, found, fmt.Sprintf("%s chain does not exist", chainID))
	return chain
}

// GetChainID returns the chainID used for the provided index.
func GetChainID(index int) string {
	return ibctesting.GetChainID(index)
}

// CommitBlock commits a block on the provided indexes and then increments the global time.
//
// CONTRACT: the passed in list of indexes must not contain duplicates
func (coord *Coordinator) CommitBlock(chains ...*TestChain) {
	for _, chain := range chains {
		chain.NextBlock()
	}
	coord.IncrementTime()
}

// CommitNBlocks commits n blocks to state and updates the block height by 1 for each commit.
func (coord *Coordinator) CommitNBlocks(chain *TestChain, n uint64) {
	for i := uint64(0); i < n; i++ {
		chain.NextBlock()
		coord.IncrementTime()
	}
}

// ConnOpenInitOnBothChains initializes a connection on both endpoints with the state INIT
// using the OpenInit handshake call.
func (coord *Coordinator) ConnOpenInitOnBothChains(path *Path) error {
	if err := path.EndpointA.ConnOpenInit(); err != nil {
		return err
	}

	if err := path.EndpointB.ConnOpenInit(); err != nil {
		return err
	}

	if err := path.EndpointA.UpdateClient(); err != nil {
		return err
	}

	err := path.EndpointB.UpdateClient()

	return err
}

// ChanOpenInitOnBothChains initializes a channel on the source chain and counterparty chain
// with the state INIT using the OpenInit handshake call.
func (coord *Coordinator) ChanOpenInitOnBothChains(path *Path) error {
	// NOTE: only creation of a capability for a transfer or mock port is supported
	// Other applications must bind to the port in InitGenesis or modify this code.

	if err := path.EndpointA.ChanOpenInit(); err != nil {
		return err
	}

	if err := path.EndpointB.ChanOpenInit(); err != nil {
		return err
	}

	if err := path.EndpointA.UpdateClient(); err != nil {
		return err
	}

	err := path.EndpointB.UpdateClient()

	return err
}

// RelayAndAckPendingPackets sends pending packages from path.EndpointA to the counterparty chain and acks
func (coord *Coordinator) RelayAndAckPendingPackets(path *Path) error {
	// get all the packet to relay src->dest
	src := path.EndpointA
	require.NoError(coord.t, src.UpdateClient())
	coord.t.Logf("Relay: %d Packets A->B, %d Packets B->A\n", len(src.Chain.PendingSendPackets), len(path.EndpointB.Chain.PendingSendPackets))
	for _, v := range src.Chain.PendingSendPackets {
		err := path.RelayPacket(v, nil)
		if err != nil {
			return err
		}
		src.Chain.PendingSendPackets = src.Chain.PendingSendPackets[1:]
	}

	src = path.EndpointB
	require.NoError(coord.t, src.UpdateClient())
	for _, v := range src.Chain.PendingSendPackets {
		err := path.RelayPacket(v, nil)
		if err != nil {
			return err
		}
		src.Chain.PendingSendPackets = src.Chain.PendingSendPackets[1:]
	}
	return nil
}

// TimeoutPendingPackets returns the package to source chain to let the IBC app revert any operation.
// from A to B
func (coord *Coordinator) TimeoutPendingPackets(path *Path) error {
	src := path.EndpointA
	dest := path.EndpointB

	toSend := src.Chain.PendingSendPackets
	coord.t.Logf("Timeout %d Packets A->B\n", len(toSend))
	require.NoError(coord.t, src.UpdateClient())

	// Increment time and commit block so that 5 second delay period passes between send and receive
	coord.IncrementTime()
	coord.CommitBlock(src.Chain, dest.Chain)
	for _, packet := range toSend {
		// get proof of packet unreceived on dest
		packetKey := host.PacketReceiptKey(packet.GetDestPort(), packet.GetDestChannel(), packet.GetSequence())
		proofUnreceived, proofHeight := dest.QueryProof(packetKey)
		timeoutMsg := channeltypes.NewMsgTimeout(packet, packet.Sequence, proofUnreceived, proofHeight, src.Chain.SenderAccount.GetAddress().String())
		err := src.Chain.sendMsgs(timeoutMsg)
		if err != nil {
			return err
		}
	}
	src.Chain.PendingSendPackets = nil
	return nil
}

// CloseChannel close channel on both sides
func (coord *Coordinator) CloseChannel(path *Path) {
	err := path.EndpointA.ChanCloseInit()
	require.NoError(coord.t, err)
	coord.IncrementTime()
	err = path.EndpointB.UpdateClient()
	require.NoError(coord.t, err)
	err = path.EndpointB.ChanCloseConfirm()
	require.NoError(coord.t, err)
}
