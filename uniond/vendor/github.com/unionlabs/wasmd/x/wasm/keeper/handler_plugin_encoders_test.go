package keeper

import (
	"testing"

	wasmvmtypes "github.com/CosmWasm/wasmvm/v2/types"
	"github.com/cosmos/gogoproto/proto"
	ibctransfertypes "github.com/cosmos/ibc-go/v8/modules/apps/transfer/types"
	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types" //nolint:staticcheck
	channeltypes "github.com/cosmos/ibc-go/v8/modules/core/04-channel/types"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	sdkmath "cosmossdk.io/math"

	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	banktypes "cosmossdk.io/x/bank/types"
	distributiontypes "cosmossdk.io/x/distribution/types"
	govv1 "cosmossdk.io/x/gov/types/v1"
	stakingtypes "cosmossdk.io/x/staking/types"

	"github.com/CosmWasm/wasmd/x/wasm/keeper/wasmtesting"
	"github.com/CosmWasm/wasmd/x/wasm/types"
)

func TestEncoding(t *testing.T) {
	var (
		addr1       = RandomAccountAddress(t)
		addr2       = RandomAccountAddress(t)
		addr3       = RandomAccountAddress(t)
		invalidAddr = "xrnd1d02kd90n38qvr3qb9qof83fn2d2"
	)
	valAddr := make(sdk.ValAddress, types.SDKAddrLen)
	valAddr[0] = 12
	valAddr2 := make(sdk.ValAddress, types.SDKAddrLen)
	valAddr2[1] = 123

	jsonMsg := types.RawContractMessage(`{"foo": 123}`)

	bankMsg := &banktypes.MsgSend{
		FromAddress: addr2.String(),
		ToAddress:   addr1.String(),
		Amount: sdk.Coins{
			sdk.NewInt64Coin("uatom", 12345),
			sdk.NewInt64Coin("utgd", 54321),
		},
	}
	bankMsgBin := must(proto.Marshal(bankMsg))
	proposalMsg := &govv1.MsgSubmitProposal{
		Proposer:       addr1.String(),
		Messages:       []*codectypes.Any{must(codectypes.NewAnyWithValue(types.MsgStoreCodeFixture()))},
		InitialDeposit: sdk.NewCoins(sdk.NewInt64Coin("uatom", 12345)),
	}
	proposalMsgBin, err := proto.Marshal(proposalMsg)
	require.NoError(t, err)

	cases := map[string]struct {
		sender sdk.AccAddress
		srcMsg wasmvmtypes.CosmosMsg
		// set if valid
		output []sdk.Msg
		// set if expect mapping fails
		expError bool
	}{
		"simple send": {
			sender: addr1,
			srcMsg: wasmvmtypes.CosmosMsg{
				Bank: &wasmvmtypes.BankMsg{
					Send: &wasmvmtypes.SendMsg{
						ToAddress: addr2.String(),
						Amount: []wasmvmtypes.Coin{
							{
								Denom:  "uatom",
								Amount: "12345",
							},
							{
								Denom:  "usdt",
								Amount: "54321",
							},
						},
					},
				},
			},
			output: []sdk.Msg{
				&banktypes.MsgSend{
					FromAddress: addr1.String(),
					ToAddress:   addr2.String(),
					Amount: sdk.Coins{
						sdk.NewInt64Coin("uatom", 12345),
						sdk.NewInt64Coin("usdt", 54321),
					},
				},
			},
		},
		"invalid send amount": {
			sender: addr1,
			srcMsg: wasmvmtypes.CosmosMsg{
				Bank: &wasmvmtypes.BankMsg{
					Send: &wasmvmtypes.SendMsg{
						ToAddress: addr2.String(),
						Amount: []wasmvmtypes.Coin{
							{
								Denom:  "uatom",
								Amount: "123.456",
							},
						},
					},
				},
			},
			expError: true,
		},
		"invalid address": {
			sender: addr1,
			srcMsg: wasmvmtypes.CosmosMsg{
				Bank: &wasmvmtypes.BankMsg{
					Send: &wasmvmtypes.SendMsg{
						ToAddress: invalidAddr,
						Amount: []wasmvmtypes.Coin{
							{
								Denom:  "uatom",
								Amount: "7890",
							},
						},
					},
				},
			},
			expError: false, // addresses are checked in the handler
			output: []sdk.Msg{
				&banktypes.MsgSend{
					FromAddress: addr1.String(),
					ToAddress:   invalidAddr,
					Amount: sdk.Coins{
						sdk.NewInt64Coin("uatom", 7890),
					},
				},
			},
		},
		"wasm execute": {
			sender: addr1,
			srcMsg: wasmvmtypes.CosmosMsg{
				Wasm: &wasmvmtypes.WasmMsg{
					Execute: &wasmvmtypes.ExecuteMsg{
						ContractAddr: addr2.String(),
						Msg:          jsonMsg,
						Funds: []wasmvmtypes.Coin{
							wasmvmtypes.NewCoin(12, "eth"),
						},
					},
				},
			},
			output: []sdk.Msg{
				&types.MsgExecuteContract{
					Sender:   addr1.String(),
					Contract: addr2.String(),
					Msg:      jsonMsg,
					Funds:    sdk.NewCoins(sdk.NewInt64Coin("eth", 12)),
				},
			},
		},
		"wasm instantiate": {
			sender: addr1,
			srcMsg: wasmvmtypes.CosmosMsg{
				Wasm: &wasmvmtypes.WasmMsg{
					Instantiate: &wasmvmtypes.InstantiateMsg{
						CodeID: 7,
						Msg:    jsonMsg,
						Funds: []wasmvmtypes.Coin{
							wasmvmtypes.NewCoin(123, "eth"),
						},
						Label: "myLabel",
						Admin: addr2.String(),
					},
				},
			},
			output: []sdk.Msg{
				&types.MsgInstantiateContract{
					Sender: addr1.String(),
					CodeID: 7,
					Label:  "myLabel",
					Msg:    jsonMsg,
					Funds:  sdk.NewCoins(sdk.NewInt64Coin("eth", 123)),
					Admin:  addr2.String(),
				},
			},
		},
		"wasm instantiate2": {
			sender: addr1,
			srcMsg: wasmvmtypes.CosmosMsg{
				Wasm: &wasmvmtypes.WasmMsg{
					Instantiate2: &wasmvmtypes.Instantiate2Msg{
						CodeID: 7,
						Msg:    jsonMsg,
						Funds: []wasmvmtypes.Coin{
							wasmvmtypes.NewCoin(123, "eth"),
						},
						Label: "myLabel",
						Admin: addr2.String(),
						Salt:  []byte("mySalt"),
					},
				},
			},
			output: []sdk.Msg{
				&types.MsgInstantiateContract2{
					Sender: addr1.String(),
					Admin:  addr2.String(),
					CodeID: 7,
					Label:  "myLabel",
					Msg:    jsonMsg,
					Funds:  sdk.NewCoins(sdk.NewInt64Coin("eth", 123)),
					Salt:   []byte("mySalt"),
					FixMsg: false,
				},
			},
		},
		"wasm migrate": {
			sender: addr2,
			srcMsg: wasmvmtypes.CosmosMsg{
				Wasm: &wasmvmtypes.WasmMsg{
					Migrate: &wasmvmtypes.MigrateMsg{
						ContractAddr: addr1.String(),
						NewCodeID:    12,
						Msg:          jsonMsg,
					},
				},
			},
			output: []sdk.Msg{
				&types.MsgMigrateContract{
					Sender:   addr2.String(),
					Contract: addr1.String(),
					CodeID:   12,
					Msg:      jsonMsg,
				},
			},
		},
		"wasm update admin": {
			sender: addr2,
			srcMsg: wasmvmtypes.CosmosMsg{
				Wasm: &wasmvmtypes.WasmMsg{
					UpdateAdmin: &wasmvmtypes.UpdateAdminMsg{
						ContractAddr: addr1.String(),
						Admin:        addr3.String(),
					},
				},
			},
			output: []sdk.Msg{
				&types.MsgUpdateAdmin{
					Sender:   addr2.String(),
					Contract: addr1.String(),
					NewAdmin: addr3.String(),
				},
			},
		},
		"wasm clear admin": {
			sender: addr2,
			srcMsg: wasmvmtypes.CosmosMsg{
				Wasm: &wasmvmtypes.WasmMsg{
					ClearAdmin: &wasmvmtypes.ClearAdminMsg{
						ContractAddr: addr1.String(),
					},
				},
			},
			output: []sdk.Msg{
				&types.MsgClearAdmin{
					Sender:   addr2.String(),
					Contract: addr1.String(),
				},
			},
		},
		"staking delegate": {
			sender: addr1,
			srcMsg: wasmvmtypes.CosmosMsg{
				Staking: &wasmvmtypes.StakingMsg{
					Delegate: &wasmvmtypes.DelegateMsg{
						Validator: valAddr.String(),
						Amount:    wasmvmtypes.NewCoin(777, "stake"),
					},
				},
			},
			output: []sdk.Msg{
				&stakingtypes.MsgDelegate{
					DelegatorAddress: addr1.String(),
					ValidatorAddress: valAddr.String(),
					Amount:           sdk.NewInt64Coin("stake", 777),
				},
			},
		},
		"staking delegate to non-validator - invalid": {
			sender: addr1,
			srcMsg: wasmvmtypes.CosmosMsg{
				Staking: &wasmvmtypes.StakingMsg{
					Delegate: &wasmvmtypes.DelegateMsg{
						Validator: addr2.String(),
						Amount:    wasmvmtypes.NewCoin(777, "stake"),
					},
				},
			},
			expError: false, // fails in the handler
			output: []sdk.Msg{
				&stakingtypes.MsgDelegate{
					DelegatorAddress: addr1.String(),
					ValidatorAddress: addr2.String(),
					Amount:           sdk.NewInt64Coin("stake", 777),
				},
			},
		},
		"staking undelegate": {
			sender: addr1,
			srcMsg: wasmvmtypes.CosmosMsg{
				Staking: &wasmvmtypes.StakingMsg{
					Undelegate: &wasmvmtypes.UndelegateMsg{
						Validator: valAddr.String(),
						Amount:    wasmvmtypes.NewCoin(555, "stake"),
					},
				},
			},
			output: []sdk.Msg{
				&stakingtypes.MsgUndelegate{
					DelegatorAddress: addr1.String(),
					ValidatorAddress: valAddr.String(),
					Amount:           sdk.NewInt64Coin("stake", 555),
				},
			},
		},
		"staking redelegate": {
			sender: addr1,
			srcMsg: wasmvmtypes.CosmosMsg{
				Staking: &wasmvmtypes.StakingMsg{
					Redelegate: &wasmvmtypes.RedelegateMsg{
						SrcValidator: valAddr.String(),
						DstValidator: valAddr2.String(),
						Amount:       wasmvmtypes.NewCoin(222, "stake"),
					},
				},
			},
			output: []sdk.Msg{
				&stakingtypes.MsgBeginRedelegate{
					DelegatorAddress:    addr1.String(),
					ValidatorSrcAddress: valAddr.String(),
					ValidatorDstAddress: valAddr2.String(),
					Amount:              sdk.NewInt64Coin("stake", 222),
				},
			},
		},
		"staking withdraw (explicit recipient)": {
			sender: addr1,
			srcMsg: wasmvmtypes.CosmosMsg{
				Distribution: &wasmvmtypes.DistributionMsg{
					WithdrawDelegatorReward: &wasmvmtypes.WithdrawDelegatorRewardMsg{
						Validator: valAddr2.String(),
					},
				},
			},
			output: []sdk.Msg{
				&distributiontypes.MsgWithdrawDelegatorReward{
					DelegatorAddress: addr1.String(),
					ValidatorAddress: valAddr2.String(),
				},
			},
		},
		"staking set withdraw address": {
			sender: addr1,
			srcMsg: wasmvmtypes.CosmosMsg{
				Distribution: &wasmvmtypes.DistributionMsg{
					SetWithdrawAddress: &wasmvmtypes.SetWithdrawAddressMsg{
						Address: addr2.String(),
					},
				},
			},
			output: []sdk.Msg{
				&distributiontypes.MsgSetWithdrawAddress{
					DelegatorAddress: addr1.String(),
					WithdrawAddress:  addr2.String(),
				},
			},
		},
		"distribution fund community pool": {
			sender: addr1,
			srcMsg: wasmvmtypes.CosmosMsg{
				Distribution: &wasmvmtypes.DistributionMsg{
					FundCommunityPool: &wasmvmtypes.FundCommunityPoolMsg{
						Amount: wasmvmtypes.Array[wasmvmtypes.Coin]{
							wasmvmtypes.NewCoin(200, "stones"),
							wasmvmtypes.NewCoin(200, "feathers"),
						},
					},
				},
			},
			output: []sdk.Msg{
				&distributiontypes.MsgFundCommunityPool{
					Depositor: addr1.String(),
					Amount: sdk.NewCoins(
						sdk.NewInt64Coin("stones", 200),
						sdk.NewInt64Coin("feathers", 200),
					),
				},
			},
		},
		"stargate encoded bank msg": {
			sender: addr2,
			srcMsg: wasmvmtypes.CosmosMsg{
				Any: &wasmvmtypes.AnyMsg{
					TypeURL: "/cosmos.bank.v1beta1.MsgSend",
					Value:   bankMsgBin,
				},
			},
			output: []sdk.Msg{bankMsg},
		},
		"stargate encoded msg with any type": {
			sender: addr2,
			srcMsg: wasmvmtypes.CosmosMsg{
				Any: &wasmvmtypes.AnyMsg{
					TypeURL: "/cosmos.gov.v1.MsgSubmitProposal",
					Value:   proposalMsgBin,
				},
			},
			output: []sdk.Msg{proposalMsg},
		},
		"stargate encoded invalid typeUrl": {
			sender: addr2,
			srcMsg: wasmvmtypes.CosmosMsg{
				Any: &wasmvmtypes.AnyMsg{
					TypeURL: "/cosmos.bank.v2.MsgSend",
					Value:   bankMsgBin,
				},
			},
			expError: true,
		},
	}
	encodingConfig := MakeEncodingConfig(t)
	for name, tc := range cases {
		t.Run(name, func(t *testing.T) {
			encoder := DefaultEncoders(encodingConfig.Codec, wasmtesting.MockIBCTransferKeeper{})
			res, err := encoder.Encode(sdk.Context{}, tc.sender, "", tc.srcMsg)
			if tc.expError {
				assert.Error(t, err)
				return
			}
			require.NoError(t, err)
			assert.Equal(t, tc.output, res)
		})
	}
}

