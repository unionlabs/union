package baseapp

import (
	"bytes"
	"context"
	"fmt"

	"github.com/cockroachdb/errors"
	abci "github.com/cometbft/cometbft/abci/types"
	cryptoenc "github.com/cometbft/cometbft/crypto/encoding"
	cmtprotocrypto "github.com/cometbft/cometbft/proto/tendermint/crypto"
	cmtproto "github.com/cometbft/cometbft/proto/tendermint/types"
	protoio "github.com/cosmos/gogoproto/io"
	"github.com/cosmos/gogoproto/proto"

	"cosmossdk.io/math"

	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/mempool"
)

// VoteExtensionThreshold defines the total voting power % that must be
// submitted in order for all vote extensions to be considered valid for a
// given height.
var VoteExtensionThreshold = math.LegacyNewDecWithPrec(667, 3)

type (
	// ValidatorStore defines the interface contract require for verifying vote
	// extension signatures. Typically, this will be implemented by the x/staking
	// module, which has knowledge of the CometBFT public key.
	ValidatorStore interface {
		GetPubKeyByConsAddr(context.Context, sdk.ConsAddress) (cmtprotocrypto.PublicKey, error)
	}

	// GasTx defines the contract that a transaction with a gas limit must implement.
	GasTx interface {
		GetGas() uint64
	}
)

// ValidateVoteExtensions defines a helper function for verifying vote extension
// signatures that may be passed or manually injected into a block proposal from
// a proposer in PrepareProposal. It returns an error if any signature is invalid
// or if unexpected vote extensions and/or signatures are found or less than 2/3
// power is received.
func ValidateVoteExtensions(
	ctx sdk.Context,
	valStore ValidatorStore,
	currentHeight int64,
	chainID string,
	extCommit abci.ExtendedCommitInfo,
) error {
	cp := ctx.ConsensusParams()
	// Start checking vote extensions only **after** the vote extensions enable
	// height, because when `currentHeight == VoteExtensionsEnableHeight`
	// PrepareProposal doesn't get any vote extensions in its request.
	extsEnabled := cp.Abci != nil && currentHeight > cp.Abci.VoteExtensionsEnableHeight && cp.Abci.VoteExtensionsEnableHeight != 0
	marshalDelimitedFn := func(msg proto.Message) ([]byte, error) {
		var buf bytes.Buffer
		if err := protoio.NewDelimitedWriter(&buf).WriteMsg(msg); err != nil {
			return nil, err
		}

		return buf.Bytes(), nil
	}

	var (
		// Total voting power of all vote extensions.
		totalVP int64
		// Total voting power of all validators that submitted valid vote extensions.
		sumVP int64
	)

	for _, vote := range extCommit.Votes {
		totalVP += vote.Validator.Power

		// Only check + include power if the vote is a commit vote. There must be super-majority, otherwise the
		// previous block (the block vote is for) could not have been committed.
		if vote.BlockIdFlag != cmtproto.BlockIDFlagCommit {
			continue
		}

		if !extsEnabled {
			if len(vote.VoteExtension) > 0 {
				return fmt.Errorf("vote extensions disabled; received non-empty vote extension at height %d", currentHeight)
			}
			if len(vote.ExtensionSignature) > 0 {
				return fmt.Errorf("vote extensions disabled; received non-empty vote extension signature at height %d", currentHeight)
			}

			continue
		}

		if len(vote.ExtensionSignature) == 0 {
			return fmt.Errorf("vote extensions enabled; received empty vote extension signature at height %d", currentHeight)
		}

		valConsAddr := sdk.ConsAddress(vote.Validator.Address)
		pubKeyProto, err := valStore.GetPubKeyByConsAddr(ctx, valConsAddr)
		if err != nil {
			return fmt.Errorf("failed to get validator %X public key: %w", valConsAddr, err)
		}

		cmtPubKey, err := cryptoenc.PubKeyFromProto(pubKeyProto)
		if err != nil {
			return fmt.Errorf("failed to convert validator %X public key: %w", valConsAddr, err)
		}

		cve := cmtproto.CanonicalVoteExtension{
			Extension: vote.VoteExtension,
			Height:    currentHeight - 1, // the vote extension was signed in the previous height
			Round:     int64(extCommit.Round),
			ChainId:   chainID,
		}

		extSignBytes, err := marshalDelimitedFn(&cve)
		if err != nil {
			return fmt.Errorf("failed to encode CanonicalVoteExtension: %w", err)
		}

		if !cmtPubKey.VerifySignature(extSignBytes, vote.ExtensionSignature) {
			return fmt.Errorf("failed to verify validator %X vote extension signature", valConsAddr)
		}

		sumVP += vote.Validator.Power
	}

	if totalVP > 0 {
		percentSubmitted := math.LegacyNewDecFromInt(math.NewInt(sumVP)).Quo(math.LegacyNewDecFromInt(math.NewInt(totalVP)))
		if percentSubmitted.LT(VoteExtensionThreshold) {
			return fmt.Errorf("insufficient cumulative voting power received to verify vote extensions; got: %s, expected: >=%s", percentSubmitted, VoteExtensionThreshold)
		}
	}

	return nil
}

