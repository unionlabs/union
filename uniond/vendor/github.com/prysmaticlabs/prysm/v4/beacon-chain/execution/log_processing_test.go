package execution

import (
	"context"
	"encoding/binary"
	"math/big"
	"testing"
	"time"

	"github.com/ethereum/go-ethereum"
	"github.com/ethereum/go-ethereum/common"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/cache/depositcache"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/feed"
	statefeed "github.com/prysmaticlabs/prysm/v4/beacon-chain/core/feed/state"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/db"
	testDB "github.com/prysmaticlabs/prysm/v4/beacon-chain/db/testing"
	mockExecution "github.com/prysmaticlabs/prysm/v4/beacon-chain/execution/testing"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	contracts "github.com/prysmaticlabs/prysm/v4/contracts/deposit"
	"github.com/prysmaticlabs/prysm/v4/contracts/deposit/mock"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
	logTest "github.com/sirupsen/logrus/hooks/test"
)

func TestProcessDepositLog_OK(t *testing.T) {
	hook := logTest.NewGlobal()

	testAcc, err := mock.Setup()
	require.NoError(t, err, "Unable to set up simulated backend")

	beaconDB := testDB.SetupDB(t)
	depositCache, err := depositcache.New()
	require.NoError(t, err)

	server, endpoint, err := mockExecution.SetupRPCServer()
	require.NoError(t, err)
	t.Cleanup(func() {
		server.Stop()
	})
	web3Service, err := NewService(context.Background(),
		WithHttpEndpoint(endpoint),
		WithDepositContractAddress(testAcc.ContractAddr),
		WithDatabase(beaconDB),
		WithDepositCache(depositCache),
	)
	require.NoError(t, err, "unable to setup web3 ETH1.0 chain service")
	web3Service = setDefaultMocks(web3Service)
	web3Service.depositContractCaller, err = contracts.NewDepositContractCaller(testAcc.ContractAddr, testAcc.Backend)
	require.NoError(t, err)

	testAcc.Backend.Commit()

	deposits, _, err := util.DeterministicDepositsAndKeys(1)
	require.NoError(t, err)

	_, depositRoots, err := util.DeterministicDepositTrie(len(deposits))
	require.NoError(t, err)
	data := deposits[0].Data

	testAcc.TxOpts.Value = mock.Amount32Eth()
	testAcc.TxOpts.GasLimit = 1000000
	_, err = testAcc.Contract.Deposit(testAcc.TxOpts, data.PublicKey, data.WithdrawalCredentials, data.Signature, depositRoots[0])
	require.NoError(t, err, "Could not deposit to deposit contract")

	testAcc.Backend.Commit()

	query := ethereum.FilterQuery{
		Addresses: []common.Address{
			web3Service.cfg.depositContractAddr,
		},
	}

	logs, err := testAcc.Backend.FilterLogs(web3Service.ctx, query)
	require.NoError(t, err, "Unable to retrieve logs")

	if len(logs) == 0 {
		t.Fatal("no logs")
	}

	err = web3Service.ProcessLog(context.Background(), &logs[0])
	require.NoError(t, err)

	require.LogsDoNotContain(t, hook, "Could not unpack log")
	require.LogsDoNotContain(t, hook, "Could not save in trie")
	require.LogsDoNotContain(t, hook, "could not deserialize validator public key")
	require.LogsDoNotContain(t, hook, "could not convert bytes to signature")
	require.LogsDoNotContain(t, hook, "could not sign root for deposit data")
	require.LogsDoNotContain(t, hook, "deposit signature did not verify")
	require.LogsDoNotContain(t, hook, "could not tree hash deposit data")
	require.LogsDoNotContain(t, hook, "deposit merkle branch of deposit root did not verify for root")
	require.LogsContain(t, hook, "Deposit registered from deposit contract")

	hook.Reset()
}

