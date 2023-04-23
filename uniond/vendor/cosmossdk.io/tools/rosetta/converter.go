package rosetta

import (
	"bytes"
	"encoding/json"
	"fmt"
	"reflect"

	"cosmossdk.io/math"
	"github.com/btcsuite/btcd/btcec/v2"
	rosettatypes "github.com/coinbase/rosetta-sdk-go/types"
	abci "github.com/cometbft/cometbft/abci/types"
	"github.com/cometbft/cometbft/crypto"
	tmcoretypes "github.com/cometbft/cometbft/rpc/core/types"
	tmtypes "github.com/cometbft/cometbft/types"

	crgerrs "cosmossdk.io/tools/rosetta/lib/errors"
	crgtypes "cosmossdk.io/tools/rosetta/lib/types"
	sdkclient "github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/codec"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	"github.com/cosmos/cosmos-sdk/crypto/keys/secp256k1"
	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/tx/signing"
	authsigning "github.com/cosmos/cosmos-sdk/x/auth/signing"
	auth "github.com/cosmos/cosmos-sdk/x/auth/types"
	banktypes "github.com/cosmos/cosmos-sdk/x/bank/types"
)

// Converter is a utility that can be used to convert
// back and forth from rosetta to sdk and tendermint types
// IMPORTANT NOTES:
//   - IT SHOULD BE USED ONLY TO DEAL WITH THINGS
//     IN A STATELESS WAY! IT SHOULD NEVER INTERACT DIRECTLY
//     WITH TENDERMINT RPC AND COSMOS GRPC
//
// - IT SHOULD RETURN cosmos rosetta gateway error types!
type Converter interface {
	// ToSDK exposes the methods that convert
	// rosetta types to cosmos sdk and tendermint types
	ToSDK() ToSDKConverter
	// ToRosetta exposes the methods that convert
	// sdk and tendermint types to rosetta types
	ToRosetta() ToRosettaConverter
}

// ToRosettaConverter is an interface that exposes
// all the functions used to convert sdk and
// tendermint types to rosetta known types
type ToRosettaConverter interface {
	// BlockResponse returns a block response given a result block
	BlockResponse(block *tmcoretypes.ResultBlock) crgtypes.BlockResponse
	// BeginBlockToTx converts the given begin block hash to rosetta transaction hash
	BeginBlockTxHash(blockHash []byte) string
	// EndBlockTxHash converts the given endblock hash to rosetta transaction hash
	EndBlockTxHash(blockHash []byte) string
	// Amounts converts sdk.Coins to rosetta.Amounts
	Amounts(ownedCoins []sdk.Coin, availableCoins sdk.Coins) []*rosettatypes.Amount
	// Ops converts an sdk.Msg to rosetta operations
	Ops(status string, msg sdk.Msg) ([]*rosettatypes.Operation, error)
	// OpsAndSigners takes raw transaction bytes and returns rosetta operations and the expected signers
	OpsAndSigners(txBytes []byte) (ops []*rosettatypes.Operation, signers []*rosettatypes.AccountIdentifier, err error)
	// Meta converts an sdk.Msg to rosetta metadata
	Meta(msg sdk.Msg) (meta map[string]interface{}, err error)
	// SignerData returns account signing data from a queried any account
	SignerData(anyAccount *codectypes.Any) (*SignerData, error)
	// SigningComponents returns rosetta's components required to build a signable transaction
	SigningComponents(tx authsigning.Tx, metadata *ConstructionMetadata, rosPubKeys []*rosettatypes.PublicKey) (txBytes []byte, payloadsToSign []*rosettatypes.SigningPayload, err error)
	// Tx converts a tendermint transaction and tx result if provided to a rosetta tx
	Tx(rawTx tmtypes.Tx, txResult *abci.ResponseDeliverTx) (*rosettatypes.Transaction, error)
	// TxIdentifiers converts a tendermint tx to transaction identifiers
	TxIdentifiers(txs []tmtypes.Tx) []*rosettatypes.TransactionIdentifier
	// BalanceOps converts events to balance operations
	BalanceOps(status string, events []abci.Event) []*rosettatypes.Operation
	// SyncStatus converts a tendermint status to sync status
	SyncStatus(status *tmcoretypes.ResultStatus) *rosettatypes.SyncStatus
	// Peers converts tendermint peers to rosetta
	Peers(peers []tmcoretypes.Peer) []*rosettatypes.Peer
}