type (
	// ProposalTxVerifier defines the interface that is implemented by BaseApp,
	// that any custom ABCI PrepareProposal and ProcessProposal handler can use
	// to verify a transaction.
	ProposalTxVerifier interface {
		PrepareProposalVerifyTx(tx sdk.Tx) ([]byte, error)
		ProcessProposalVerifyTx(txBz []byte) (sdk.Tx, error)
		TxDecode(txBz []byte) (sdk.Tx, error)
		TxEncode(tx sdk.Tx) ([]byte, error)
	}

	// DefaultProposalHandler defines the default ABCI PrepareProposal and
	// ProcessProposal handlers.
	DefaultProposalHandler struct {
		mempool    mempool.Mempool
		txVerifier ProposalTxVerifier
		txSelector TxSelector
	}
)

func NewDefaultProposalHandler(mp mempool.Mempool, txVerifier ProposalTxVerifier) *DefaultProposalHandler {
	return &DefaultProposalHandler{
		mempool:    mp,
		txVerifier: txVerifier,
		txSelector: NewDefaultTxSelector(),
	}
}

// SetTxSelector sets the TxSelector function on the DefaultProposalHandler.
func (h *DefaultProposalHandler) SetTxSelector(ts TxSelector) {
	h.txSelector = ts
}