func TestProcessDepositLog_InsertsPendingDeposit(t *testing.T) {
	hook := logTest.NewGlobal()
	testAcc, err := mock.Setup()
	require.NoError(t, err, "Unable to set up simulated backend")
	beaconDB := testDB.SetupDB(t)
	depositCache, err := depositcache.New()
	require.NoError(t, err)
	server, endpoint, err := mockExecution.SetupRPCServer()
	require.NoError(t, err)
	t.Cleanup(func() {
		server.Stop()
	})

	web3Service, err := NewService(context.Background(),
		WithHttpEndpoint(endpoint),
		WithDepositContractAddress(testAcc.ContractAddr),
		WithDatabase(beaconDB),
		WithDepositCache(depositCache),
	)
	require.NoError(t, err, "unable to setup web3 ETH1.0 chain service")
	web3Service = setDefaultMocks(web3Service)
	web3Service.depositContractCaller, err = contracts.NewDepositContractCaller(testAcc.ContractAddr, testAcc.Backend)
	require.NoError(t, err)

	testAcc.Backend.Commit()

	deposits, _, err := util.DeterministicDepositsAndKeys(1)
	require.NoError(t, err)
	_, depositRoots, err := util.DeterministicDepositTrie(len(deposits))
	require.NoError(t, err)
	data := deposits[0].Data

	testAcc.TxOpts.Value = mock.Amount32Eth()
	testAcc.TxOpts.GasLimit = 1000000

	_, err = testAcc.Contract.Deposit(testAcc.TxOpts, data.PublicKey, data.WithdrawalCredentials, data.Signature, depositRoots[0])
	require.NoError(t, err, "Could not deposit to deposit contract")

	_, err = testAcc.Contract.Deposit(testAcc.TxOpts, data.PublicKey, data.WithdrawalCredentials, data.Signature, depositRoots[0])
	require.NoError(t, err, "Could not deposit to deposit contract")

	testAcc.Backend.Commit()

	query := ethereum.FilterQuery{
		Addresses: []common.Address{
			web3Service.cfg.depositContractAddr,
		},
	}

	logs, err := testAcc.Backend.FilterLogs(web3Service.ctx, query)
	require.NoError(t, err, "Unable to retrieve logs")

	web3Service.chainStartData.Chainstarted = true

	err = web3Service.ProcessDepositLog(context.Background(), &logs[0])
	require.NoError(t, err)
	err = web3Service.ProcessDepositLog(context.Background(), &logs[1])
	require.NoError(t, err)

	pendingDeposits := web3Service.cfg.depositCache.PendingDeposits(context.Background(), nil /*blockNum*/)
	require.Equal(t, 2, len(pendingDeposits), "Unexpected number of deposits")

	hook.Reset()
}

func TestUnpackDepositLogData_OK(t *testing.T) {
	testAcc, err := mock.Setup()
	require.NoError(t, err, "Unable to set up simulated backend")
	beaconDB := testDB.SetupDB(t)
	server, endpoint, err := mockExecution.SetupRPCServer()
	require.NoError(t, err)
	t.Cleanup(func() {
		server.Stop()
	})
	web3Service, err := NewService(context.Background(),
		WithHttpEndpoint(endpoint),
		WithDepositContractAddress(testAcc.ContractAddr),
		WithDatabase(beaconDB),
	)
	require.NoError(t, err, "unable to setup web3 ETH1.0 chain service")
	web3Service = setDefaultMocks(web3Service)
	web3Service.depositContractCaller, err = contracts.NewDepositContractCaller(testAcc.ContractAddr, testAcc.Backend)
	require.NoError(t, err)

	testAcc.Backend.Commit()

	deposits, _, err := util.DeterministicDepositsAndKeys(1)
	require.NoError(t, err)
	_, depositRoots, err := util.DeterministicDepositTrie(len(deposits))
	require.NoError(t, err)
	data := deposits[0].Data

	testAcc.TxOpts.Value = mock.Amount32Eth()
	testAcc.TxOpts.GasLimit = 1000000
	_, err = testAcc.Contract.Deposit(testAcc.TxOpts, data.PublicKey, data.WithdrawalCredentials, data.Signature, depositRoots[0])
	require.NoError(t, err, "Could not deposit to deposit contract")
	testAcc.Backend.Commit()

	query := ethereum.FilterQuery{
		Addresses: []common.Address{
			web3Service.cfg.depositContractAddr,
		},
	}

	logz, err := testAcc.Backend.FilterLogs(web3Service.ctx, query)
	require.NoError(t, err, "Unable to retrieve logs")

	loggedPubkey, withCreds, _, loggedSig, index, err := contracts.UnpackDepositLogData(logz[0].Data)
	require.NoError(t, err, "Unable to unpack logs")

	require.Equal(t, uint64(0), binary.LittleEndian.Uint64(index), "Retrieved merkle tree index is incorrect")
	require.DeepEqual(t, data.PublicKey, loggedPubkey, "Pubkey is not the same as the data that was put in")
	require.DeepEqual(t, data.Signature, loggedSig, "Proof of Possession is not the same as the data that was put in")
	require.DeepEqual(t, data.WithdrawalCredentials, withCreds, "Withdrawal Credentials is not the same as the data that was put in")
}

