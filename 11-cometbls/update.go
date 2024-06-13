package cometbls

import (
	"bytes"
	"crypto/sha256"
	"encoding/hex"
	"fmt"

	errorsmod "cosmossdk.io/errors"
	storetypes "cosmossdk.io/store/types"

	"github.com/cosmos/cosmos-sdk/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/holiman/uint256"

	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	commitmenttypes "github.com/cosmos/ibc-go/v8/modules/core/23-commitment/types"
	host "github.com/cosmos/ibc-go/v8/modules/core/24-host"
	"github.com/cosmos/ibc-go/v8/modules/core/exported"

	backend_opts "github.com/consensys/gnark/backend"
	backend "github.com/consensys/gnark/backend/groth16"
	backend_bn254 "github.com/consensys/gnark/backend/groth16/bn254"
	"github.com/consensys/gnark/frontend"

	"github.com/consensys/gnark-crypto/ecc"
	lcgadget "github.com/unionlabs/union/galoisd/pkg/lightclient/nonadjacent"

	cometbn254 "github.com/cometbft/cometbft/crypto/bn254"
	"github.com/consensys/gnark-crypto/ecc/bn254/fr"
)

var verifyingKey backend_bn254.VerifyingKey

func init() {
	vkHex := "8967072901cc7ab63357f1ddc4196c7c1feda50540d8026d7f6f0167c118a899d923def15f75234f2a6d53b566a2528441e98050b38803673e9179b834fc39a499355fd270b7601d5d88408b7e9e53d260512e2180cd260017dc941f2fc96d65153f0344c6bf2d8a891b979bc61d39a98fb11155fcd57418f30ea018ea842874a0e76be91a3148e2f8ef644222b3ce5b939a73bd2e0a40814f7f92a79c483acf2216bbe0c289e07936b4d9653b91521a24c570c808fa46dfd12ec4429e71b61999fcfb245459d63a4923b8f8c488d1e6af7ca358867b88eb0cdefe896c221f09e95e4c18d1e0475de4549b2547611d8301e1afff1047a6f5a288c9314af0b9fc05d403c8c91820a385a72c18d6a4962cef41a3ab93daa7ed289b1e95db4d04eb00000003e71843e52743864f4bb67ce94a2ce8fe82c8f61042c4c1ced8531d94305392818b0dbe71f4d60e02e9160ec2b015cae3a09cbe4f437226e2c02e1a5e5d124bcac29e93d5f47c0c7671350398ed8c40f5bc5c2f5b00363c7e2eb18a91a1c490c70000000100000000a57df6f8132cb0037f7dfdf1a29b04c1ff92ba082eda513996ba2bfa9fbd198713f0d8d8879885ca567ef99298c30c397e6fba584658f4127713a814c06de55aefbfe141a7555cf7e3e86b092660b81cfb68a025ad817e45cec0b0f2e2ca636802a104df1c015f2307fa2859627098cdf9fdb521d61d323943343a12304e5baf"
	vk, err := hex.DecodeString(vkHex)
	if err != nil {
		panic(fmt.Sprintf("could not decode the hex verifying key: '%s'", vkHex))
	}

	_, err = backend.VerifyingKey(&verifyingKey).ReadFrom(bytes.NewReader(vk))
	if err != nil {
		panic(fmt.Sprintf("could not read the verifying key: '%s'", vkHex))
	}
}

type cometblsHashToField struct {
	data []byte
}

func (c *cometblsHashToField) Write(p []byte) (n int, err error) {
	c.data = append(c.data, p...)
	return len(p), nil
}

func (c *cometblsHashToField) Sum(b []byte) []byte {
	e := cometbn254.HashToField(c.data)
	eB := e.Bytes()
	return append(b, eB[:]...)
}

func (c *cometblsHashToField) Reset() {
	c.data = []byte{}
}

func (c *cometblsHashToField) Size() int {
	return fr.Bytes
}

func (c *cometblsHashToField) BlockSize() int {
	return fr.Bytes
}

// VerifyClientMessage checks if the clientMessage is of type Header or Misbehaviour and verifies the message
func (cs *ClientState) VerifyClientMessage(
	ctx sdk.Context, cdc codec.BinaryCodec, clientStore storetypes.KVStore,
	clientMsg exported.ClientMessage,
) error {
	switch msg := clientMsg.(type) {
	case *Header:
		return cs.verifyHeader(ctx, clientStore, cdc, msg)
	case *Misbehaviour:
		return cs.verifyMisbehaviour(ctx, clientStore, cdc, msg)
	default:
		return clienttypes.ErrInvalidClientType
	}
}