// ToSDKConverter is an interface that exposes
// all the functions used to convert rosetta types
// to tendermint and sdk types
type ToSDKConverter interface {
	// UnsignedTx converts rosetta operations to an unsigned cosmos sdk transactions
	UnsignedTx(ops []*rosettatypes.Operation) (tx authsigning.Tx, err error)
	// SignedTx adds the provided signatures after decoding the unsigned transaction raw bytes
	// and returns the signed tx bytes
	SignedTx(txBytes []byte, signatures []*rosettatypes.Signature) (signedTxBytes []byte, err error)
	// Msg converts metadata to an sdk message
	Msg(meta map[string]interface{}, msg sdk.Msg) (err error)
	// HashToTxType returns the transaction type (end block, begin block or deliver tx)
	// and the real hash to query in order to get information
	HashToTxType(hashBytes []byte) (txType TransactionType, realHash []byte)
	// PubKey attempts to convert a rosetta public key to cosmos sdk one
	PubKey(pk *rosettatypes.PublicKey) (cryptotypes.PubKey, error)
}

type converter struct {
	newTxBuilder    func() sdkclient.TxBuilder
	txBuilderFromTx func(tx sdk.Tx) (sdkclient.TxBuilder, error)
	txDecode        sdk.TxDecoder
	txEncode        sdk.TxEncoder
	bytesToSign     func(tx authsigning.Tx, signerData authsigning.SignerData) (b []byte, err error)
	ir              codectypes.InterfaceRegistry
	cdc             *codec.ProtoCodec
}

func NewConverter(cdc *codec.ProtoCodec, ir codectypes.InterfaceRegistry, cfg sdkclient.TxConfig) Converter {
	return converter{
		newTxBuilder:    cfg.NewTxBuilder,
		txBuilderFromTx: cfg.WrapTxBuilder,
		txDecode:        cfg.TxDecoder(),
		txEncode:        cfg.TxEncoder(),
		bytesToSign: func(tx authsigning.Tx, signerData authsigning.SignerData) (b []byte, err error) {
			bytesToSign, err := cfg.SignModeHandler().GetSignBytes(signing.SignMode_SIGN_MODE_LEGACY_AMINO_JSON, signerData, tx)
			if err != nil {
				return nil, err
			}

			return crypto.Sha256(bytesToSign), nil
		},
		ir:  ir,
		cdc: cdc,
	}
}

func (c converter) ToSDK() ToSDKConverter {
	return c
}

func (c converter) ToRosetta() ToRosettaConverter {
	return c
}

// OpsToUnsignedTx returns all the sdk.Msgs given the operations
func (c converter) UnsignedTx(ops []*rosettatypes.Operation) (tx authsigning.Tx, err error) {
	builder := c.newTxBuilder()

	var msgs []sdk.Msg

	for i := 0; i < len(ops); i++ {
		op := ops[i]

		protoMessage, err := c.ir.Resolve(op.Type)
		if err != nil {
			return nil, crgerrs.WrapError(crgerrs.ErrBadArgument, "operation not found: "+op.Type)
		}

		msg, ok := protoMessage.(sdk.Msg)
		if !ok {
			return nil, crgerrs.WrapError(crgerrs.ErrBadArgument, "operation is not a valid supported sdk.Msg: "+op.Type)
		}

		err = c.Msg(op.Metadata, msg)
		if err != nil {
			return nil, crgerrs.WrapError(crgerrs.ErrCodec, err.Error())
		}

		// verify message correctness
		if err = msg.ValidateBasic(); err != nil {
			return nil, crgerrs.WrapError(
				crgerrs.ErrBadArgument,
				fmt.Sprintf("validation of operation at index %d failed: %s", op.OperationIdentifier.Index, err),
			)
		}
		signers := msg.GetSigners()
		// check if there are enough signers
		if len(signers) == 0 {
			return nil, crgerrs.WrapError(crgerrs.ErrBadArgument, fmt.Sprintf("operation at index %d got no signers", op.OperationIdentifier.Index))
		}
		// append the msg
		msgs = append(msgs, msg)
		// if there's only one signer then simply continue
		if len(signers) == 1 {
			continue
		}
		// after we have got the msg, we need to verify if the message has multiple signers
		// if it has got multiple signers, then we need to fetch all the related operations
		// which involve the other signers of the msg, we expect to find them in order
		// so if the msg is named "v1.test.Send" and it expects 3 signers, the next 3 operations
		// must be with the same name "v1.test.Send" and contain the other signers
		// then we can just skip their processing
		for j := 0; j < len(signers)-1; j++ {
			skipOp := ops[i+j] // get the next index
			// verify that the operation is equal to the new one
			if skipOp.Type != op.Type {
				return nil, crgerrs.WrapError(
					crgerrs.ErrBadArgument,
					fmt.Sprintf("operation at index %d should have had type %s got: %s", i+j, op.Type, skipOp.Type),
				)
			}

			if !reflect.DeepEqual(op.Metadata, skipOp.Metadata) {
				return nil, crgerrs.WrapError(
					crgerrs.ErrBadArgument,
					fmt.Sprintf("operation at index %d should have had metadata equal to %#v, got: %#v", i+j, op.Metadata, skipOp.Metadata))
			}

			i++ // increase so we skip it
		}
	}

	if err := builder.SetMsgs(msgs...); err != nil {
		return nil, crgerrs.WrapError(crgerrs.ErrBadArgument, err.Error())
	}

	return builder.GetTx(), nil
}

