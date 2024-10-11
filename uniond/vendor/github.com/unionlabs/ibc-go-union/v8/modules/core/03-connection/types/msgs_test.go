package types_test

import (
	"fmt"
	"testing"

	dbm "github.com/cosmos/cosmos-db"
	"github.com/stretchr/testify/require"
	testifysuite "github.com/stretchr/testify/suite"

	log "cosmossdk.io/log"
	"cosmossdk.io/store/iavl"
	"cosmossdk.io/store/metrics"
	"cosmossdk.io/store/rootmulti"
	storetypes "cosmossdk.io/store/types"

	sdk "github.com/cosmos/cosmos-sdk/types"

	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	"github.com/cosmos/ibc-go/v8/modules/core/03-connection/types"
	commitmenttypes "github.com/cosmos/ibc-go/v8/modules/core/23-commitment/types"
	"github.com/cosmos/ibc-go/v8/modules/core/exported"
	ibctm "github.com/cosmos/ibc-go/v8/modules/light-clients/07-tendermint"
	ibctesting "github.com/cosmos/ibc-go/v8/testing"
	"github.com/cosmos/ibc-go/v8/testing/simapp"
)

var (
	signer = "cosmos1ckgw5d7jfj7wwxjzs9fdrdev9vc8dzcw3n2lht"

	emptyPrefix = commitmenttypes.MerklePrefix{}
	emptyProof  = []byte{}
)

type MsgTestSuite struct {
	testifysuite.Suite

	coordinator *ibctesting.Coordinator

	chainA *ibctesting.TestChain
	chainB *ibctesting.TestChain

	proof []byte
}

func (suite *MsgTestSuite) SetupTest() {
	suite.coordinator = ibctesting.NewCoordinator(suite.T(), 2)

	suite.chainA = suite.coordinator.GetChain(ibctesting.GetChainID(1))
	suite.chainB = suite.coordinator.GetChain(ibctesting.GetChainID(2))

	app := simapp.Setup(suite.T(), false)
	db := dbm.NewMemDB()
	store := rootmulti.NewStore(db, log.NewNopLogger(), metrics.NewNoOpMetrics())
	storeKey := storetypes.NewKVStoreKey("iavlStoreKey")

	store.MountStoreWithDB(storeKey, storetypes.StoreTypeIAVL, nil)
	err := store.LoadVersion(0)
	suite.Require().NoError(err)
	iavlStore := store.GetCommitStore(storeKey).(*iavl.Store)

	iavlStore.Set([]byte("KEY"), []byte("VALUE"))
	_ = store.Commit()

	res, err := store.Query(&storetypes.RequestQuery{
		Data:  []byte("KEY"),
		Path:  fmt.Sprintf("/%s/key", storeKey.Name()), // required path to get key/value+proof
		Prove: true,
	})
	suite.Require().NoError(err)

	merkleProof, err := commitmenttypes.ConvertProofs(res.ProofOps)
	suite.Require().NoError(err)
	proof, err := app.AppCodec().Marshal(&merkleProof)
	suite.Require().NoError(err)

	suite.proof = proof
}

func TestMsgTestSuite(t *testing.T) {
	testifysuite.Run(t, new(MsgTestSuite))
}

func (suite *MsgTestSuite) TestNewMsgConnectionOpenInit() {
	prefix := commitmenttypes.NewMerklePrefix([]byte("storePrefixKey"))
	// empty versions are considered valid, the default compatible versions
	// will be used in protocol.
	var version *types.Version

	testCases := []struct {
		name    string
		msg     *types.MsgConnectionOpenInit
		expPass bool
	}{
		{"localhost client ID", types.NewMsgConnectionOpenInit(exported.LocalhostClientID, "clienttotest", prefix, version, 500, signer), false},
		{"invalid client ID", types.NewMsgConnectionOpenInit("test/iris", "clienttotest", prefix, version, 500, signer), false},
		{"invalid counterparty client ID", types.NewMsgConnectionOpenInit("clienttotest", "(clienttotest)", prefix, version, 500, signer), false},
		{"invalid counterparty connection ID", &types.MsgConnectionOpenInit{connectionID, types.NewCounterparty("clienttotest", "connectiontotest", prefix), version, 500, signer}, false},
		{"empty counterparty prefix", types.NewMsgConnectionOpenInit("clienttotest", "clienttotest", emptyPrefix, version, 500, signer), false},
		{"supplied version fails basic validation", types.NewMsgConnectionOpenInit("clienttotest", "clienttotest", prefix, &types.Version{}, 500, signer), false},
		{"empty singer", types.NewMsgConnectionOpenInit("clienttotest", "clienttotest", prefix, version, 500, ""), false},
		{"success", types.NewMsgConnectionOpenInit("clienttotest", "clienttotest", prefix, version, 500, signer), true},
	}

	for _, tc := range testCases {
		tc := tc

		err := tc.msg.ValidateBasic()
		if tc.expPass {
			suite.Require().NoError(err, tc.name)
		} else {
			suite.Require().Error(err, tc.name)
		}
	}
}