func TestEncodeIbcMsg(t *testing.T) {
	var (
		addr1 = RandomAccountAddress(t)
		addr2 = RandomAccountAddress(t)
	)

	cases := map[string]struct {
		sender             sdk.AccAddress
		srcMsg             wasmvmtypes.CosmosMsg
		srcContractIBCPort string
		transferPortSource types.ICS20TransferPortSource
		// set if valid
		output []sdk.Msg
		// set if expect mapping fails
		expError bool
	}{
		"IBC transfer with block timeout": {
			sender:             addr1,
			srcContractIBCPort: "myIBCPort",
			srcMsg: wasmvmtypes.CosmosMsg{
				IBC: &wasmvmtypes.IBCMsg{
					Transfer: &wasmvmtypes.TransferMsg{
						ChannelID: "myChanID",
						ToAddress: addr2.String(),
						Amount: wasmvmtypes.Coin{
							Denom:  "ALX",
							Amount: "1",
						},
						Timeout: wasmvmtypes.IBCTimeout{
							Block: &wasmvmtypes.IBCTimeoutBlock{Revision: 1, Height: 2},
						},
					},
				},
			},
			transferPortSource: wasmtesting.MockIBCTransferKeeper{GetPortFn: func(ctx sdk.Context) string {
				return "myTransferPort"
			}},
			output: []sdk.Msg{
				&ibctransfertypes.MsgTransfer{
					SourcePort:    "myTransferPort",
					SourceChannel: "myChanID",
					Token: sdk.Coin{
						Denom:  "ALX",
						Amount: sdkmath.NewInt(1),
					},
					Sender:        addr1.String(),
					Receiver:      addr2.String(),
					TimeoutHeight: clienttypes.Height{RevisionNumber: 1, RevisionHeight: 2},
				},
			},
		},
		"IBC transfer with time timeout": {
			sender:             addr1,
			srcContractIBCPort: "myIBCPort",
			srcMsg: wasmvmtypes.CosmosMsg{
				IBC: &wasmvmtypes.IBCMsg{
					Transfer: &wasmvmtypes.TransferMsg{
						ChannelID: "myChanID",
						ToAddress: addr2.String(),
						Amount: wasmvmtypes.Coin{
							Denom:  "ALX",
							Amount: "1",
						},
						Timeout: wasmvmtypes.IBCTimeout{Timestamp: 100},
					},
				},
			},
			transferPortSource: wasmtesting.MockIBCTransferKeeper{GetPortFn: func(ctx sdk.Context) string {
				return "transfer"
			}},
			output: []sdk.Msg{
				&ibctransfertypes.MsgTransfer{
					SourcePort:    "transfer",
					SourceChannel: "myChanID",
					Token: sdk.Coin{
						Denom:  "ALX",
						Amount: sdkmath.NewInt(1),
					},
					Sender:           addr1.String(),
					Receiver:         addr2.String(),
					TimeoutTimestamp: 100,
				},
			},
		},
		"IBC transfer with time and height timeout": {
			sender:             addr1,
			srcContractIBCPort: "myIBCPort",
			srcMsg: wasmvmtypes.CosmosMsg{
				IBC: &wasmvmtypes.IBCMsg{
					Transfer: &wasmvmtypes.TransferMsg{
						ChannelID: "myChanID",
						ToAddress: addr2.String(),
						Amount: wasmvmtypes.Coin{
							Denom:  "ALX",
							Amount: "1",
						},
						Timeout: wasmvmtypes.IBCTimeout{Timestamp: 100, Block: &wasmvmtypes.IBCTimeoutBlock{Height: 1, Revision: 2}},
					},
				},
			},
			transferPortSource: wasmtesting.MockIBCTransferKeeper{GetPortFn: func(ctx sdk.Context) string {
				return "transfer"
			}},
			output: []sdk.Msg{
				&ibctransfertypes.MsgTransfer{
					SourcePort:    "transfer",
					SourceChannel: "myChanID",
					Token: sdk.Coin{
						Denom:  "ALX",
						Amount: sdkmath.NewInt(1),
					},
					Sender:           addr1.String(),
					Receiver:         addr2.String(),
					TimeoutTimestamp: 100,
					TimeoutHeight:    clienttypes.NewHeight(2, 1),
				},
			},
		},
		"IBC transfer with memo": {
			sender:             addr1,
			srcContractIBCPort: "myIBCPort",
			srcMsg: wasmvmtypes.CosmosMsg{
				IBC: &wasmvmtypes.IBCMsg{
					Transfer: &wasmvmtypes.TransferMsg{
						ChannelID: "myChanID",
						ToAddress: addr2.String(),
						Amount: wasmvmtypes.Coin{
							Denom:  "ALX",
							Amount: "1",
						},
						Timeout: wasmvmtypes.IBCTimeout{Timestamp: 100},
						Memo:    "myMemo",
					},
				},
			},
			transferPortSource: wasmtesting.MockIBCTransferKeeper{GetPortFn: func(ctx sdk.Context) string {
				return "myTransferPort"
			}},
			output: []sdk.Msg{
				&ibctransfertypes.MsgTransfer{
					SourcePort:    "myTransferPort",
					SourceChannel: "myChanID",
					Token: sdk.Coin{
						Denom:  "ALX",
						Amount: sdkmath.NewInt(1),
					},
					Sender:           addr1.String(),
					Receiver:         addr2.String(),
					TimeoutTimestamp: 100,
					Memo:             "myMemo",
				},
			},
		},
		"IBC close channel": {
			sender:             addr1,
			srcContractIBCPort: "myIBCPort",
			srcMsg: wasmvmtypes.CosmosMsg{
				IBC: &wasmvmtypes.IBCMsg{
					CloseChannel: &wasmvmtypes.CloseChannelMsg{
						ChannelID: "channel-1",
					},
				},
			},
			output: []sdk.Msg{
				&channeltypes.MsgChannelCloseInit{
					PortId:    "wasm." + addr1.String(),
					ChannelId: "channel-1",
					Signer:    addr1.String(),
				},
			},
		},
	}
	encodingConfig := MakeEncodingConfig(t)
	for name, tc := range cases {
		t.Run(name, func(t *testing.T) {
			var ctx sdk.Context
			encoder := DefaultEncoders(encodingConfig.Codec, tc.transferPortSource)
			res, err := encoder.Encode(ctx, tc.sender, tc.srcContractIBCPort, tc.srcMsg)
			if tc.expError {
				assert.Error(t, err)
				return
			}
			require.NoError(t, err)
			assert.Equal(t, tc.output, res)
		})
	}
}