// Msg unmarshals the rosetta metadata to the given sdk.Msg
func (c converter) Msg(meta map[string]interface{}, msg sdk.Msg) error {
	metaBytes, err := json.Marshal(meta)
	if err != nil {
		return err
	}
	return c.cdc.UnmarshalJSON(metaBytes, msg)
}

func (c converter) Meta(msg sdk.Msg) (meta map[string]interface{}, err error) {
	b, err := c.cdc.MarshalJSON(msg)
	if err != nil {
		return nil, crgerrs.WrapError(crgerrs.ErrCodec, err.Error())
	}

	err = json.Unmarshal(b, &meta)
	if err != nil {
		return nil, crgerrs.WrapError(crgerrs.ErrCodec, err.Error())
	}

	return
}

// Ops will create an operation for each msg signer
// with the message proto name as type, and the raw fields
// as metadata
func (c converter) Ops(status string, msg sdk.Msg) ([]*rosettatypes.Operation, error) {
	opName := sdk.MsgTypeURL(msg)

	meta, err := c.Meta(msg)
	if err != nil {
		return nil, err
	}

	ops := make([]*rosettatypes.Operation, len(msg.GetSigners()))
	for i, signer := range msg.GetSigners() {
		op := &rosettatypes.Operation{
			Type:     opName,
			Status:   &status,
			Account:  &rosettatypes.AccountIdentifier{Address: signer.String()},
			Metadata: meta,
		}

		ops[i] = op
	}

	return ops, nil
}

// Tx converts a tendermint raw transaction and its result (if provided) to a rosetta transaction
func (c converter) Tx(rawTx tmtypes.Tx, txResult *abci.ResponseDeliverTx) (*rosettatypes.Transaction, error) {
	// decode tx
	tx, err := c.txDecode(rawTx)
	if err != nil {
		return nil, crgerrs.WrapError(crgerrs.ErrCodec, err.Error())
	}
	// get initial status, as per sdk design, if one msg fails
	// the whole TX will be considered failing, so we can't have
	// 1 msg being success and 1 msg being reverted
	status := StatusTxSuccess
	switch txResult {
	// if nil, we're probably checking an unconfirmed tx
	// or trying to build a new transaction, so status
	// is not put inside
	case nil:
		status = ""
	// set the status
	default:
		if txResult.Code != abci.CodeTypeOK {
			status = StatusTxReverted
		}
	}
	// get operations from msgs
	msgs := tx.GetMsgs()
	var rawTxOps []*rosettatypes.Operation

	for _, msg := range msgs {
		ops, err := c.Ops(status, msg)
		if err != nil {
			return nil, err
		}
		rawTxOps = append(rawTxOps, ops...)
	}

	// now get balance events from response deliver tx
	var balanceOps []*rosettatypes.Operation
	// tx result might be nil, in case we're querying an unconfirmed tx from the mempool
	if txResult != nil {
		balanceOps = c.BalanceOps(StatusTxSuccess, txResult.Events) // force set to success because no events for failed tx
	}

	// now normalize indexes
	totalOps := AddOperationIndexes(rawTxOps, balanceOps)

	return &rosettatypes.Transaction{
		TransactionIdentifier: &rosettatypes.TransactionIdentifier{Hash: fmt.Sprintf("%X", rawTx.Hash())},
		Operations:            totalOps,
	}, nil
}