func (suite *MsgTestSuite) TestNewMsgConnectionOpenTry() {
	prefix := commitmenttypes.NewMerklePrefix([]byte("storePrefixKey"))

	clientState := ibctm.NewClientState(
		chainID, ibctm.DefaultTrustLevel, ibctesting.TrustingPeriod, ibctesting.UnbondingPeriod, ibctesting.MaxClockDrift, clientHeight, commitmenttypes.GetSDKSpecs(), ibctesting.UpgradePath,
	)
	protoAny, err := clienttypes.PackClientState(clientState)
	suite.Require().NoError(err)

	counterparty := types.NewCounterparty("connectiontotest", "clienttotest", prefix)

	testCases := []struct {
		name    string
		msg     *types.MsgConnectionOpenTry
		expPass bool
	}{
		{"non empty connection ID", &types.MsgConnectionOpenTry{"connection-0", "clienttotesta", protoAny, counterparty, 500, []*types.Version{ibctesting.ConnectionVersion}, clientHeight, suite.proof, suite.proof, suite.proof, clientHeight, signer, nil}, false},
		{"localhost client ID", types.NewMsgConnectionOpenTry(exported.LocalhostClientID, "connectiontotest", "clienttotest", clientState, prefix, []*types.Version{ibctesting.ConnectionVersion}, 500, suite.proof, suite.proof, suite.proof, clientHeight, clientHeight, signer), false},
		{"invalid client ID", types.NewMsgConnectionOpenTry("test/iris", "connectiontotest", "clienttotest", clientState, prefix, []*types.Version{ibctesting.ConnectionVersion}, 500, suite.proof, suite.proof, suite.proof, clientHeight, clientHeight, signer), false},
		{"invalid counterparty connection ID", types.NewMsgConnectionOpenTry("clienttotesta", "ibc/test", "clienttotest", clientState, prefix, []*types.Version{ibctesting.ConnectionVersion}, 500, suite.proof, suite.proof, suite.proof, clientHeight, clientHeight, signer), false},
		{"invalid counterparty client ID", types.NewMsgConnectionOpenTry("clienttotesta", "connectiontotest", "test/conn1", clientState, prefix, []*types.Version{ibctesting.ConnectionVersion}, 500, suite.proof, suite.proof, suite.proof, clientHeight, clientHeight, signer), false},
		{"empty counterparty prefix", types.NewMsgConnectionOpenTry("clienttotesta", "connectiontotest", "clienttotest", clientState, emptyPrefix, []*types.Version{ibctesting.ConnectionVersion}, 500, suite.proof, suite.proof, suite.proof, clientHeight, clientHeight, signer), false},
		{"empty counterpartyVersions", types.NewMsgConnectionOpenTry("clienttotesta", "connectiontotest", "clienttotest", clientState, prefix, []*types.Version{}, 500, suite.proof, suite.proof, suite.proof, clientHeight, clientHeight, signer), false},
		{"empty proofInit", types.NewMsgConnectionOpenTry("clienttotesta", "connectiontotest", "clienttotest", clientState, prefix, []*types.Version{ibctesting.ConnectionVersion}, 500, emptyProof, suite.proof, suite.proof, clientHeight, clientHeight, signer), false},
		{"empty singer", types.NewMsgConnectionOpenTry("clienttotesta", "connectiontotest", "clienttotest", clientState, prefix, []*types.Version{ibctesting.ConnectionVersion}, 500, suite.proof, suite.proof, suite.proof, clientHeight, clientHeight, ""), false},
		{"success", types.NewMsgConnectionOpenTry("clienttotesta", "connectiontotest", "clienttotest", clientState, prefix, []*types.Version{ibctesting.ConnectionVersion}, 500, suite.proof, suite.proof, suite.proof, clientHeight, clientHeight, signer), true},
		{"invalid version", types.NewMsgConnectionOpenTry("clienttotesta", "connectiontotest", "clienttotest", clientState, prefix, []*types.Version{{}}, 500, suite.proof, suite.proof, suite.proof, clientHeight, clientHeight, signer), false},
	}

	for _, tc := range testCases {
		tc := tc

		err := tc.msg.ValidateBasic()
		if tc.expPass {
			suite.Require().NoError(err, tc.name)
		} else {
			suite.Require().Error(err, tc.name)
		}
	}
}