func TestEncodeGovMsg(t *testing.T) {
	myAddr := RandomAccountAddress(t)

	cases := map[string]struct {
		sender             sdk.AccAddress
		srcMsg             wasmvmtypes.CosmosMsg
		transferPortSource types.ICS20TransferPortSource
		// set if valid
		output []sdk.Msg
		// set if expect mapping fails
		expError bool
	}{
		"Gov vote: yes": {
			sender: myAddr,
			srcMsg: wasmvmtypes.CosmosMsg{
				Gov: &wasmvmtypes.GovMsg{
					Vote: &wasmvmtypes.VoteMsg{ProposalId: 1, Option: wasmvmtypes.Yes},
				},
			},
			output: []sdk.Msg{
				&govv1.MsgVote{
					ProposalId: 1,
					Voter:      myAddr.String(),
					Option:     govv1.OptionYes,
				},
			},
		},
		"Gov vote: No": {
			sender: myAddr,
			srcMsg: wasmvmtypes.CosmosMsg{
				Gov: &wasmvmtypes.GovMsg{
					Vote: &wasmvmtypes.VoteMsg{ProposalId: 1, Option: wasmvmtypes.No},
				},
			},
			output: []sdk.Msg{
				&govv1.MsgVote{
					ProposalId: 1,
					Voter:      myAddr.String(),
					Option:     govv1.OptionNo,
				},
			},
		},
		"Gov vote: Abstain": {
			sender: myAddr,
			srcMsg: wasmvmtypes.CosmosMsg{
				Gov: &wasmvmtypes.GovMsg{
					Vote: &wasmvmtypes.VoteMsg{ProposalId: 10, Option: wasmvmtypes.Abstain},
				},
			},
			output: []sdk.Msg{
				&govv1.MsgVote{
					ProposalId: 10,
					Voter:      myAddr.String(),
					Option:     govv1.OptionAbstain,
				},
			},
		},
		"Gov vote: No with veto": {
			sender: myAddr,
			srcMsg: wasmvmtypes.CosmosMsg{
				Gov: &wasmvmtypes.GovMsg{
					Vote: &wasmvmtypes.VoteMsg{ProposalId: 1, Option: wasmvmtypes.NoWithVeto},
				},
			},
			output: []sdk.Msg{
				&govv1.MsgVote{
					ProposalId: 1,
					Voter:      myAddr.String(),
					Option:     govv1.OptionNoWithVeto,
				},
			},
		},
		"Gov vote: unset option": {
			sender: myAddr,
			srcMsg: wasmvmtypes.CosmosMsg{
				Gov: &wasmvmtypes.GovMsg{
					Vote: &wasmvmtypes.VoteMsg{ProposalId: 1},
				},
			},
			expError: true,
		},
		"Gov weighted vote: single vote": {
			sender: myAddr,
			srcMsg: wasmvmtypes.CosmosMsg{
				Gov: &wasmvmtypes.GovMsg{
					VoteWeighted: &wasmvmtypes.VoteWeightedMsg{
						ProposalId: 1,
						Options: []wasmvmtypes.WeightedVoteOption{
							{Option: wasmvmtypes.Yes, Weight: "1"},
						},
					},
				},
			},
			output: []sdk.Msg{
				&govv1.MsgVoteWeighted{
					ProposalId: 1,
					Voter:      myAddr.String(),
					Options: []*govv1.WeightedVoteOption{
						{Option: govv1.OptionYes, Weight: sdkmath.LegacyNewDec(1).String()},
					},
				},
			},
		},
		"Gov weighted vote: splitted": {
			sender: myAddr,
			srcMsg: wasmvmtypes.CosmosMsg{
				Gov: &wasmvmtypes.GovMsg{
					VoteWeighted: &wasmvmtypes.VoteWeightedMsg{
						ProposalId: 1,
						Options: []wasmvmtypes.WeightedVoteOption{
							{Option: wasmvmtypes.Yes, Weight: "0.23"},
							{Option: wasmvmtypes.No, Weight: "0.24"},
							{Option: wasmvmtypes.Abstain, Weight: "0.26"},
							{Option: wasmvmtypes.NoWithVeto, Weight: "0.27"},
						},
					},
				},
			},
			output: []sdk.Msg{
				&govv1.MsgVoteWeighted{
					ProposalId: 1,
					Voter:      myAddr.String(),
					Options: []*govv1.WeightedVoteOption{
						{Option: govv1.OptionYes, Weight: sdkmath.LegacyNewDecWithPrec(23, 2).String()},
						{Option: govv1.OptionNo, Weight: sdkmath.LegacyNewDecWithPrec(24, 2).String()},
						{Option: govv1.OptionAbstain, Weight: sdkmath.LegacyNewDecWithPrec(26, 2).String()},
						{Option: govv1.OptionNoWithVeto, Weight: sdkmath.LegacyNewDecWithPrec(27, 2).String()},
					},
				},
			},
		},
		"Gov weighted vote: duplicate option - invalid": {
			sender: myAddr,
			srcMsg: wasmvmtypes.CosmosMsg{
				Gov: &wasmvmtypes.GovMsg{
					VoteWeighted: &wasmvmtypes.VoteWeightedMsg{
						ProposalId: 1,
						Options: []wasmvmtypes.WeightedVoteOption{
							{Option: wasmvmtypes.Yes, Weight: "0.5"},
							{Option: wasmvmtypes.Yes, Weight: "0.5"},
						},
					},
				},
			},
			output: []sdk.Msg{
				&govv1.MsgVoteWeighted{
					ProposalId: 1,
					Voter:      myAddr.String(),
					Options: []*govv1.WeightedVoteOption{
						{Option: govv1.OptionYes, Weight: sdkmath.LegacyNewDecWithPrec(5, 1).String()},
						{Option: govv1.OptionYes, Weight: sdkmath.LegacyNewDecWithPrec(5, 1).String()},
					},
				},
			},
		},
		"Gov weighted vote: weight sum exceeds 1- invalid": {
			sender: myAddr,
			srcMsg: wasmvmtypes.CosmosMsg{
				Gov: &wasmvmtypes.GovMsg{
					VoteWeighted: &wasmvmtypes.VoteWeightedMsg{
						ProposalId: 1,
						Options: []wasmvmtypes.WeightedVoteOption{
							{Option: wasmvmtypes.Yes, Weight: "0.51"},
							{Option: wasmvmtypes.No, Weight: "0.5"},
						},
					},
				},
			},
			output: []sdk.Msg{
				&govv1.MsgVoteWeighted{
					ProposalId: 1,
					Voter:      myAddr.String(),
					Options: []*govv1.WeightedVoteOption{
						{Option: govv1.OptionYes, Weight: sdkmath.LegacyNewDecWithPrec(51, 2).String()},
						{Option: govv1.OptionNo, Weight: sdkmath.LegacyNewDecWithPrec(5, 1).String()},
					},
				},
			},
		},
		"Gov weighted vote: weight sum less than 1 - invalid": {
			sender: myAddr,
			srcMsg: wasmvmtypes.CosmosMsg{
				Gov: &wasmvmtypes.GovMsg{
					VoteWeighted: &wasmvmtypes.VoteWeightedMsg{
						ProposalId: 1,
						Options: []wasmvmtypes.WeightedVoteOption{
							{Option: wasmvmtypes.Yes, Weight: "0.49"},
							{Option: wasmvmtypes.No, Weight: "0.5"},
						},
					},
				},
			},
			output: []sdk.Msg{
				&govv1.MsgVoteWeighted{
					ProposalId: 1,
					Voter:      myAddr.String(),
					Options: []*govv1.WeightedVoteOption{
						{Option: govv1.OptionYes, Weight: sdkmath.LegacyNewDecWithPrec(49, 2).String()},
						{Option: govv1.OptionNo, Weight: sdkmath.LegacyNewDecWithPrec(5, 1).String()},
					},
				},
			},
		},
	}
	encodingConfig := MakeEncodingConfig(t)
	for name, tc := range cases {
		t.Run(name, func(t *testing.T) {
			var ctx sdk.Context
			encoder := DefaultEncoders(encodingConfig.Codec, tc.transferPortSource)
			res, gotEncErr := encoder.Encode(ctx, tc.sender, "myIBCPort", tc.srcMsg)
			if tc.expError {
				assert.Error(t, gotEncErr)
				return
			}
			require.NoError(t, gotEncErr)
			assert.Equal(t, tc.output, res)
		})
	}
}