func TestProcessETH2GenesisLog_8DuplicatePubkeys(t *testing.T) {
	hook := logTest.NewGlobal()
	testAcc, err := mock.Setup()
	require.NoError(t, err, "Unable to set up simulated backend")
	beaconDB := testDB.SetupDB(t)
	depositCache, err := depositcache.New()
	require.NoError(t, err)
	server, endpoint, err := mockExecution.SetupRPCServer()
	require.NoError(t, err)
	t.Cleanup(func() {
		server.Stop()
	})

	web3Service, err := NewService(context.Background(),
		WithHttpEndpoint(endpoint),
		WithDepositContractAddress(testAcc.ContractAddr),
		WithDatabase(beaconDB),
		WithDepositCache(depositCache),
	)
	require.NoError(t, err, "unable to setup web3 ETH1.0 chain service")
	web3Service = setDefaultMocks(web3Service)
	web3Service.depositContractCaller, err = contracts.NewDepositContractCaller(testAcc.ContractAddr, testAcc.Backend)
	require.NoError(t, err)

	params.SetupTestConfigCleanup(t)
	bConfig := params.MinimalSpecConfig().Copy()
	bConfig.MinGenesisTime = 0
	params.OverrideBeaconConfig(bConfig)

	testAcc.Backend.Commit()
	require.NoError(t, testAcc.Backend.AdjustTime(time.Duration(int64(time.Now().Nanosecond()))))

	deposits, _, err := util.DeterministicDepositsAndKeys(1)
	require.NoError(t, err)
	_, depositRoots, err := util.DeterministicDepositTrie(len(deposits))
	require.NoError(t, err)
	data := deposits[0].Data

	testAcc.TxOpts.Value = mock.Amount32Eth()
	testAcc.TxOpts.GasLimit = 1000000

	// 64 Validators are used as size required for beacon-chain to start. This number
	// is defined in the deposit contract as the number required for the testnet. The actual number
	// is 2**14
	for i := 0; i < depositsReqForChainStart; i++ {
		testAcc.TxOpts.Value = mock.Amount32Eth()
		_, err = testAcc.Contract.Deposit(testAcc.TxOpts, data.PublicKey, data.WithdrawalCredentials, data.Signature, depositRoots[0])
		require.NoError(t, err, "Could not deposit to deposit contract")

		testAcc.Backend.Commit()
	}

	query := ethereum.FilterQuery{
		Addresses: []common.Address{
			web3Service.cfg.depositContractAddr,
		},
	}

	logs, err := testAcc.Backend.FilterLogs(web3Service.ctx, query)
	require.NoError(t, err, "Unable to retrieve logs")

	for i := range logs {
		err = web3Service.ProcessLog(context.Background(), &logs[i])
		require.NoError(t, err)
	}
	assert.Equal(t, false, web3Service.chainStartData.Chainstarted, "Genesis has been triggered despite being 8 duplicate keys")

	require.LogsDoNotContain(t, hook, "Minimum number of validators reached for beacon-chain to start")
	hook.Reset()
}

