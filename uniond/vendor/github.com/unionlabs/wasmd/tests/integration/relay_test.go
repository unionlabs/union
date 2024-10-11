package integration

import (
	"encoding/json"
	"errors"
	"testing"
	"time"

	wasmvm "github.com/CosmWasm/wasmvm/v2"
	wasmvmtypes "github.com/CosmWasm/wasmvm/v2/types"
	ibctransfertypes "github.com/cosmos/ibc-go/v8/modules/apps/transfer/types"
	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types" //nolint:staticcheck
	channeltypes "github.com/cosmos/ibc-go/v8/modules/core/04-channel/types"
	ibctesting "github.com/cosmos/ibc-go/v8/testing"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	errorsmod "cosmossdk.io/errors"
	sdkmath "cosmossdk.io/math"

	sdk "github.com/cosmos/cosmos-sdk/types"

	"github.com/CosmWasm/wasmd/app"
	wasmibctesting "github.com/CosmWasm/wasmd/tests/ibctesting"
	wasmkeeper "github.com/CosmWasm/wasmd/x/wasm/keeper"
	"github.com/CosmWasm/wasmd/x/wasm/keeper/wasmtesting"
	"github.com/CosmWasm/wasmd/x/wasm/types"
)

func TestFromIBCTransferToContract(t *testing.T) {
	// scenario: given two chains,
	//           with a contract on chain B
	//           then the contract can handle the receiving side of an ics20 transfer
	//           that was started on chain A via ibc transfer module

	transferAmount := sdkmath.NewInt(1)
	specs := map[string]struct {
		contract                    wasmtesting.IBCContractCallbacks
		setupContract               func(t *testing.T, contract wasmtesting.IBCContractCallbacks, chain *wasmibctesting.TestChain)
		expChainAPendingSendPackets int
		expChainBPendingSendPackets int
		expChainABalanceDiff        sdkmath.Int
		expChainBBalanceDiff        sdkmath.Int
		expErr                      bool
	}{
		"ack": {
			contract: &ackReceiverContract{},
			setupContract: func(t *testing.T, contract wasmtesting.IBCContractCallbacks, chain *wasmibctesting.TestChain) {
				c := contract.(*ackReceiverContract)
				c.t = t
				c.chain = chain
			},
			expChainAPendingSendPackets: 0,
			expChainBPendingSendPackets: 0,
			expChainABalanceDiff:        transferAmount.Neg(),
			expChainBBalanceDiff:        transferAmount,
		},
		"nack": {
			contract: &nackReceiverContract{},
			setupContract: func(t *testing.T, contract wasmtesting.IBCContractCallbacks, chain *wasmibctesting.TestChain) {
				c := contract.(*nackReceiverContract)
				c.t = t
			},
			expChainAPendingSendPackets: 0,
			expChainBPendingSendPackets: 0,
			expChainABalanceDiff:        sdkmath.ZeroInt(),
			expChainBBalanceDiff:        sdkmath.ZeroInt(),
		},
		"error": {
			contract: &errorReceiverContract{},
			setupContract: func(t *testing.T, contract wasmtesting.IBCContractCallbacks, chain *wasmibctesting.TestChain) {
				c := contract.(*errorReceiverContract)
				c.t = t
			},
			expChainAPendingSendPackets: 1,
			expChainBPendingSendPackets: 0,
			expChainABalanceDiff:        transferAmount.Neg(),
			expChainBBalanceDiff:        sdkmath.ZeroInt(),
			expErr:                      true,
		},
	}
	for name, spec := range specs {
		t.Run(name, func(t *testing.T) {
			var (
				chainAOpts = []wasmkeeper.Option{wasmkeeper.WithWasmEngine(
					wasmtesting.NewIBCContractMockWasmEngine(spec.contract),
				)}
				coordinator = wasmibctesting.NewCoordinator(t, 2, []wasmkeeper.Option{}, chainAOpts)
				chainA      = coordinator.GetChain(wasmibctesting.GetChainID(1))
				chainB      = coordinator.GetChain(wasmibctesting.GetChainID(2))
			)
			coordinator.CommitBlock(chainA, chainB)
			myContractAddr := chainB.SeedNewContractInstance()
			contractBPortID := chainB.ContractInfo(myContractAddr).IBCPortID

			spec.setupContract(t, spec.contract, chainB)

			path := wasmibctesting.NewPath(chainA, chainB)
			path.EndpointA.ChannelConfig = &ibctesting.ChannelConfig{
				PortID:  "transfer",
				Version: ibctransfertypes.Version,
				Order:   channeltypes.UNORDERED,
			}
			path.EndpointB.ChannelConfig = &ibctesting.ChannelConfig{
				PortID:  contractBPortID,
				Version: ibctransfertypes.Version,
				Order:   channeltypes.UNORDERED,
			}

			coordinator.SetupConnections(path)
			coordinator.CreateChannels(path)

			originalChainABalance := chainA.Balance(chainA.SenderAccount.GetAddress(), sdk.DefaultBondDenom)
			// when transfer via sdk transfer from A (module) -> B (contract)
			coinToSendToB := sdk.NewCoin(sdk.DefaultBondDenom, transferAmount)
			timeoutHeight := clienttypes.NewHeight(1, 110)

			msg := ibctransfertypes.NewMsgTransfer(path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, coinToSendToB, chainA.SenderAccount.GetAddress().String(), chainB.SenderAccount.GetAddress().String(), timeoutHeight, 0, "")
			_, err := chainA.SendMsgs(msg)
			require.NoError(t, err)
			require.NoError(t, path.EndpointB.UpdateClient())

			// then
			require.Equal(t, 1, len(chainA.PendingSendPackets))
			require.Equal(t, 0, len(chainB.PendingSendPackets))

			// and when relay to chain B and handle Ack on chain A
			err = coordinator.RelayAndAckPendingPackets(path)
			if spec.expErr {
				require.Error(t, err)
			} else {
				require.NoError(t, err)
			}

			// then
			require.Equal(t, spec.expChainAPendingSendPackets, len(chainA.PendingSendPackets))
			require.Equal(t, spec.expChainBPendingSendPackets, len(chainB.PendingSendPackets))

			// and source chain balance was decreased
			newChainABalance := chainA.Balance(chainA.SenderAccount.GetAddress(), sdk.DefaultBondDenom)
			assert.Equal(t, originalChainABalance.Amount.Add(spec.expChainABalanceDiff), newChainABalance.Amount)

			// and dest chain balance contains voucher
			expBalance := ibctransfertypes.GetTransferCoin(path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID, coinToSendToB.Denom, spec.expChainBBalanceDiff)
			gotBalance := chainB.Balance(chainB.SenderAccount.GetAddress(), expBalance.Denom)
			assert.Equal(t, expBalance, gotBalance, "got total balance: %s", chainB.AllBalances(chainB.SenderAccount.GetAddress()))
		})
	}
}