func (suite *MsgTestSuite) TestNewMsgConnectionOpenAck() {
	clientState := ibctm.NewClientState(
		chainID, ibctm.DefaultTrustLevel, ibctesting.TrustingPeriod, ibctesting.UnbondingPeriod, ibctesting.MaxClockDrift, clientHeight, commitmenttypes.GetSDKSpecs(), ibctesting.UpgradePath,
	)

	connectionID := "connection-0"

	testCases := []struct {
		name    string
		msg     *types.MsgConnectionOpenAck
		expPass bool
	}{
		{"invalid connection ID", types.NewMsgConnectionOpenAck("test/conn1", connectionID, clientState, suite.proof, suite.proof, suite.proof, clientHeight, clientHeight, ibctesting.ConnectionVersion, signer), false},
		{"invalid counterparty connection ID", types.NewMsgConnectionOpenAck(connectionID, "test/conn1", clientState, suite.proof, suite.proof, suite.proof, clientHeight, clientHeight, ibctesting.ConnectionVersion, signer), false},
		{"empty proofTry", types.NewMsgConnectionOpenAck(connectionID, connectionID, clientState, emptyProof, suite.proof, suite.proof, clientHeight, clientHeight, ibctesting.ConnectionVersion, signer), false},
		{"invalid version", types.NewMsgConnectionOpenAck(connectionID, connectionID, clientState, suite.proof, suite.proof, suite.proof, clientHeight, clientHeight, &types.Version{}, signer), false},
		{"empty signer", types.NewMsgConnectionOpenAck(connectionID, connectionID, clientState, suite.proof, suite.proof, suite.proof, clientHeight, clientHeight, ibctesting.ConnectionVersion, ""), false},
		{"success", types.NewMsgConnectionOpenAck(connectionID, connectionID, clientState, suite.proof, suite.proof, suite.proof, clientHeight, clientHeight, ibctesting.ConnectionVersion, signer), true},
	}

	for _, tc := range testCases {
		tc := tc

		err := tc.msg.ValidateBasic()
		if tc.expPass {
			suite.Require().NoError(err, tc.name)
		} else {
			suite.Require().Error(err, tc.name)
		}
	}
}

func (suite *MsgTestSuite) TestNewMsgConnectionOpenConfirm() {
	testCases := []struct {
		name    string
		msg     *types.MsgConnectionOpenConfirm
		expPass bool
	}{
		{"invalid connection ID", types.NewMsgConnectionOpenConfirm("test/conn1", suite.proof, clientHeight, signer), false},
		{"empty proofTry", types.NewMsgConnectionOpenConfirm(connectionID, emptyProof, clientHeight, signer), false},
		{"empty signer", types.NewMsgConnectionOpenConfirm(connectionID, suite.proof, clientHeight, ""), false},
		{"success", types.NewMsgConnectionOpenConfirm(connectionID, suite.proof, clientHeight, signer), true},
	}

	for _, tc := range testCases {
		tc := tc

		err := tc.msg.ValidateBasic()
		if tc.expPass {
			suite.Require().NoError(err, tc.name)
		} else {
			suite.Require().Error(err, tc.name)
		}
	}
}

// TestMsgUpdateParamsValidateBasic tests ValidateBasic for MsgUpdateParams
func (suite *MsgTestSuite) TestMsgUpdateParamsValidateBasic() {
	signer := suite.chainA.App.GetIBCKeeper().GetAuthority()
	testCases := []struct {
		name    string
		msg     *types.MsgUpdateParams
		expPass bool
	}{
		{
			"success: valid signer and params",
			types.NewMsgUpdateParams(signer, types.DefaultParams()),
			true,
		},
		{
			"failure: invalid signer address",
			types.NewMsgUpdateParams("invalid", types.DefaultParams()),
			false,
		},
		{
			"failure: invalid time per block",
			types.NewMsgUpdateParams(signer, types.NewParams(0)),
			false,
		},
	}

	for _, tc := range testCases {
		tc := tc

		err := tc.msg.ValidateBasic()
		if tc.expPass {
			suite.Require().NoError(err, "valid case %s failed", tc.name)
		} else {
			suite.Require().Error(err, "invalid case %s passed", tc.name)
		}
	}
}

// TestMsgUpdateParamsGetSigners tests GetSigners for MsgUpdateParams
func TestMsgUpdateParamsGetSigners(t *testing.T) {
	testCases := []struct {
		name    string
		address sdk.AccAddress
		expPass bool
	}{
		{"success: valid address", sdk.AccAddress(ibctesting.TestAccAddress), true},
		{"failure: nil address", nil, false},
	}

	for _, tc := range testCases {
		tc := tc

		msg := types.MsgUpdateParams{
			Signer: tc.address.String(),
			Params: types.DefaultParams(),
		}
		if tc.expPass {
			require.Equal(t, []sdk.AccAddress{tc.address}, msg.GetSigners())
		} else {
			require.Panics(t, func() {
				msg.GetSigners()
			})
		}
	}
}