func TestProcessETH2GenesisLog(t *testing.T) {
	params.SetupTestConfigCleanup(t)
	cfg := params.BeaconConfig().Copy()
	cfg.GenesisDelay = 0
	params.OverrideBeaconConfig(cfg)
	hook := logTest.NewGlobal()
	testAcc, err := mock.Setup()
	require.NoError(t, err, "Unable to set up simulated backend")
	beaconDB := testDB.SetupDB(t)
	depositCache, err := depositcache.New()
	require.NoError(t, err)

	server, endpoint, err := mockExecution.SetupRPCServer()
	require.NoError(t, err)
	t.Cleanup(func() {
		server.Stop()
	})
	web3Service, err := NewService(context.Background(),
		WithHttpEndpoint(endpoint),
		WithDepositContractAddress(testAcc.ContractAddr),
		WithDatabase(beaconDB),
		WithDepositCache(depositCache),
	)
	require.NoError(t, err, "unable to setup web3 ETH1.0 chain service")
	web3Service = setDefaultMocks(web3Service)
	web3Service.depositContractCaller, err = contracts.NewDepositContractCaller(testAcc.ContractAddr, testAcc.Backend)
	web3Service.rpcClient = &mockExecution.RPCClient{Backend: testAcc.Backend}
	require.NoError(t, err)
	params.SetupTestConfigCleanup(t)
	bConfig := params.MinimalSpecConfig().Copy()
	bConfig.MinGenesisTime = 0
	params.OverrideBeaconConfig(bConfig)

	testAcc.Backend.Commit()
	require.NoError(t, testAcc.Backend.AdjustTime(time.Duration(int64(time.Now().Nanosecond()))))

	deposits, _, err := util.DeterministicDepositsAndKeys(uint64(depositsReqForChainStart))
	require.NoError(t, err)
	_, roots, err := util.DeterministicDepositTrie(len(deposits))
	require.NoError(t, err)

	// 64 Validators are used as size required for beacon-chain to start. This number
	// is defined in the deposit contract as the number required for the testnet. The actual number
	// is 2**14
	for i := 0; i < depositsReqForChainStart; i++ {
		data := deposits[i].Data
		testAcc.TxOpts.Value = mock.Amount32Eth()
		testAcc.TxOpts.GasLimit = 1000000
		_, err = testAcc.Contract.Deposit(testAcc.TxOpts, data.PublicKey, data.WithdrawalCredentials, data.Signature, roots[i])
		require.NoError(t, err, "Could not deposit to deposit contract")

		testAcc.Backend.Commit()
	}

	query := ethereum.FilterQuery{
		Addresses: []common.Address{
			web3Service.cfg.depositContractAddr,
		},
	}

	logs, err := testAcc.Backend.FilterLogs(web3Service.ctx, query)
	require.NoError(t, err, "Unable to retrieve logs")
	require.Equal(t, depositsReqForChainStart, len(logs))

	// Set up our subscriber now to listen for the chain started event.
	stateChannel := make(chan *feed.Event, 1)
	stateSub := web3Service.cfg.stateNotifier.StateFeed().Subscribe(stateChannel)
	defer stateSub.Unsubscribe()

	for i := range logs {
		err = web3Service.ProcessLog(context.Background(), &logs[i])
		require.NoError(t, err)
	}

	err = web3Service.ProcessETH1Block(context.Background(), big.NewInt(int64(logs[len(logs)-1].BlockNumber)))
	require.NoError(t, err)

	cachedDeposits := web3Service.chainStartData.ChainstartDeposits
	require.Equal(t, depositsReqForChainStart, len(cachedDeposits))

	// Receive the chain started event.
	for started := false; !started; {
		event := <-stateChannel
		if event.Type == statefeed.ChainStarted {
			started = true
		}
	}

	require.LogsDoNotContain(t, hook, "Unable to unpack ChainStart log data")
	require.LogsDoNotContain(t, hook, "Receipt root from log doesn't match the root saved in memory")
	require.LogsDoNotContain(t, hook, "Invalid timestamp from log")
	require.LogsContain(t, hook, "Minimum number of validators reached for beacon-chain to start")

	hook.Reset()
}