func TestConvertWasmCoinToSdkCoin(t *testing.T) {
	specs := map[string]struct {
		src    wasmvmtypes.Coin
		expErr bool
		expVal sdk.Coin
	}{
		"all good": {
			src: wasmvmtypes.Coin{
				Denom:  "foo",
				Amount: "1",
			},
			expVal: sdk.NewCoin("foo", sdkmath.NewIntFromUint64(1)),
		},
		"negative amount": {
			src: wasmvmtypes.Coin{
				Denom:  "foo",
				Amount: "-1",
			},
			expErr: true,
		},
		"denom to short": {
			src: wasmvmtypes.Coin{
				Denom:  "f",
				Amount: "1",
			},
			expErr: true,
		},
		"invalid demum char": {
			src: wasmvmtypes.Coin{
				Denom:  "&fff",
				Amount: "1",
			},
			expErr: true,
		},
		"not a number amount": {
			src: wasmvmtypes.Coin{
				Denom:  "foo",
				Amount: "bar",
			},
			expErr: true,
		},
	}
	for name, spec := range specs {
		t.Run(name, func(t *testing.T) {
			gotVal, gotErr := ConvertWasmCoinToSdkCoin(spec.src)
			if spec.expErr {
				require.Error(t, gotErr)
				return
			}
			require.NoError(t, gotErr)
			assert.Equal(t, spec.expVal, gotVal)
		})
	}
}