// verifyHeader returns an error if:
// - the client or header provided are not parseable to tendermint types
// - the header is invalid
// - header height is less than or equal to the trusted header height
// - header revision is not equal to trusted header revision
// - header valset commit verification fails
// - header timestamp is past the trusting period in relation to the consensus state
// - header timestamp is less than or equal to the consensus state timestamp
func (cs *ClientState) verifyHeader(
	ctx sdk.Context, clientStore storetypes.KVStore, cdc codec.BinaryCodec,
	header *Header,
) error {
	// Retrieve trusted consensus states for each Header in misbehaviour
	consState, found := GetConsensusState(clientStore, cdc, header.TrustedHeight)
	if !found {
		return errorsmod.Wrapf(clienttypes.ErrConsensusStateNotFound, "could not get trusted consensus state from clientStore for Header at TrustedHeight: %s", header.TrustedHeight)
	}

	// UpdateClient only accepts updates with a header at the same revision
	// as the trusted consensus state
	if header.GetHeight().GetRevisionNumber() != header.TrustedHeight.RevisionNumber {
		return errorsmod.Wrapf(
			ErrInvalidHeaderHeight,
			"header height revision %d does not match trusted header revision %d",
			header.GetHeight().GetRevisionNumber(), header.TrustedHeight.RevisionNumber,
		)
	}

	if consState.GetTimestamp() > uint64(header.SignedHeader.Header.GetTime().UnixNano()) {
		return errorsmod.Wrapf(
			ErrInvalidHeaderTimestamp,
			"trusted header timestamp %d is greater than the new header timestamp %d",
			consState.GetTimestamp(), header.SignedHeader.Header.GetTime().UnixNano(),
		)
	}

	// assert header height is newer than consensus state
	if header.GetHeight().LTE(header.TrustedHeight) {
		return errorsmod.Wrapf(
			clienttypes.ErrInvalidHeader,
			"header height ≤ consensus state height (%s ≤ %s)", header.GetHeight(), header.TrustedHeight,
		)
	}

	if header.GetTime().UnixNano() >= ctx.BlockHeader().Time.UnixNano()+int64(cs.MaxClockDrift) {
		return errorsmod.Wrapf(
			clienttypes.ErrInvalidHeader,
			"header time >= max drift (%d >= currentTime + %d)", header.GetTime().UnixNano(), cs.MaxClockDrift,
		)
	}

	if header.SignedHeader.Header.Height == int64(header.TrustedHeight.RevisionHeight)+1 &&
		!bytes.Equal(header.SignedHeader.Header.ValidatorsHash, consState.NextValidatorsHash) {
		return errorsmod.Wrapf(
			clienttypes.ErrInvalidHeader,
			"the validators hash %s doesn't match the trusted validators hash %s for an adjacent block", header.SignedHeader.Header.ValidatorsHash, consState.NextValidatorsHash,
		)
	}

	// TODO(aeryz): verify zkp

	var proof backend_bn254.Proof
	_, err := proof.ReadFrom(bytes.NewReader(header.ZeroKnowledgeProof))
	if err != nil {
		panic("cannot load the proof")
	}

	hasher := sha256.New()
	var chainId [32]byte = [32]byte{}
	for i := 0; i < len(cs.ChainId); i++ {
		chainId[32-len(cs.ChainId)+i] = []byte(cs.ChainId)[i]
	}
	hasher.Write(chainId[:])
	hasher.Write(uint256.NewInt(uint64(header.SignedHeader.Header.Height)).Bytes())
	hasher.Write(uint256.NewInt(uint64(header.SignedHeader.Header.Time.Second())).Bytes())
	hasher.Write(uint256.NewInt(uint64(header.SignedHeader.Header.Time.Nanosecond())).Bytes())
	hasher.Write(header.SignedHeader.Header.ValidatorsHash)
	hasher.Write(header.SignedHeader.Header.NextValidatorsHash)
	hasher.Write(header.SignedHeader.Header.AppHash)
	hasher.Write(consState.NextValidatorsHash)
	inputsHash := hasher.Sum(nil)
	inputsHash[0] = 0

	witness := lcgadget.Circuit{
		InputsHash: inputsHash,
	}

	publicWitness, err := frontend.NewWitness(&witness, ecc.BN254.ScalarField(), frontend.PublicOnly())
	if err != nil {
		panic("invalid zkp")
	}

	err = backend.Verify(
		backend.Proof(&proof),
		backend.VerifyingKey(&verifyingKey),
		publicWitness,
		backend_opts.WithVerifierHashToFieldFunction(&cometblsHashToField{}),
	)

	return nil
}