func TestProcessETH2GenesisLog_CorrectNumOfDeposits(t *testing.T) {
	params.SetupTestConfigCleanup(t)
	hook := logTest.NewGlobal()
	testAcc, err := mock.Setup()
	require.NoError(t, err, "Unable to set up simulated backend")
	kvStore := testDB.SetupDB(t)
	depositCache, err := depositcache.New()
	require.NoError(t, err)
	server, endpoint, err := mockExecution.SetupRPCServer()
	require.NoError(t, err)
	t.Cleanup(func() {
		server.Stop()
	})

	web3Service, err := NewService(context.Background(),
		WithHttpEndpoint(endpoint),
		WithDepositContractAddress(testAcc.ContractAddr),
		WithDatabase(kvStore),
		WithDepositCache(depositCache),
	)
	require.NoError(t, err, "unable to setup web3 ETH1.0 chain service")
	web3Service = setDefaultMocks(web3Service)
	web3Service.depositContractCaller, err = contracts.NewDepositContractCaller(testAcc.ContractAddr, testAcc.Backend)
	require.NoError(t, err)
	web3Service.rpcClient = &mockExecution.RPCClient{Backend: testAcc.Backend}
	web3Service.httpLogger = testAcc.Backend
	web3Service.latestEth1Data.LastRequestedBlock = 0
	web3Service.latestEth1Data.BlockHeight = testAcc.Backend.Blockchain().CurrentBlock().Number.Uint64()
	web3Service.latestEth1Data.BlockTime = testAcc.Backend.Blockchain().CurrentBlock().Time
	bConfig := params.MinimalSpecConfig().Copy()
	bConfig.MinGenesisTime = 0
	bConfig.SecondsPerETH1Block = 10
	params.OverrideBeaconConfig(bConfig)
	nConfig := params.BeaconNetworkConfig()
	nConfig.ContractDeploymentBlock = 0
	params.OverrideBeaconNetworkConfig(nConfig)

	testAcc.Backend.Commit()

	totalNumOfDeposits := depositsReqForChainStart + 30

	deposits, _, err := util.DeterministicDepositsAndKeys(uint64(totalNumOfDeposits))
	require.NoError(t, err)
	_, depositRoots, err := util.DeterministicDepositTrie(len(deposits))
	require.NoError(t, err)
	depositOffset := 5

	// 64 Validators are used as size required for beacon-chain to start. This number
	// is defined in the deposit contract as the number required for the testnet. The actual number
	// is 2**14
	for i := 0; i < totalNumOfDeposits; i++ {
		data := deposits[i].Data
		testAcc.TxOpts.Value = mock.Amount32Eth()
		testAcc.TxOpts.GasLimit = 1000000
		_, err = testAcc.Contract.Deposit(testAcc.TxOpts, data.PublicKey, data.WithdrawalCredentials, data.Signature, depositRoots[i])
		require.NoError(t, err, "Could not deposit to deposit contract")
		// pack 8 deposits into a block with an offset of
		// 5
		if (i+1)%8 == depositOffset {
			testAcc.Backend.Commit()
		}
	}
	// Forward the chain to account for the follow distance
	for i := uint64(0); i < params.BeaconConfig().Eth1FollowDistance; i++ {
		testAcc.Backend.Commit()
	}
	web3Service.latestEth1Data.BlockHeight = testAcc.Backend.Blockchain().CurrentBlock().Number.Uint64()
	web3Service.latestEth1Data.BlockTime = testAcc.Backend.Blockchain().CurrentBlock().Time

	// Set up our subscriber now to listen for the chain started event.
	stateChannel := make(chan *feed.Event, 1)
	stateSub := web3Service.cfg.stateNotifier.StateFeed().Subscribe(stateChannel)
	defer stateSub.Unsubscribe()

	err = web3Service.processPastLogs(context.Background())
	require.NoError(t, err)

	cachedDeposits := web3Service.chainStartData.ChainstartDeposits
	requiredDepsForChainstart := depositsReqForChainStart + depositOffset
	require.Equal(t, requiredDepsForChainstart, len(cachedDeposits), "Did not cache the chain start deposits correctly")

	// Receive the chain started event.
	for started := false; !started; {
		event := <-stateChannel
		if event.Type == statefeed.ChainStarted {
			started = true
		}
	}

	require.LogsDoNotContain(t, hook, "Unable to unpack ChainStart log data")
	require.LogsDoNotContain(t, hook, "Receipt root from log doesn't match the root saved in memory")
	require.LogsDoNotContain(t, hook, "Invalid timestamp from log")
	require.LogsContain(t, hook, "Minimum number of validators reached for beacon-chain to start")

	hook.Reset()
}

