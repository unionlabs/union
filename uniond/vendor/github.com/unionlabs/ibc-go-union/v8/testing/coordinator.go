package ibctesting

import (
	"fmt"
	"strconv"
	"testing"
	"time"

	"github.com/stretchr/testify/require"
)

var (
	ChainIDPrefix = "testchain"
	// to disable revision format, set ChainIDSuffix to ""
	ChainIDSuffix   = "-1"
	globalStartTime = time.Date(2020, 1, 2, 0, 0, 0, 0, time.UTC)
	TimeIncrement   = time.Second * 5
)

// Coordinator is a testing struct which contains N TestChain's. It handles keeping all chains
// in sync with regards to time.
type Coordinator struct {
	*testing.T

	CurrentTime time.Time
	Chains      map[string]*TestChain
}

// NewCoordinator initializes Coordinator with N TestChain's
func NewCoordinator(t *testing.T, n int) *Coordinator {
	t.Helper()
	chains := make(map[string]*TestChain)
	coord := &Coordinator{
		T:           t,
		CurrentTime: globalStartTime,
	}

	for i := 1; i <= n; i++ {
		chainID := GetChainID(i)
		chains[chainID] = NewTestChain(t, coord, chainID)
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
	require.NoError(coord.T, err)

	err = path.EndpointB.CreateClient()
	require.NoError(coord.T, err)
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
	require.NoError(coord.T, err)

	err = path.EndpointB.ConnOpenTry()
	require.NoError(coord.T, err)

	err = path.EndpointA.ConnOpenAck()
	require.NoError(coord.T, err)

	err = path.EndpointB.ConnOpenConfirm()
	require.NoError(coord.T, err)

	// ensure counterparty is up to date
	err = path.EndpointA.UpdateClient()
	require.NoError(coord.T, err)
}

// CreateMockChannels constructs and executes channel handshake messages to create OPEN
// channels that use a mock application module that returns nil on all callbacks. This
// function is expects the channels to be successfully opened otherwise testing will
// fail.
func (coord *Coordinator) CreateMockChannels(path *Path) {
	path.EndpointA.ChannelConfig.PortID = MockPort
	path.EndpointB.ChannelConfig.PortID = MockPort

	coord.CreateChannels(path)
}

// CreateTransferChannels constructs and executes channel handshake messages to create OPEN
// ibc-transfer channels on chainA and chainB. The function expects the channels to be
// successfully opened otherwise testing will fail.
func (coord *Coordinator) CreateTransferChannels(path *Path) {
	path.EndpointA.ChannelConfig.PortID = TransferPort
	path.EndpointB.ChannelConfig.PortID = TransferPort

	coord.CreateChannels(path)
}

// CreateChannel constructs and executes channel handshake messages in order to create
// OPEN channels on chainA and chainB. The function expects the channels to be successfully
// opened otherwise testing will fail.
func (coord *Coordinator) CreateChannels(path *Path) {
	err := path.EndpointA.ChanOpenInit()
	require.NoError(coord.T, err)

	err = path.EndpointB.ChanOpenTry()
	require.NoError(coord.T, err)

	err = path.EndpointA.ChanOpenAck()
	require.NoError(coord.T, err)

	err = path.EndpointB.ChanOpenConfirm()
	require.NoError(coord.T, err)

	// ensure counterparty is up to date
	err = path.EndpointA.UpdateClient()
	require.NoError(coord.T, err)
}

// GetChain returns the TestChain using the given chainID and returns an error if it does
// not exist.
func (coord *Coordinator) GetChain(chainID string) *TestChain {
	chain, found := coord.Chains[chainID]
	require.True(coord.T, found, fmt.Sprintf("%s chain does not exist", chainID))
	return chain
}

// GetChainID returns the chainID used for the provided index.
func GetChainID(index int) string {
	return ChainIDPrefix + strconv.Itoa(index) + ChainIDSuffix
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