func TestContractCanInitiateIBCTransferMsg(t *testing.T) {
	// scenario: given two chains,
	//           with a contract on chain A
	//           then the contract can start an ibc transfer via ibctransfertypes.NewMsgTransfer
	//           that is handled on chain A by the ibc transfer module and
	//           received on chain B via ibc transfer module as well

	myContract := &sendViaIBCTransferContract{t: t}
	var (
		chainAOpts = []wasmkeeper.Option{
			wasmkeeper.WithWasmEngine(
				wasmtesting.NewIBCContractMockWasmEngine(myContract)),
		}
		coordinator = wasmibctesting.NewCoordinator(t, 2, chainAOpts)
		chainA      = coordinator.GetChain(wasmibctesting.GetChainID(1))
		chainB      = coordinator.GetChain(wasmibctesting.GetChainID(2))
	)
	myContractAddr := chainA.SeedNewContractInstance()
	coordinator.CommitBlock(chainA, chainB)

	path := wasmibctesting.NewPath(chainA, chainB)
	path.EndpointA.ChannelConfig = &ibctesting.ChannelConfig{
		PortID:  ibctransfertypes.PortID,
		Version: ibctransfertypes.Version,
		Order:   channeltypes.UNORDERED,
	}
	path.EndpointB.ChannelConfig = &ibctesting.ChannelConfig{
		PortID:  ibctransfertypes.PortID,
		Version: ibctransfertypes.Version,
		Order:   channeltypes.UNORDERED,
	}
	coordinator.SetupConnections(path)
	coordinator.CreateChannels(path)

	// when contract is triggered to send IBCTransferMsg
	receiverAddress := chainB.SenderAccount.GetAddress()
	coinToSendToB := sdk.NewCoin(sdk.DefaultBondDenom, sdkmath.NewInt(100))

	// start transfer from chainA to chainB
	startMsg := &types.MsgExecuteContract{
		Sender:   chainA.SenderAccount.GetAddress().String(),
		Contract: myContractAddr.String(),
		Msg: startTransfer{
			ChannelID:    path.EndpointA.ChannelID,
			CoinsToSend:  coinToSendToB,
			ReceiverAddr: receiverAddress.String(),
		}.GetBytes(),
	}
	// trigger contract to start the transfer
	_, err := chainA.SendMsgs(startMsg)
	require.NoError(t, err)

	// then
	require.Equal(t, 1, len(chainA.PendingSendPackets))
	require.Equal(t, 0, len(chainB.PendingSendPackets))

	// and when relay to chain B and handle Ack on chain A
	err = coordinator.RelayAndAckPendingPackets(path)
	require.NoError(t, err)

	// then
	require.Equal(t, 0, len(chainA.PendingSendPackets))
	require.Equal(t, 0, len(chainB.PendingSendPackets))

	// and dest chain balance contains voucher
	bankKeeperB := chainB.App.(*app.WasmApp).BankKeeper
	expBalance := ibctransfertypes.GetTransferCoin(path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID, coinToSendToB.Denom, coinToSendToB.Amount)
	gotBalance := chainB.Balance(chainB.SenderAccount.GetAddress(), expBalance.Denom)
	assert.Equal(t, expBalance, gotBalance, "got total balance: %s", bankKeeperB.GetAllBalances(chainB.GetContext(), chainB.SenderAccount.GetAddress()))
}