func TestProcessETH2GenesisLog_LargePeriodOfNoLogs(t *testing.T) {
	params.SetupTestConfigCleanup(t)
	hook := logTest.NewGlobal()
	testAcc, err := mock.Setup()
	require.NoError(t, err, "Unable to set up simulated backend")
	kvStore := testDB.SetupDB(t)
	depositCache, err := depositcache.New()
	require.NoError(t, err)
	server, endpoint, err := mockExecution.SetupRPCServer()
	require.NoError(t, err)
	t.Cleanup(func() {
		server.Stop()
	})

	web3Service, err := NewService(context.Background(),
		WithHttpEndpoint(endpoint),
		WithDepositContractAddress(testAcc.ContractAddr),
		WithDatabase(kvStore),
		WithDepositCache(depositCache),
	)
	require.NoError(t, err, "unable to setup web3 ETH1.0 chain service")
	web3Service = setDefaultMocks(web3Service)
	web3Service.depositContractCaller, err = contracts.NewDepositContractCaller(testAcc.ContractAddr, testAcc.Backend)
	require.NoError(t, err)
	web3Service.rpcClient = &mockExecution.RPCClient{Backend: testAcc.Backend}
	web3Service.httpLogger = testAcc.Backend
	web3Service.latestEth1Data.LastRequestedBlock = 0
	web3Service.latestEth1Data.BlockHeight = testAcc.Backend.Blockchain().CurrentBlock().Number.Uint64()
	web3Service.latestEth1Data.BlockTime = testAcc.Backend.Blockchain().CurrentBlock().Time
	bConfig := params.MinimalSpecConfig().Copy()
	bConfig.SecondsPerETH1Block = 10
	params.OverrideBeaconConfig(bConfig)
	nConfig := params.BeaconNetworkConfig()
	nConfig.ContractDeploymentBlock = 0
	params.OverrideBeaconNetworkConfig(nConfig)

	testAcc.Backend.Commit()

	totalNumOfDeposits := depositsReqForChainStart + 30

	deposits, _, err := util.DeterministicDepositsAndKeys(uint64(totalNumOfDeposits))
	require.NoError(t, err)
	_, depositRoots, err := util.DeterministicDepositTrie(len(deposits))
	require.NoError(t, err)
	depositOffset := 5

	// 64 Validators are used as size required for beacon-chain to start. This number
	// is defined in the deposit contract as the number required for the testnet. The actual number
	// is 2**14
	for i := 0; i < totalNumOfDeposits; i++ {
		data := deposits[i].Data
		testAcc.TxOpts.Value = mock.Amount32Eth()
		testAcc.TxOpts.GasLimit = 1000000
		_, err = testAcc.Contract.Deposit(testAcc.TxOpts, data.PublicKey, data.WithdrawalCredentials, data.Signature, depositRoots[i])
		require.NoError(t, err, "Could not deposit to deposit contract")
		// pack 8 deposits into a block with an offset of
		// 5
		if (i+1)%8 == depositOffset {
			testAcc.Backend.Commit()
		}
	}
	// Forward the chain to 'mine' blocks without logs
	for i := uint64(0); i < 1500; i++ {
		testAcc.Backend.Commit()
	}
	wantedGenesisTime := testAcc.Backend.Blockchain().CurrentBlock().Time

	// Forward the chain to account for the follow distance
	for i := uint64(0); i < params.BeaconConfig().Eth1FollowDistance; i++ {
		testAcc.Backend.Commit()
	}
	web3Service.latestEth1Data.BlockHeight = testAcc.Backend.Blockchain().CurrentBlock().Number.Uint64()
	web3Service.latestEth1Data.BlockTime = testAcc.Backend.Blockchain().CurrentBlock().Time

	// Set the genesis time 500 blocks ahead of the last
	// deposit log.
	bConfig = params.MinimalSpecConfig().Copy()
	bConfig.MinGenesisTime = wantedGenesisTime - 10
	params.OverrideBeaconConfig(bConfig)

	// Set up our subscriber now to listen for the chain started event.
	stateChannel := make(chan *feed.Event, 1)
	stateSub := web3Service.cfg.stateNotifier.StateFeed().Subscribe(stateChannel)
	defer stateSub.Unsubscribe()

	err = web3Service.processPastLogs(context.Background())
	require.NoError(t, err)

	cachedDeposits := web3Service.chainStartData.ChainstartDeposits
	require.Equal(t, totalNumOfDeposits, len(cachedDeposits), "Did not cache the chain start deposits correctly")

	// Receive the chain started event.
	for started := false; !started; {
		event := <-stateChannel
		if event.Type == statefeed.ChainStarted {
			started = true
		}
	}

	require.LogsDoNotContain(t, hook, "Unable to unpack ChainStart log data")
	require.LogsDoNotContain(t, hook, "Receipt root from log doesn't match the root saved in memory")
	require.LogsDoNotContain(t, hook, "Invalid timestamp from log")
	require.LogsContain(t, hook, "Minimum number of validators reached for beacon-chain to start")

	hook.Reset()
}