func (c converter) BalanceOps(status string, events []abci.Event) []*rosettatypes.Operation {
	var ops []*rosettatypes.Operation

	for _, e := range events {
		balanceOps, ok := sdkEventToBalanceOperations(status, e)
		if !ok {
			continue
		}
		ops = append(ops, balanceOps...)
	}

	return ops
}

// sdkEventToBalanceOperations converts an event to a rosetta balance operation
// it will panic if the event is malformed because it might mean the sdk spec
// has changed and rosetta needs to reflect those changes too.
// The balance operations are multiple, one for each denom.
func sdkEventToBalanceOperations(status string, event abci.Event) (operations []*rosettatypes.Operation, isBalanceEvent bool) {
	var (
		accountIdentifier string
		coinChange        sdk.Coins
		isSub             bool
	)

	switch event.Type {
	default:
		return nil, false
	case banktypes.EventTypeCoinSpent:
		spender := sdk.MustAccAddressFromBech32(string(event.Attributes[0].Value))
		coins, err := sdk.ParseCoinsNormalized(string(event.Attributes[1].Value))
		if err != nil {
			panic(err)
		}

		isSub = true
		coinChange = coins
		accountIdentifier = spender.String()

	case banktypes.EventTypeCoinReceived:
		receiver := sdk.MustAccAddressFromBech32(string(event.Attributes[0].Value))
		coins, err := sdk.ParseCoinsNormalized(string(event.Attributes[1].Value))
		if err != nil {
			panic(err)
		}

		isSub = false
		coinChange = coins
		accountIdentifier = receiver.String()

	// rosetta does not have the concept of burning coins, so we need to mock
	// the burn as a send to an address that cannot be resolved to anything
	case banktypes.EventTypeCoinBurn:
		coins, err := sdk.ParseCoinsNormalized(string(event.Attributes[1].Value))
		if err != nil {
			panic(err)
		}

		coinChange = coins
		accountIdentifier = BurnerAddressIdentifier
	}

	operations = make([]*rosettatypes.Operation, len(coinChange))

	for i, coin := range coinChange {

		value := coin.Amount.String()
		// in case the event is a subtract balance one the rewrite value with
		// the negative coin identifier
		if isSub {
			value = "-" + value
		}

		op := &rosettatypes.Operation{
			Type:    event.Type,
			Status:  &status,
			Account: &rosettatypes.AccountIdentifier{Address: accountIdentifier},
			Amount: &rosettatypes.Amount{
				Value: value,
				Currency: &rosettatypes.Currency{
					Symbol:   coin.Denom,
					Decimals: 0,
				},
			},
		}

		operations[i] = op
	}
	return operations, true
}

// Amounts converts []sdk.Coin to rosetta amounts
func (c converter) Amounts(ownedCoins []sdk.Coin, availableCoins sdk.Coins) []*rosettatypes.Amount {
	amounts := make([]*rosettatypes.Amount, len(availableCoins))
	ownedCoinsMap := make(map[string]math.Int, len(availableCoins))

	for _, ownedCoin := range ownedCoins {
		ownedCoinsMap[ownedCoin.Denom] = ownedCoin.Amount
	}

	for i, coin := range availableCoins {
		value, owned := ownedCoinsMap[coin.Denom]
		if !owned {
			amounts[i] = &rosettatypes.Amount{
				Value: sdk.NewInt(0).String(),
				Currency: &rosettatypes.Currency{
					Symbol: coin.Denom,
				},
			}
			continue
		}
		amounts[i] = &rosettatypes.Amount{
			Value: value.String(),
			Currency: &rosettatypes.Currency{
				Symbol: coin.Denom,
			},
		}
	}

	return amounts
}