func TestContractCanEmulateIBCTransferMessage(t *testing.T) {
	// scenario: given two chains,
	//           with a contract on chain A
	//           then the contract can emulate the ibc transfer module in the contract to send an ibc packet
	//           which is received on chain B via ibc transfer module

	myContract := &sendEmulatedIBCTransferContract{t: t}

	var (
		chainAOpts = []wasmkeeper.Option{
			wasmkeeper.WithWasmEngine(
				wasmtesting.NewIBCContractMockWasmEngine(myContract)),
		}
		coordinator = wasmibctesting.NewCoordinator(t, 2, chainAOpts)

		chainA = coordinator.GetChain(wasmibctesting.GetChainID(1))
		chainB = coordinator.GetChain(wasmibctesting.GetChainID(2))
	)
	myContractAddr := chainA.SeedNewContractInstance()
	myContract.contractAddr = myContractAddr.String()

	path := wasmibctesting.NewPath(chainA, chainB)
	path.EndpointA.ChannelConfig = &ibctesting.ChannelConfig{
		PortID:  chainA.ContractInfo(myContractAddr).IBCPortID,
		Version: ibctransfertypes.Version,
		Order:   channeltypes.UNORDERED,
	}
	path.EndpointB.ChannelConfig = &ibctesting.ChannelConfig{
		PortID:  ibctransfertypes.PortID,
		Version: ibctransfertypes.Version,
		Order:   channeltypes.UNORDERED,
	}
	coordinator.SetupConnections(path)
	coordinator.CreateChannels(path)

	// when contract is triggered to send the ibc package to chain B
	timeout := uint64(chainB.LastHeader.Header.Time.Add(time.Hour).UnixNano()) // enough time to not timeout
	receiverAddress := chainB.SenderAccount.GetAddress()
	coinToSendToB := sdk.NewCoin(sdk.DefaultBondDenom, sdkmath.NewInt(100))

	// start transfer from chainA to chainB
	startMsg := &types.MsgExecuteContract{
		Sender:   chainA.SenderAccount.GetAddress().String(),
		Contract: myContractAddr.String(),
		Msg: startTransfer{
			ChannelID:       path.EndpointA.ChannelID,
			CoinsToSend:     coinToSendToB,
			ReceiverAddr:    receiverAddress.String(),
			ContractIBCPort: chainA.ContractInfo(myContractAddr).IBCPortID,
			Timeout:         timeout,
		}.GetBytes(),
		Funds: sdk.NewCoins(coinToSendToB),
	}
	_, err := chainA.SendMsgs(startMsg)
	require.NoError(t, err)

	// then
	require.Equal(t, 1, len(chainA.PendingSendPackets))
	require.Equal(t, 0, len(chainB.PendingSendPackets))

	// and when relay to chain B and handle Ack on chain A
	err = coordinator.RelayAndAckPendingPackets(path)
	require.NoError(t, err)

	// then
	require.Equal(t, 0, len(chainA.PendingSendPackets))
	require.Equal(t, 0, len(chainB.PendingSendPackets))

	// and dest chain balance contains voucher
	bankKeeperB := chainB.App.(*app.WasmApp).BankKeeper
	expBalance := ibctransfertypes.GetTransferCoin(path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID, coinToSendToB.Denom, coinToSendToB.Amount)
	gotBalance := chainB.Balance(chainB.SenderAccount.GetAddress(), expBalance.Denom)
	assert.Equal(t, expBalance, gotBalance, "got total balance: %s", bankKeeperB.GetAllBalances(chainB.GetContext(), chainB.SenderAccount.GetAddress()))
}

func TestContractCanEmulateIBCTransferMessageWithTimeout(t *testing.T) {
	// scenario: given two chains,
	//           with a contract on chain A
	//           then the contract can emulate the ibc transfer module in the contract to send an ibc packet
	//           which is not received on chain B and times out

	myContract := &sendEmulatedIBCTransferContract{t: t}

	var (
		chainAOpts = []wasmkeeper.Option{
			wasmkeeper.WithWasmEngine(
				wasmtesting.NewIBCContractMockWasmEngine(myContract)),
		}
		coordinator = wasmibctesting.NewCoordinator(t, 2, chainAOpts)

		chainA = coordinator.GetChain(wasmibctesting.GetChainID(1))
		chainB = coordinator.GetChain(wasmibctesting.GetChainID(2))
	)
	coordinator.CommitBlock(chainA, chainB)
	myContractAddr := chainA.SeedNewContractInstance()
	myContract.contractAddr = myContractAddr.String()

	path := wasmibctesting.NewPath(chainA, chainB)
	path.EndpointA.ChannelConfig = &ibctesting.ChannelConfig{
		PortID:  chainA.ContractInfo(myContractAddr).IBCPortID,
		Version: ibctransfertypes.Version,
		Order:   channeltypes.UNORDERED,
	}
	path.EndpointB.ChannelConfig = &ibctesting.ChannelConfig{
		PortID:  ibctransfertypes.PortID,
		Version: ibctransfertypes.Version,
		Order:   channeltypes.UNORDERED,
	}
	coordinator.SetupConnections(path)
	coordinator.CreateChannels(path)
	coordinator.UpdateTime()

	// when contract is triggered to send the ibc package to chain B
	timeout := uint64(chainB.LastHeader.Header.Time.Add(time.Nanosecond).UnixNano()) // will timeout
	receiverAddress := chainB.SenderAccount.GetAddress()
	coinToSendToB := sdk.NewCoin(sdk.DefaultBondDenom, sdkmath.NewInt(100))
	initialContractBalance := chainA.Balance(myContractAddr, sdk.DefaultBondDenom)
	initialSenderBalance := chainA.Balance(chainA.SenderAccount.GetAddress(), sdk.DefaultBondDenom)

	// custom payload data to be transferred into a proper ICS20 ibc packet
	startMsg := &types.MsgExecuteContract{
		Sender:   chainA.SenderAccount.GetAddress().String(),
		Contract: myContractAddr.String(),
		Msg: startTransfer{
			ChannelID:       path.EndpointA.ChannelID,
			CoinsToSend:     coinToSendToB,
			ReceiverAddr:    receiverAddress.String(),
			ContractIBCPort: chainA.ContractInfo(myContractAddr).IBCPortID,
			Timeout:         timeout,
		}.GetBytes(),
		Funds: sdk.NewCoins(coinToSendToB),
	}
	_, err := chainA.SendMsgs(startMsg)
	require.NoError(t, err)
	coordinator.CommitBlock(chainA, chainB)
	// then
	require.Equal(t, 1, len(chainA.PendingSendPackets))
	require.Equal(t, 0, len(chainB.PendingSendPackets))
	newContractBalance := chainA.Balance(myContractAddr, sdk.DefaultBondDenom)
	assert.Equal(t, initialContractBalance.Add(coinToSendToB), newContractBalance) // hold in escrow

	// when timeout packet send (by the relayer)
	err = coordinator.TimeoutPendingPackets(path)
	require.NoError(t, err)
	coordinator.CommitBlock(chainA)

	// then
	require.Equal(t, 0, len(chainA.PendingSendPackets))
	require.Equal(t, 0, len(chainB.PendingSendPackets))

	// and then verify account balances restored
	newContractBalance = chainA.Balance(myContractAddr, sdk.DefaultBondDenom)
	assert.Equal(t, initialContractBalance.String(), newContractBalance.String())
	newSenderBalance := chainA.Balance(chainA.SenderAccount.GetAddress(), sdk.DefaultBondDenom)
	assert.Equal(t, initialSenderBalance.String(), newSenderBalance.String())
}