func TestConvertWasmCoinsToSdkCoins(t *testing.T) {
	specs := map[string]struct {
		src    []wasmvmtypes.Coin
		exp    sdk.Coins
		expErr bool
	}{
		"empty": {
			src: []wasmvmtypes.Coin{},
			exp: nil,
		},
		"single coin": {
			src: []wasmvmtypes.Coin{{Denom: "foo", Amount: "1"}},
			exp: sdk.NewCoins(sdk.NewCoin("foo", sdkmath.NewInt(1))),
		},
		"multiple coins": {
			src: []wasmvmtypes.Coin{
				{Denom: "foo", Amount: "1"},
				{Denom: "bar", Amount: "2"},
			},
			exp: sdk.NewCoins(
				sdk.NewCoin("bar", sdkmath.NewInt(2)),
				sdk.NewCoin("foo", sdkmath.NewInt(1)),
			),
		},
		"sorted": {
			src: []wasmvmtypes.Coin{
				{Denom: "foo", Amount: "1"},
				{Denom: "other", Amount: "1"},
				{Denom: "bar", Amount: "1"},
			},
			exp: []sdk.Coin{
				sdk.NewCoin("bar", sdkmath.NewInt(1)),
				sdk.NewCoin("foo", sdkmath.NewInt(1)),
				sdk.NewCoin("other", sdkmath.NewInt(1)),
			},
		},
		"zero amounts dropped": {
			src: []wasmvmtypes.Coin{
				{Denom: "foo", Amount: "1"},
				{Denom: "bar", Amount: "0"},
			},
			exp: sdk.NewCoins(
				sdk.NewCoin("foo", sdkmath.NewInt(1)),
			),
		},
		"duplicate denoms merged": {
			src: []wasmvmtypes.Coin{
				{Denom: "foo", Amount: "1"},
				{Denom: "foo", Amount: "1"},
			},
			exp: []sdk.Coin{sdk.NewCoin("foo", sdkmath.NewInt(2))},
		},
		"duplicate denoms with one 0 amount does not fail": {
			src: []wasmvmtypes.Coin{
				{Denom: "foo", Amount: "0"},
				{Denom: "foo", Amount: "1"},
			},
			exp: []sdk.Coin{sdk.NewCoin("foo", sdkmath.NewInt(1))},
		},
		"empty denom rejected": {
			src:    []wasmvmtypes.Coin{{Denom: "", Amount: "1"}},
			expErr: true,
		},
		"invalid denom rejected": {
			src:    []wasmvmtypes.Coin{{Denom: "!%&", Amount: "1"}},
			expErr: true,
		},
	}
	for name, spec := range specs {
		t.Run(name, func(t *testing.T) {
			gotCoins, gotErr := ConvertWasmCoinsToSdkCoins(spec.src)
			if spec.expErr {
				require.Error(t, gotErr)
				return
			}
			require.NoError(t, gotErr)
			assert.Equal(t, spec.exp, gotCoins)
			assert.NoError(t, gotCoins.Validate())
		})
	}
}