// AddOperationIndexes adds the indexes to operations adhering to specific rules:
// operations related to messages will be always before than the balance ones
func AddOperationIndexes(msgOps []*rosettatypes.Operation, balanceOps []*rosettatypes.Operation) (finalOps []*rosettatypes.Operation) {
	lenMsgOps := len(msgOps)
	lenBalanceOps := len(balanceOps)
	finalOps = make([]*rosettatypes.Operation, 0, lenMsgOps+lenBalanceOps)

	var currentIndex int64
	// add indexes to msg ops
	for _, op := range msgOps {
		op.OperationIdentifier = &rosettatypes.OperationIdentifier{
			Index: currentIndex,
		}

		finalOps = append(finalOps, op)
		currentIndex++
	}

	// add indexes to balance ops
	for _, op := range balanceOps {
		op.OperationIdentifier = &rosettatypes.OperationIdentifier{
			Index: currentIndex,
		}

		finalOps = append(finalOps, op)
		currentIndex++
	}

	return finalOps
}

// EndBlockTxHash produces a mock endblock hash that rosetta can query
// for endblock operations, it also serves the purpose of representing
// part of the state changes happening at endblock level (balance ones)
func (c converter) EndBlockTxHash(hash []byte) string {
	final := append([]byte{EndBlockHashStart}, hash...)
	return fmt.Sprintf("%X", final)
}

// BeginBlockTxHash produces a mock beginblock hash that rosetta can query
// for beginblock operations, it also serves the purpose of representing
// part of the state changes happening at beginblock level (balance ones)
func (c converter) BeginBlockTxHash(hash []byte) string {
	final := append([]byte{BeginBlockHashStart}, hash...)
	return fmt.Sprintf("%X", final)
}

// HashToTxType takes the provided hash bytes from rosetta and discerns if they are
// a deliver tx type or endblock/begin block hash, returning the real hash afterwards
func (c converter) HashToTxType(hashBytes []byte) (txType TransactionType, realHash []byte) {
	switch len(hashBytes) {
	case DeliverTxSize:
		return DeliverTxTx, hashBytes

	case BeginEndBlockTxSize:
		switch hashBytes[0] {
		case BeginBlockHashStart:
			return BeginBlockTx, hashBytes[1:]
		case EndBlockHashStart:
			return EndBlockTx, hashBytes[1:]
		default:
			return UnrecognizedTx, nil
		}

	default:
		return UnrecognizedTx, nil
	}
}

// StatusToSyncStatus converts a tendermint status to rosetta sync status
func (c converter) SyncStatus(status *tmcoretypes.ResultStatus) *rosettatypes.SyncStatus {
	// determine sync status
	stage := StatusPeerSynced
	if status.SyncInfo.CatchingUp {
		stage = StatusPeerSyncing
	}

	return &rosettatypes.SyncStatus{
		CurrentIndex: &status.SyncInfo.LatestBlockHeight,
		TargetIndex:  nil, // sync info does not allow us to get target height
		Stage:        &stage,
	}
}

// TxIdentifiers converts a tendermint raw transactions into an array of rosetta tx identifiers
func (c converter) TxIdentifiers(txs []tmtypes.Tx) []*rosettatypes.TransactionIdentifier {
	converted := make([]*rosettatypes.TransactionIdentifier, len(txs))
	for i, tx := range txs {
		converted[i] = &rosettatypes.TransactionIdentifier{Hash: fmt.Sprintf("%X", tx.Hash())}
	}

	return converted
}

// tmResultBlockToRosettaBlockResponse converts a tendermint result block to block response
func (c converter) BlockResponse(block *tmcoretypes.ResultBlock) crgtypes.BlockResponse {
	var parentBlock *rosettatypes.BlockIdentifier

	switch block.Block.Height {
	case 1:
		parentBlock = &rosettatypes.BlockIdentifier{
			Index: 1,
			Hash:  fmt.Sprintf("%X", block.BlockID.Hash.Bytes()),
		}
	default:
		parentBlock = &rosettatypes.BlockIdentifier{
			Index: block.Block.Height - 1,
			Hash:  fmt.Sprintf("%X", block.Block.LastBlockID.Hash.Bytes()),
		}
	}
	return crgtypes.BlockResponse{
		Block: &rosettatypes.BlockIdentifier{
			Index: block.Block.Height,
			Hash:  block.Block.Hash().String(),
		},
		ParentBlock:          parentBlock,
		MillisecondTimestamp: timeToMilliseconds(block.Block.Time),
		TxCount:              int64(len(block.Block.Txs)),
	}
}