func TestContractEmulateIBCTransferMessageOnDiffContractIBCChannel(t *testing.T) {
	// scenario: given two chains, A and B
	//           with 2 contract A1 and A2 on chain A
	//           then the contract A2 try to send an ibc packet via IBC Channel that create by A1 and B
	myContractA1 := &sendEmulatedIBCTransferContract{}
	myContractA2 := &sendEmulatedIBCTransferContract{}

	var (
		chainAOpts = []wasmkeeper.Option{
			wasmkeeper.WithWasmEngine(
				wasmtesting.NewIBCContractMockWasmEngine(myContractA1),
			),
			wasmkeeper.WithWasmEngine(
				wasmtesting.NewIBCContractMockWasmEngine(myContractA2),
			),
		}

		coordinator = wasmibctesting.NewCoordinator(t, 2, chainAOpts)

		chainA = coordinator.GetChain(wasmibctesting.GetChainID(1))
		chainB = coordinator.GetChain(wasmibctesting.GetChainID(2))
	)

	coordinator.CommitBlock(chainA, chainB)
	myContractAddr1 := chainA.SeedNewContractInstance()
	myContractA1.contractAddr = myContractAddr1.String()
	myContractAddr2 := chainA.SeedNewContractInstance()
	myContractA2.contractAddr = myContractAddr2.String()

	path := wasmibctesting.NewPath(chainA, chainB)
	path.EndpointA.ChannelConfig = &ibctesting.ChannelConfig{
		PortID:  chainA.ContractInfo(myContractAddr1).IBCPortID,
		Version: ibctransfertypes.Version,
		Order:   channeltypes.UNORDERED,
	}
	path.EndpointB.ChannelConfig = &ibctesting.ChannelConfig{
		PortID:  ibctransfertypes.PortID,
		Version: ibctransfertypes.Version,
		Order:   channeltypes.UNORDERED,
	}
	coordinator.SetupConnections(path)
	coordinator.CreateChannels(path)

	// when contract is triggered to send the ibc package to chain B
	timeout := uint64(chainB.LastHeader.Header.Time.Add(time.Hour).UnixNano()) // enough time to not timeout
	receiverAddress := chainB.SenderAccount.GetAddress()
	coinToSendToB := sdk.NewCoin(sdk.DefaultBondDenom, sdkmath.NewInt(100))

	// start transfer from chainA - A2 to chainB via IBC channel
	startMsg := &types.MsgExecuteContract{
		Sender:   chainA.SenderAccount.GetAddress().String(),
		Contract: myContractAddr2.String(),
		Msg: startTransfer{
			ChannelID:    path.EndpointA.ChannelID,
			CoinsToSend:  coinToSendToB,
			ReceiverAddr: receiverAddress.String(),
			Timeout:      timeout,
		}.GetBytes(),
		Funds: sdk.NewCoins(coinToSendToB),
	}
	_, err := chainA.SendMsgs(startMsg)
	require.Error(t, err)
}