// PrepareProposalHandler returns the default implementation for processing an
// ABCI proposal. The application's mempool is enumerated and all valid
// transactions are added to the proposal. Transactions are valid if they:
//
// 1) Successfully encode to bytes.
// 2) Are valid (i.e. pass runTx, AnteHandler only).
//
// Enumeration is halted once RequestPrepareProposal.MaxBytes of transactions is
// reached or the mempool is exhausted.
//
// Note:
//
// - Step (2) is identical to the validation step performed in
// DefaultProcessProposal. It is very important that the same validation logic
// is used in both steps, and applications must ensure that this is the case in
// non-default handlers.
//
// - If no mempool is set or if the mempool is a no-op mempool, the transactions
// requested from CometBFT will simply be returned, which, by default, are in
// FIFO order.
func (h *DefaultProposalHandler) PrepareProposalHandler() sdk.PrepareProposalHandler {
	return func(ctx sdk.Context, req *abci.RequestPrepareProposal) (*abci.ResponsePrepareProposal, error) {
		var maxBlockGas uint64
		if b := ctx.ConsensusParams().Block; b != nil {
			maxBlockGas = uint64(b.MaxGas)
		}

		defer h.txSelector.Clear()

		// If the mempool is nil or NoOp we simply return the transactions
		// requested from CometBFT, which, by default, should be in FIFO order.
		//
		// Note, we still need to ensure the transactions returned respect req.MaxTxBytes.
		_, isNoOp := h.mempool.(mempool.NoOpMempool)
		if h.mempool == nil || isNoOp {
			for _, txBz := range req.Txs {
				tx, err := h.txVerifier.TxDecode(txBz)
				if err != nil {
					return nil, err
				}

				stop := h.txSelector.SelectTxForProposal(ctx, uint64(req.MaxTxBytes), maxBlockGas, tx, txBz)
				if stop {
					break
				}
			}

			return &abci.ResponsePrepareProposal{Txs: h.txSelector.SelectedTxs(ctx)}, nil
		}

		iterator := h.mempool.Select(ctx, req.Txs)
		for iterator != nil {
			memTx := iterator.Tx()

			// NOTE: Since transaction verification was already executed in CheckTx,
			// which calls mempool.Insert, in theory everything in the pool should be
			// valid. But some mempool implementations may insert invalid txs, so we
			// check again.
			txBz, err := h.txVerifier.PrepareProposalVerifyTx(memTx)
			if err != nil {
				err := h.mempool.Remove(memTx)
				if err != nil && !errors.Is(err, mempool.ErrTxNotFound) {
					return nil, err
				}
			} else {
				stop := h.txSelector.SelectTxForProposal(ctx, uint64(req.MaxTxBytes), maxBlockGas, memTx, txBz)
				if stop {
					break
				}
			}

			iterator = iterator.Next()
		}

		return &abci.ResponsePrepareProposal{Txs: h.txSelector.SelectedTxs(ctx)}, nil
	}
}

// ProcessProposalHandler returns the default implementation for processing an
// ABCI proposal. Every transaction in the proposal must pass 2 conditions:
//
// 1. The transaction bytes must decode to a valid transaction.
// 2. The transaction must be valid (i.e. pass runTx, AnteHandler only)
//
// If any transaction fails to pass either condition, the proposal is rejected.
// Note that step (2) is identical to the validation step performed in
// DefaultPrepareProposal. It is very important that the same validation logic
// is used in both steps, and applications must ensure that this is the case in
// non-default handlers.
func (h *DefaultProposalHandler) ProcessProposalHandler() sdk.ProcessProposalHandler {
	// If the mempool is nil or NoOp we simply return ACCEPT,
	// because PrepareProposal may have included txs that could fail verification.
	_, isNoOp := h.mempool.(mempool.NoOpMempool)
	if h.mempool == nil || isNoOp {
		return NoOpProcessProposal()
	}

	return func(ctx sdk.Context, req *abci.RequestProcessProposal) (*abci.ResponseProcessProposal, error) {
		var totalTxGas uint64

		var maxBlockGas int64
		if b := ctx.ConsensusParams().Block; b != nil {
			maxBlockGas = b.MaxGas
		}

		for _, txBytes := range req.Txs {
			tx, err := h.txVerifier.ProcessProposalVerifyTx(txBytes)
			if err != nil {
				return &abci.ResponseProcessProposal{Status: abci.ResponseProcessProposal_REJECT}, nil
			}

			if maxBlockGas > 0 {
				gasTx, ok := tx.(GasTx)
				if ok {
					totalTxGas += gasTx.GetGas()
				}

				if totalTxGas > uint64(maxBlockGas) {
					return &abci.ResponseProcessProposal{Status: abci.ResponseProcessProposal_REJECT}, nil
				}
			}
		}

		return &abci.ResponseProcessProposal{Status: abci.ResponseProcessProposal_ACCEPT}, nil
	}
}

// NoOpPrepareProposal defines a no-op PrepareProposal handler. It will always
// return the transactions sent by the client's request.
func NoOpPrepareProposal() sdk.PrepareProposalHandler {
	return func(_ sdk.Context, req *abci.RequestPrepareProposal) (*abci.ResponsePrepareProposal, error) {
		return &abci.ResponsePrepareProposal{Txs: req.Txs}, nil
	}
}