// Peers converts tm peers to rosetta peers
func (c converter) Peers(peers []tmcoretypes.Peer) []*rosettatypes.Peer {
	converted := make([]*rosettatypes.Peer, len(peers))

	for i, peer := range peers {
		converted[i] = &rosettatypes.Peer{
			PeerID: peer.NodeInfo.Moniker,
			Metadata: map[string]interface{}{
				"addr": peer.NodeInfo.ListenAddr,
			},
		}
	}

	return converted
}

// OpsAndSigners takes transactions bytes and returns the operation, is signed is true it will return
// the account identifiers which have signed the transaction
func (c converter) OpsAndSigners(txBytes []byte) (ops []*rosettatypes.Operation, signers []*rosettatypes.AccountIdentifier, err error) {
	rosTx, err := c.ToRosetta().Tx(txBytes, nil)
	if err != nil {
		return nil, nil, err
	}
	ops = rosTx.Operations

	// get the signers
	sdkTx, err := c.txDecode(txBytes)
	if err != nil {
		return nil, nil, crgerrs.WrapError(crgerrs.ErrCodec, err.Error())
	}

	txBuilder, err := c.txBuilderFromTx(sdkTx)
	if err != nil {
		return nil, nil, crgerrs.WrapError(crgerrs.ErrCodec, err.Error())
	}

	for _, signer := range txBuilder.GetTx().GetSigners() {
		signers = append(signers, &rosettatypes.AccountIdentifier{
			Address: signer.String(),
		})
	}

	return
}

func (c converter) SignedTx(txBytes []byte, signatures []*rosettatypes.Signature) (signedTxBytes []byte, err error) {
	rawTx, err := c.txDecode(txBytes)
	if err != nil {
		return nil, err
	}

	txBuilder, err := c.txBuilderFromTx(rawTx)
	if err != nil {
		return nil, err
	}

	notSignedSigs, err := txBuilder.GetTx().GetSignaturesV2() //
	if err != nil {
		return nil, crgerrs.WrapError(crgerrs.ErrCodec, err.Error())
	}

	if len(notSignedSigs) != len(signatures) {
		return nil, crgerrs.WrapError(
			crgerrs.ErrInvalidTransaction,
			fmt.Sprintf("expected transaction to have signers data matching the provided signatures: %d <-> %d", len(notSignedSigs), len(signatures)))
	}

	signedSigs := make([]signing.SignatureV2, len(notSignedSigs))
	for i, signature := range signatures {
		// TODO(fdymylja): here we should check that the public key matches...
		signedSigs[i] = signing.SignatureV2{
			PubKey: notSignedSigs[i].PubKey,
			Data: &signing.SingleSignatureData{
				SignMode:  signing.SignMode_SIGN_MODE_LEGACY_AMINO_JSON,
				Signature: signature.Bytes,
			},
			Sequence: notSignedSigs[i].Sequence,
		}
	}

	if err = txBuilder.SetSignatures(signedSigs...); err != nil {
		return nil, err
	}

	txBytes, err = c.txEncode(txBuilder.GetTx())
	if err != nil {
		return nil, err
	}

	return txBytes, nil
}

func (c converter) PubKey(pubKey *rosettatypes.PublicKey) (cryptotypes.PubKey, error) {
	if pubKey.CurveType != "secp256k1" {
		return nil, crgerrs.WrapError(crgerrs.ErrUnsupportedCurve, "only secp256k1 supported")
	}

	cmp, err := btcec.ParsePubKey(pubKey.Bytes)
	if err != nil {
		return nil, crgerrs.WrapError(crgerrs.ErrBadArgument, err.Error())
	}

	compressedPublicKey := make([]byte, secp256k1.PubKeySize)
	copy(compressedPublicKey, cmp.SerializeCompressed())

	pk := &secp256k1.PubKey{Key: compressedPublicKey}

	return pk, nil
}