func TestContractHandlesChannelClose(t *testing.T) {
	// scenario: a contract is the sending side of an ics20 transfer but the packet was not received
	// on the destination chain within the timeout boundaries
	myContractA := &captureCloseContract{}
	myContractB := &captureCloseContract{}

	var (
		chainAOpts = []wasmkeeper.Option{
			wasmkeeper.WithWasmEngine(
				wasmtesting.NewIBCContractMockWasmEngine(myContractA)),
		}
		chainBOpts = []wasmkeeper.Option{
			wasmkeeper.WithWasmEngine(
				wasmtesting.NewIBCContractMockWasmEngine(myContractB)),
		}
		coordinator = wasmibctesting.NewCoordinator(t, 2, chainAOpts, chainBOpts)

		chainA = coordinator.GetChain(wasmibctesting.GetChainID(1))
		chainB = coordinator.GetChain(wasmibctesting.GetChainID(2))
	)

	coordinator.CommitBlock(chainA, chainB)
	myContractAddrA := chainA.SeedNewContractInstance()
	_ = chainB.SeedNewContractInstance() // skip one instance
	myContractAddrB := chainB.SeedNewContractInstance()

	path := wasmibctesting.NewPath(chainA, chainB)
	path.EndpointA.ChannelConfig = &ibctesting.ChannelConfig{
		PortID:  chainA.ContractInfo(myContractAddrA).IBCPortID,
		Version: ibctransfertypes.Version,
		Order:   channeltypes.UNORDERED,
	}
	path.EndpointB.ChannelConfig = &ibctesting.ChannelConfig{
		PortID:  chainB.ContractInfo(myContractAddrB).IBCPortID,
		Version: ibctransfertypes.Version,
		Order:   channeltypes.UNORDERED,
	}
	coordinator.SetupConnections(path)
	coordinator.CreateChannels(path)
	coordinator.CloseChannel(path)
	assert.True(t, myContractB.closeCalled)
}

func TestContractHandlesChannelCloseNotOwned(t *testing.T) {
	// scenario: given two chains,
	//           with a contract A1, A2 on chain A, contract B on chain B
	//           contract A2 try to close ibc channel that create between A1 and B

	myContractA1 := &closeChannelContract{}
	myContractA2 := &closeChannelContract{}
	myContractB := &closeChannelContract{}

	var (
		chainAOpts = []wasmkeeper.Option{
			wasmkeeper.WithWasmEngine(
				wasmtesting.NewIBCContractMockWasmEngine(myContractA1)),
			wasmkeeper.WithWasmEngine(
				wasmtesting.NewIBCContractMockWasmEngine(myContractA2)),
		}
		chainBOpts = []wasmkeeper.Option{
			wasmkeeper.WithWasmEngine(
				wasmtesting.NewIBCContractMockWasmEngine(myContractB)),
		}
		coordinator = wasmibctesting.NewCoordinator(t, 2, chainAOpts, chainBOpts)

		chainA = coordinator.GetChain(wasmibctesting.GetChainID(1))
		chainB = coordinator.GetChain(wasmibctesting.GetChainID(2))
	)

	coordinator.CommitBlock(chainA, chainB)
	myContractAddrA1 := chainA.SeedNewContractInstance()
	myContractAddrA2 := chainA.SeedNewContractInstance()
	_ = chainB.SeedNewContractInstance() // skip one instance
	_ = chainB.SeedNewContractInstance() // skip one instance
	myContractAddrB := chainB.SeedNewContractInstance()

	path := wasmibctesting.NewPath(chainA, chainB)
	path.EndpointA.ChannelConfig = &ibctesting.ChannelConfig{
		PortID:  chainA.ContractInfo(myContractAddrA1).IBCPortID,
		Version: ibctransfertypes.Version,
		Order:   channeltypes.UNORDERED,
	}
	path.EndpointB.ChannelConfig = &ibctesting.ChannelConfig{
		PortID:  chainB.ContractInfo(myContractAddrB).IBCPortID,
		Version: ibctransfertypes.Version,
		Order:   channeltypes.UNORDERED,
	}
	coordinator.SetupConnections(path)
	coordinator.CreateChannels(path)

	closeIBCChannelMsg := &types.MsgExecuteContract{
		Sender:   chainA.SenderAccount.GetAddress().String(),
		Contract: myContractAddrA2.String(),
		Msg: closeIBCChannel{
			ChannelID: path.EndpointA.ChannelID,
		}.GetBytes(),
		Funds: sdk.NewCoins(sdk.NewCoin(sdk.DefaultBondDenom, sdkmath.NewInt(100))),
	}

	_, err := chainA.SendMsgs(closeIBCChannelMsg)
	require.Error(t, err)
}

var _ wasmtesting.IBCContractCallbacks = &captureCloseContract{}

// contract that sets a flag on IBC channel close only.
type captureCloseContract struct {
	contractStub
	closeCalled bool
}

func (c *captureCloseContract) IBCChannelClose(_ wasmvm.Checksum, _ wasmvmtypes.Env, _ wasmvmtypes.IBCChannelCloseMsg, _ wasmvm.KVStore, _ wasmvm.GoAPI, _ wasmvm.Querier, _ wasmvm.GasMeter, _ uint64, _ wasmvmtypes.UFraction) (*wasmvmtypes.IBCBasicResult, uint64, error) {
	c.closeCalled = true
	return &wasmvmtypes.IBCBasicResult{Ok: &wasmvmtypes.IBCBasicResponse{}}, 1, nil
}

var _ wasmtesting.IBCContractCallbacks = &sendViaIBCTransferContract{}

// contract that initiates an ics-20 transfer on execute via sdk message
type sendViaIBCTransferContract struct {
	contractStub
	t *testing.T
}