func TestCheckForChainstart_NoValidator(t *testing.T) {
	hook := logTest.NewGlobal()
	testAcc, err := mock.Setup()
	require.NoError(t, err, "Unable to set up simulated backend")
	beaconDB := testDB.SetupDB(t)
	s := newPowchainService(t, testAcc, beaconDB)
	s.processChainStartIfReady(context.Background(), [32]byte{}, nil, 0)
	require.LogsDoNotContain(t, hook, "Could not determine active validator count from pre genesis state")
}

func newPowchainService(t *testing.T, eth1Backend *mock.TestAccount, beaconDB db.Database) *Service {
	depositCache, err := depositcache.New()
	require.NoError(t, err)
	server, endpoint, err := mockExecution.SetupRPCServer()
	require.NoError(t, err)
	t.Cleanup(func() {
		server.Stop()
	})
	web3Service, err := NewService(context.Background(),
		WithHttpEndpoint(endpoint),
		WithDepositContractAddress(eth1Backend.ContractAddr),
		WithDatabase(beaconDB),
		WithDepositCache(depositCache),
	)
	require.NoError(t, err, "unable to setup web3 ETH1.0 chain service")
	web3Service = setDefaultMocks(web3Service)
	web3Service.depositContractCaller, err = contracts.NewDepositContractCaller(eth1Backend.ContractAddr, eth1Backend.Backend)
	require.NoError(t, err)

	web3Service.rpcClient = &mockExecution.RPCClient{Backend: eth1Backend.Backend}
	web3Service.httpLogger = &goodLogger{backend: eth1Backend.Backend}
	params.SetupTestConfigCleanup(t)
	bConfig := params.MinimalSpecConfig().Copy()
	bConfig.MinGenesisTime = 0
	params.OverrideBeaconConfig(bConfig)
	return web3Service
}