// NoOpProcessProposal defines a no-op ProcessProposal Handler. It will always
// return ACCEPT.
func NoOpProcessProposal() sdk.ProcessProposalHandler {
	return func(_ sdk.Context, _ *abci.RequestProcessProposal) (*abci.ResponseProcessProposal, error) {
		return &abci.ResponseProcessProposal{Status: abci.ResponseProcessProposal_ACCEPT}, nil
	}
}

// NoOpExtendVote defines a no-op ExtendVote handler. It will always return an
// empty byte slice as the vote extension.
func NoOpExtendVote() sdk.ExtendVoteHandler {
	return func(_ sdk.Context, _ *abci.RequestExtendVote) (*abci.ResponseExtendVote, error) {
		return &abci.ResponseExtendVote{VoteExtension: []byte{}}, nil
	}
}

// NoOpVerifyVoteExtensionHandler defines a no-op VerifyVoteExtension handler. It
// will always return an ACCEPT status with no error.
func NoOpVerifyVoteExtensionHandler() sdk.VerifyVoteExtensionHandler {
	return func(_ sdk.Context, _ *abci.RequestVerifyVoteExtension) (*abci.ResponseVerifyVoteExtension, error) {
		return &abci.ResponseVerifyVoteExtension{Status: abci.ResponseVerifyVoteExtension_ACCEPT}, nil
	}
}

// TxSelector defines a helper type that assists in selecting transactions during
// mempool transaction selection in PrepareProposal. It keeps track of the total
// number of bytes and total gas of the selected transactions. It also keeps
// track of the selected transactions themselves.
type TxSelector interface {
	// SelectedTxs should return a copy of the selected transactions.
	SelectedTxs(ctx context.Context) [][]byte

	// Clear should clear the TxSelector, nulling out all relevant fields.
	Clear()

	// SelectTxForProposal should attempt to select a transaction for inclusion in
	// a proposal based on inclusion criteria defined by the TxSelector. It must
	// return <true> if the caller should halt the transaction selection loop
	// (typically over a mempool) or <false> otherwise.
	SelectTxForProposal(ctx context.Context, maxTxBytes, maxBlockGas uint64, memTx sdk.Tx, txBz []byte) bool
}

type defaultTxSelector struct {
	totalTxBytes uint64
	totalTxGas   uint64
	selectedTxs  [][]byte
}

func NewDefaultTxSelector() TxSelector {
	return &defaultTxSelector{}
}

func (ts *defaultTxSelector) SelectedTxs(_ context.Context) [][]byte {
	txs := make([][]byte, len(ts.selectedTxs))
	copy(txs, ts.selectedTxs)
	return txs
}

func (ts *defaultTxSelector) Clear() {
	ts.totalTxBytes = 0
	ts.totalTxGas = 0
	ts.selectedTxs = nil
}

func (ts *defaultTxSelector) SelectTxForProposal(_ context.Context, maxTxBytes, maxBlockGas uint64, memTx sdk.Tx, txBz []byte) bool {
	txSize := uint64(len(txBz))

	var txGasLimit uint64
	if memTx != nil {
		if gasTx, ok := memTx.(GasTx); ok {
			txGasLimit = gasTx.GetGas()
		}
	}

	// only add the transaction to the proposal if we have enough capacity
	if (txSize + ts.totalTxBytes) <= maxTxBytes {
		// If there is a max block gas limit, add the tx only if the limit has
		// not been met.
		if maxBlockGas > 0 {
			if (txGasLimit + ts.totalTxGas) <= maxBlockGas {
				ts.totalTxGas += txGasLimit
				ts.totalTxBytes += txSize
				ts.selectedTxs = append(ts.selectedTxs, txBz)
			}
		} else {
			ts.totalTxBytes += txSize
			ts.selectedTxs = append(ts.selectedTxs, txBz)
		}
	}

	// check if we've reached capacity; if so, we cannot select any more transactions
	return ts.totalTxBytes >= maxTxBytes || (maxBlockGas > 0 && (ts.totalTxGas >= maxBlockGas))
}