func (s *sendViaIBCTransferContract) Execute(_ wasmvm.Checksum, _ wasmvmtypes.Env, _ wasmvmtypes.MessageInfo, executeMsg []byte, _ wasmvm.KVStore, _ wasmvm.GoAPI, _ wasmvm.Querier, _ wasmvm.GasMeter, _ uint64, _ wasmvmtypes.UFraction) (*wasmvmtypes.ContractResult, uint64, error) {
	var in startTransfer
	if err := json.Unmarshal(executeMsg, &in); err != nil {
		return nil, 0, err
	}
	ibcMsg := &wasmvmtypes.IBCMsg{
		Transfer: &wasmvmtypes.TransferMsg{
			ToAddress: in.ReceiverAddr,
			Amount:    wasmvmtypes.NewCoin(in.CoinsToSend.Amount.Uint64(), in.CoinsToSend.Denom),
			ChannelID: in.ChannelID,
			Timeout: wasmvmtypes.IBCTimeout{Block: &wasmvmtypes.IBCTimeoutBlock{
				Revision: 1,
				Height:   110,
			}},
		},
	}

	return &wasmvmtypes.ContractResult{Ok: &wasmvmtypes.Response{Messages: []wasmvmtypes.SubMsg{{ReplyOn: wasmvmtypes.ReplyNever, Msg: wasmvmtypes.CosmosMsg{IBC: ibcMsg}}}}}, 0, nil
}

var _ wasmtesting.IBCContractCallbacks = &sendEmulatedIBCTransferContract{}

// contract that interacts as an ics20 sending side via IBC packets
// It can also handle the timeout.
type sendEmulatedIBCTransferContract struct {
	contractStub
	t            *testing.T
	contractAddr string
}

func (s *sendEmulatedIBCTransferContract) Execute(_ wasmvm.Checksum, _ wasmvmtypes.Env, info wasmvmtypes.MessageInfo, executeMsg []byte, _ wasmvm.KVStore, _ wasmvm.GoAPI, _ wasmvm.Querier, _ wasmvm.GasMeter, _ uint64, _ wasmvmtypes.UFraction) (*wasmvmtypes.ContractResult, uint64, error) {
	var in startTransfer
	if err := json.Unmarshal(executeMsg, &in); err != nil {
		return nil, 0, err
	}
	require.Len(s.t, info.Funds, 1)
	require.Equal(s.t, in.CoinsToSend.Amount.String(), info.Funds[0].Amount)
	require.Equal(s.t, in.CoinsToSend.Denom, info.Funds[0].Denom)
	dataPacket := ibctransfertypes.NewFungibleTokenPacketData(
		in.CoinsToSend.Denom, in.CoinsToSend.Amount.String(), info.Sender, in.ReceiverAddr, "memo",
	)
	if err := dataPacket.ValidateBasic(); err != nil {
		return nil, 0, err
	}

	ibcMsg := &wasmvmtypes.IBCMsg{
		SendPacket: &wasmvmtypes.SendPacketMsg{
			ChannelID: in.ChannelID,
			Data:      dataPacket.GetBytes(),
			Timeout:   wasmvmtypes.IBCTimeout{Timestamp: in.Timeout},
		},
	}
	return &wasmvmtypes.ContractResult{Ok: &wasmvmtypes.Response{Messages: []wasmvmtypes.SubMsg{{ReplyOn: wasmvmtypes.ReplyNever, Msg: wasmvmtypes.CosmosMsg{IBC: ibcMsg}}}}}, 0, nil
}

func (s *sendEmulatedIBCTransferContract) IBCPacketTimeout(_ wasmvm.Checksum, _ wasmvmtypes.Env, msg wasmvmtypes.IBCPacketTimeoutMsg, _ wasmvm.KVStore, _ wasmvm.GoAPI, _ wasmvm.Querier, _ wasmvm.GasMeter, _ uint64, _ wasmvmtypes.UFraction) (*wasmvmtypes.IBCBasicResult, uint64, error) {
	packet := msg.Packet

	var data ibctransfertypes.FungibleTokenPacketData
	if err := ibctransfertypes.ModuleCdc.UnmarshalJSON(packet.Data, &data); err != nil {
		return nil, 0, err
	}
	if err := data.ValidateBasic(); err != nil {
		return nil, 0, err
	}
	amount, _ := sdkmath.NewIntFromString(data.Amount)

	returnTokens := &wasmvmtypes.BankMsg{
		Send: &wasmvmtypes.SendMsg{
			ToAddress: data.Sender,
			Amount:    wasmvmtypes.Array[wasmvmtypes.Coin]{wasmvmtypes.NewCoin(amount.Uint64(), data.Denom)},
		},
	}

	return &wasmvmtypes.IBCBasicResult{Ok: &wasmvmtypes.IBCBasicResponse{Messages: []wasmvmtypes.SubMsg{{ReplyOn: wasmvmtypes.ReplyNever, Msg: wasmvmtypes.CosmosMsg{Bank: returnTokens}}}}}, 0, nil
}

var _ wasmtesting.IBCContractCallbacks = &closeChannelContract{}

type closeChannelContract struct {
	contractStub
}

func (c *closeChannelContract) IBCChannelClose(_ wasmvm.Checksum, _ wasmvmtypes.Env, _ wasmvmtypes.IBCChannelCloseMsg, _ wasmvm.KVStore, _ wasmvm.GoAPI, _ wasmvm.Querier, _ wasmvm.GasMeter, _ uint64, _ wasmvmtypes.UFraction) (*wasmvmtypes.IBCBasicResult, uint64, error) {
	return &wasmvmtypes.IBCBasicResult{Ok: &wasmvmtypes.IBCBasicResponse{}}, 1, nil
}