// UpdateState may be used to either create a consensus state for:
// - a future height greater than the latest client state height
// - a past height that was skipped during bisection
// If we are updating to a past height, a consensus state is created for that height to be persisted in client store
// If we are updating to a future height, the consensus state is created and the client state is updated to reflect
// the new latest height
// A list containing the updated consensus height is returned.
// UpdateState must only be used to update within a single revision, thus header revision number and trusted height's revision
// number must be the same. To update to a new revision, use a separate upgrade path
// UpdateState will prune the oldest consensus state if it is expired.
// If the provided clientMsg is not of type of Header then the handler will noop and empty slice is returned.
func (cs ClientState) UpdateState(ctx sdk.Context, cdc codec.BinaryCodec, clientStore storetypes.KVStore, clientMsg exported.ClientMessage) []exported.Height {
	header, ok := clientMsg.(*Header)
	if !ok {
		// clientMsg is invalid Misbehaviour, no update necessary
		return []exported.Height{}
	}

	// performance: do not prune in checkTx
	// simulation must prune for accurate gas estimation
	if (!ctx.IsCheckTx() && !ctx.IsReCheckTx()) || ctx.ExecMode() == sdk.ExecModeSimulate {
		cs.pruneOldestConsensusState(ctx, cdc, clientStore)
	}

	// check for duplicate update
	if _, found := GetConsensusState(clientStore, cdc, header.GetHeight()); found {
		// perform no-op
		return []exported.Height{header.GetHeight()}
	}

	height := header.GetHeight().(clienttypes.Height)
	if height.GT(cs.LatestHeight) {
		cs.LatestHeight = height
	}

	consensusState := &ConsensusState{
		Timestamp:          uint64(header.GetTime().UnixNano()),
		Root:               commitmenttypes.NewMerkleRoot(header.SignedHeader.Header.GetAppHash()),
		NextValidatorsHash: header.SignedHeader.Header.NextValidatorsHash,
	}

	// set client state, consensus state and asssociated metadata
	setClientState(clientStore, cdc, &cs)
	setConsensusState(clientStore, cdc, consensusState, header.GetHeight())
	setConsensusMetadata(ctx, clientStore, header.GetHeight())

	return []exported.Height{height}
}

// pruneOldestConsensusState will retrieve the earliest consensus state for this clientID and check if it is expired. If it is,
// that consensus state will be pruned from store along with all associated metadata. This will prevent the client store from
// becoming bloated with expired consensus states that can no longer be used for updates and packet verification.
func (cs ClientState) pruneOldestConsensusState(ctx sdk.Context, cdc codec.BinaryCodec, clientStore storetypes.KVStore) {
	// Check the earliest consensus state to see if it is expired, if so then set the prune height
	// so that we can delete consensus state and all associated metadata.
	var (
		pruneHeight exported.Height
	)

	pruneCb := func(height exported.Height) bool {
		consState, found := GetConsensusState(clientStore, cdc, height)
		// this error should never occur
		if !found {
			panic(errorsmod.Wrapf(clienttypes.ErrConsensusStateNotFound, "failed to retrieve consensus state at height: %s", height))
		}

		if cs.IsExpired(consState.Timestamp, uint64(ctx.BlockTime().UnixNano())) {
			pruneHeight = height
		}

		return true
	}

	IterateConsensusStateAscending(clientStore, pruneCb)

	// if pruneHeight is set, delete consensus state and metadata
	if pruneHeight != nil {
		deleteConsensusState(clientStore, pruneHeight)
		deleteConsensusMetadata(clientStore, pruneHeight)
	}
}

// UpdateStateOnMisbehaviour updates state upon misbehaviour, freezing the ClientState. This method should only be called when misbehaviour is detected
// as it does not perform any misbehaviour checks.
func (cs ClientState) UpdateStateOnMisbehaviour(ctx sdk.Context, cdc codec.BinaryCodec, clientStore storetypes.KVStore, _ exported.ClientMessage) {
	cs.FrozenHeight = FrozenHeight

	clientStore.Set(host.ClientStateKey(), clienttypes.MustMarshalClientState(cdc, &cs))
}