// SigningComponents takes a sdk tx and construction metadata and returns signable components
func (c converter) SigningComponents(tx authsigning.Tx, metadata *ConstructionMetadata, rosPubKeys []*rosettatypes.PublicKey) (txBytes []byte, payloadsToSign []*rosettatypes.SigningPayload, err error) {
	// verify metadata correctness
	feeAmount, err := sdk.ParseCoinsNormalized(metadata.GasPrice)
	if err != nil {
		return nil, nil, crgerrs.WrapError(crgerrs.ErrBadArgument, err.Error())
	}

	signers := tx.GetSigners()
	// assert the signers data provided in options are the same as the expected signing accounts
	// and that the number of rosetta provided public keys equals the one of the signers
	if len(metadata.SignersData) != len(signers) || len(signers) != len(rosPubKeys) {
		return nil, nil, crgerrs.WrapError(crgerrs.ErrBadArgument, "signers data and account identifiers mismatch")
	}

	// add transaction metadata
	builder, err := c.txBuilderFromTx(tx)
	if err != nil {
		return nil, nil, crgerrs.WrapError(crgerrs.ErrCodec, err.Error())
	}
	builder.SetFeeAmount(feeAmount)
	builder.SetGasLimit(metadata.GasLimit)
	builder.SetMemo(metadata.Memo)

	// build signatures
	partialSignatures := make([]signing.SignatureV2, len(signers))
	payloadsToSign = make([]*rosettatypes.SigningPayload, len(signers))

	// pub key ordering matters, in a future release this check might be relaxed
	for i, signer := range signers {
		// assert that the provided public keys are correctly ordered
		// by checking if the signer at index i matches the pubkey at index
		pubKey, err := c.ToSDK().PubKey(rosPubKeys[0])
		if err != nil {
			return nil, nil, err
		}
		if !bytes.Equal(pubKey.Address().Bytes(), signer.Bytes()) {
			return nil, nil, crgerrs.WrapError(
				crgerrs.ErrBadArgument,
				fmt.Sprintf("public key at index %d does not match the expected transaction signer: %X <-> %X", i, rosPubKeys[i].Bytes, signer.Bytes()),
			)
		}

		// set the signer data
		signerData := authsigning.SignerData{
			Address:       signer.String(),
			ChainID:       metadata.ChainID,
			AccountNumber: metadata.SignersData[i].AccountNumber,
			Sequence:      metadata.SignersData[i].Sequence,
			PubKey:        pubKey,
		}

		// get signature bytes
		signBytes, err := c.bytesToSign(tx, signerData)
		if err != nil {
			return nil, nil, crgerrs.WrapError(crgerrs.ErrUnknown, fmt.Sprintf("unable to sign tx: %s", err.Error()))
		}

		// set payload
		payloadsToSign[i] = &rosettatypes.SigningPayload{
			AccountIdentifier: &rosettatypes.AccountIdentifier{Address: signer.String()},
			Bytes:             signBytes,
			SignatureType:     rosettatypes.Ecdsa,
		}

		// set partial signature
		partialSignatures[i] = signing.SignatureV2{
			PubKey:   pubKey,
			Data:     &signing.SingleSignatureData{}, // needs to be set to empty otherwise the codec will cry
			Sequence: metadata.SignersData[i].Sequence,
		}

	}

	// now we set the partial signatures in the tx
	// because we will need to decode the sequence
	// information of each account in a stateless way
	err = builder.SetSignatures(partialSignatures...)
	if err != nil {
		return nil, nil, crgerrs.WrapError(crgerrs.ErrCodec, err.Error())
	}

	// finally encode the tx
	txBytes, err = c.txEncode(builder.GetTx())
	if err != nil {
		return nil, nil, crgerrs.WrapError(crgerrs.ErrCodec, err.Error())
	}

	return txBytes, payloadsToSign, nil
}

// SignerData converts the given any account to signer data
func (c converter) SignerData(anyAccount *codectypes.Any) (*SignerData, error) {
	var acc auth.AccountI
	err := c.ir.UnpackAny(anyAccount, &acc)
	if err != nil {
		return nil, crgerrs.WrapError(crgerrs.ErrCodec, err.Error())
	}

	return &SignerData{
		AccountNumber: acc.GetAccountNumber(),
		Sequence:      acc.GetSequence(),
	}, nil
}