func (c *closeChannelContract) Execute(_ wasmvm.Checksum, _ wasmvmtypes.Env, _ wasmvmtypes.MessageInfo, executeMsg []byte, _ wasmvm.KVStore, _ wasmvm.GoAPI, _ wasmvm.Querier, _ wasmvm.GasMeter, _ uint64, _ wasmvmtypes.UFraction) (*wasmvmtypes.ContractResult, uint64, error) {
	var in closeIBCChannel
	if err := json.Unmarshal(executeMsg, &in); err != nil {
		return nil, 0, err
	}
	ibcMsg := &wasmvmtypes.IBCMsg{
		CloseChannel: &wasmvmtypes.CloseChannelMsg{
			ChannelID: in.ChannelID,
		},
	}

	return &wasmvmtypes.ContractResult{Ok: &wasmvmtypes.Response{Messages: []wasmvmtypes.SubMsg{{ReplyOn: wasmvmtypes.ReplyNever, Msg: wasmvmtypes.CosmosMsg{IBC: ibcMsg}}}}}, 0, nil
}

type closeIBCChannel struct {
	ChannelID string
}

func (g closeIBCChannel) GetBytes() types.RawContractMessage {
	b, err := json.Marshal(g)
	if err != nil {
		panic(err)
	}
	return b
}

// custom contract execute payload
type startTransfer struct {
	ChannelID       string
	CoinsToSend     sdk.Coin
	ReceiverAddr    string
	ContractIBCPort string
	Timeout         uint64
}

func (g startTransfer) GetBytes() types.RawContractMessage {
	b, err := json.Marshal(g)
	if err != nil {
		panic(err)
	}
	return b
}

var _ wasmtesting.IBCContractCallbacks = &ackReceiverContract{}

// contract that acts as the receiving side for an ics-20 transfer.
type ackReceiverContract struct {
	contractStub
	t     *testing.T
	chain *wasmibctesting.TestChain
}

func (c *ackReceiverContract) IBCPacketReceive(_ wasmvm.Checksum, _ wasmvmtypes.Env, msg wasmvmtypes.IBCPacketReceiveMsg, _ wasmvm.KVStore, _ wasmvm.GoAPI, _ wasmvm.Querier, _ wasmvm.GasMeter, _ uint64, _ wasmvmtypes.UFraction) (*wasmvmtypes.IBCReceiveResult, uint64, error) {
	packet := msg.Packet

	var src ibctransfertypes.FungibleTokenPacketData
	if err := ibctransfertypes.ModuleCdc.UnmarshalJSON(packet.Data, &src); err != nil {
		return nil, 0, err
	}
	require.NoError(c.t, src.ValidateBasic())

	// call original ibctransfer keeper to not copy all code into this
	ibcPacket := toIBCPacket(packet)
	ctx := c.chain.GetContext() // HACK: please note that this is not reverted after checkTX
	err := c.chain.App.(*app.WasmApp).TransferKeeper.OnRecvPacket(ctx, ibcPacket, src)
	if err != nil {
		return nil, 0, errorsmod.Wrap(err, "within our smart contract")
	}

	var log []wasmvmtypes.EventAttribute // note: all events are under `wasm` event type
	ack := channeltypes.NewResultAcknowledgement([]byte{byte(1)}).Acknowledgement()
	return &wasmvmtypes.IBCReceiveResult{Ok: &wasmvmtypes.IBCReceiveResponse{Acknowledgement: ack, Attributes: log}}, 0, nil
}

func (c *ackReceiverContract) IBCPacketAck(_ wasmvm.Checksum, _ wasmvmtypes.Env, msg wasmvmtypes.IBCPacketAckMsg, _ wasmvm.KVStore, _ wasmvm.GoAPI, _ wasmvm.Querier, _ wasmvm.GasMeter, _ uint64, _ wasmvmtypes.UFraction) (*wasmvmtypes.IBCBasicResult, uint64, error) {
	var data ibctransfertypes.FungibleTokenPacketData
	if err := ibctransfertypes.ModuleCdc.UnmarshalJSON(msg.OriginalPacket.Data, &data); err != nil {
		return nil, 0, err
	}
	// call original ibctransfer keeper to not copy all code into this

	var ack channeltypes.Acknowledgement
	if err := ibctransfertypes.ModuleCdc.UnmarshalJSON(msg.Acknowledgement.Data, &ack); err != nil {
		return nil, 0, err
	}

	// call original ibctransfer keeper to not copy all code into this
	ctx := c.chain.GetContext() // HACK: please note that this is not reverted after checkTX
	ibcPacket := toIBCPacket(msg.OriginalPacket)
	err := c.chain.App.(*app.WasmApp).TransferKeeper.OnAcknowledgementPacket(ctx, ibcPacket, data, ack)
	if err != nil {
		return nil, 0, errorsmod.Wrap(err, "within our smart contract")
	}

	return &wasmvmtypes.IBCBasicResult{Ok: &wasmvmtypes.IBCBasicResponse{}}, 0, nil
}

// contract that acts as the receiving side for an ics-20 transfer and always returns a nack.
type nackReceiverContract struct {
	contractStub
	t *testing.T
}

func (c *nackReceiverContract) IBCPacketReceive(_ wasmvm.Checksum, _ wasmvmtypes.Env, msg wasmvmtypes.IBCPacketReceiveMsg, _ wasmvm.KVStore, _ wasmvm.GoAPI, _ wasmvm.Querier, _ wasmvm.GasMeter, _ uint64, _ wasmvmtypes.UFraction) (*wasmvmtypes.IBCReceiveResult, uint64, error) {
	packet := msg.Packet

	var src ibctransfertypes.FungibleTokenPacketData
	if err := ibctransfertypes.ModuleCdc.UnmarshalJSON(packet.Data, &src); err != nil {
		return nil, 0, err
	}
	require.NoError(c.t, src.ValidateBasic())
	return &wasmvmtypes.IBCReceiveResult{Err: "nack-testing"}, 0, nil
}

// contract that acts as the receiving side for an ics-20 transfer and always returns an error.
type errorReceiverContract struct {
	contractStub
	t *testing.T
}

func (c *errorReceiverContract) IBCPacketReceive(_ wasmvm.Checksum, _ wasmvmtypes.Env, msg wasmvmtypes.IBCPacketReceiveMsg, _ wasmvm.KVStore, _ wasmvm.GoAPI, _ wasmvm.Querier, _ wasmvm.GasMeter, _ uint64, _ wasmvmtypes.UFraction) (*wasmvmtypes.IBCReceiveResult, uint64, error) {
	packet := msg.Packet

	var src ibctransfertypes.FungibleTokenPacketData
	if err := ibctransfertypes.ModuleCdc.UnmarshalJSON(packet.Data, &src); err != nil {
		return nil, 0, err
	}
	require.NoError(c.t, src.ValidateBasic())
	return nil, 0, errors.New("error-testing")
}

// simple helper struct that implements connection setup methods.
type contractStub struct{}

func (s *contractStub) IBCChannelOpen(_ wasmvm.Checksum, _ wasmvmtypes.Env, _ wasmvmtypes.IBCChannelOpenMsg, _ wasmvm.KVStore, _ wasmvm.GoAPI, _ wasmvm.Querier, _ wasmvm.GasMeter, _ uint64, _ wasmvmtypes.UFraction) (*wasmvmtypes.IBCChannelOpenResult, uint64, error) {
	return &wasmvmtypes.IBCChannelOpenResult{Ok: &wasmvmtypes.IBC3ChannelOpenResponse{}}, 0, nil
}

func (s *contractStub) IBCChannelConnect(_ wasmvm.Checksum, _ wasmvmtypes.Env, _ wasmvmtypes.IBCChannelConnectMsg, _ wasmvm.KVStore, _ wasmvm.GoAPI, _ wasmvm.Querier, _ wasmvm.GasMeter, _ uint64, _ wasmvmtypes.UFraction) (*wasmvmtypes.IBCBasicResult, uint64, error) {
	return &wasmvmtypes.IBCBasicResult{Ok: &wasmvmtypes.IBCBasicResponse{}}, 0, nil
}

func (s *contractStub) IBCChannelClose(_ wasmvm.Checksum, _ wasmvmtypes.Env, _ wasmvmtypes.IBCChannelCloseMsg, _ wasmvm.KVStore, _ wasmvm.GoAPI, _ wasmvm.Querier, _ wasmvm.GasMeter, _ uint64, _ wasmvmtypes.UFraction) (*wasmvmtypes.IBCBasicResult, uint64, error) {
	panic("implement me")
}

func (s *contractStub) IBCPacketReceive(_ wasmvm.Checksum, _ wasmvmtypes.Env, _ wasmvmtypes.IBCPacketReceiveMsg, _ wasmvm.KVStore, _ wasmvm.GoAPI, _ wasmvm.Querier, _ wasmvm.GasMeter, _ uint64, _ wasmvmtypes.UFraction) (*wasmvmtypes.IBCReceiveResult, uint64, error) {
	panic("implement me")
}

func (s *contractStub) IBCPacketAck(_ wasmvm.Checksum, _ wasmvmtypes.Env, _ wasmvmtypes.IBCPacketAckMsg, _ wasmvm.KVStore, _ wasmvm.GoAPI, _ wasmvm.Querier, _ wasmvm.GasMeter, _ uint64, _ wasmvmtypes.UFraction) (*wasmvmtypes.IBCBasicResult, uint64, error) {
	return &wasmvmtypes.IBCBasicResult{Ok: &wasmvmtypes.IBCBasicResponse{}}, 0, nil
}

func (s *contractStub) IBCPacketTimeout(_ wasmvm.Checksum, _ wasmvmtypes.Env, _ wasmvmtypes.IBCPacketTimeoutMsg, _ wasmvm.KVStore, _ wasmvm.GoAPI, _ wasmvm.Querier, _ wasmvm.GasMeter, _ uint64, _ wasmvmtypes.UFraction) (*wasmvmtypes.IBCBasicResult, uint64, error) {
	panic("implement me")
}

func toIBCPacket(p wasmvmtypes.IBCPacket) channeltypes.Packet {
	var height clienttypes.Height
	if p.Timeout.Block != nil {
		height = clienttypes.NewHeight(p.Timeout.Block.Revision, p.Timeout.Block.Height)
	}
	return channeltypes.Packet{
		Sequence:           p.Sequence,
		SourcePort:         p.Src.PortID,
		SourceChannel:      p.Src.ChannelID,
		DestinationPort:    p.Dest.PortID,
		DestinationChannel: p.Dest.ChannelID,
		Data:               p.Data,
		TimeoutHeight:      height,
		TimeoutTimestamp:   p.Timeout.Timestamp,
	}
}
