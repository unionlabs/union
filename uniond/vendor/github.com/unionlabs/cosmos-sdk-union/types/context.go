package types

import (
	"context"
	"time"

	abci "github.com/cometbft/cometbft/api/cometbft/abci/v1"
	cmtproto "github.com/cometbft/cometbft/api/cometbft/types/v1"

	"cosmossdk.io/core/comet"
	"cosmossdk.io/core/header"
	"cosmossdk.io/log"
	"cosmossdk.io/store/gaskv"
	storetypes "cosmossdk.io/store/types"
)

// ExecMode defines the execution mode which can be set on a Context.
type ExecMode uint8

// All possible execution modes.
const (
	ExecModeCheck ExecMode = iota
	ExecModeReCheck
	ExecModeSimulate
	ExecModePrepareProposal
	ExecModeProcessProposal
	ExecModeVoteExtension
	ExecModeVerifyVoteExtension
	ExecModeFinalize
)

/*
Context is an immutable object contains all information needed to
process a request.

It contains a context.Context object inside if you want to use that,
but please do not over-use it. We try to keep all data structured
and standard additions here would be better just to add to the Context struct
*/
type Context struct {
	baseCtx              context.Context
	ms                   storetypes.MultiStore
	header               cmtproto.Header // Deprecated: Use HeaderService for height, time, and chainID and CometService for the rest
	headerHash           []byte          // Deprecated: Use HeaderService for hash
	chainID              string          // Deprecated: Use HeaderService for chainID and CometService for the rest
	txBytes              []byte
	logger               log.Logger
	voteInfo             []abci.VoteInfo // Deprecated: use Cometinfo.LastCommit.Votes instead, will be removed after 0.52
	gasMeter             storetypes.GasMeter
	blockGasMeter        storetypes.GasMeter
	checkTx              bool // Deprecated: use execMode instead, will be removed after 0.52
	recheckTx            bool // if recheckTx == true, then checkTx must also be true // Deprecated: use execMode instead, will be removed after 0.52
	sigverifyTx          bool // when run simulation, because the private key corresponding to the account in the genesis.json randomly generated, we must skip the sigverify.
	execMode             ExecMode
	minGasPrice          DecCoins
	consParams           cmtproto.ConsensusParams
	eventManager         EventManagerI
	priority             int64 // The tx priority, only relevant in CheckTx
	kvGasConfig          storetypes.GasConfig
	transientKVGasConfig storetypes.GasConfig
	streamingManager     storetypes.StreamingManager
	cometInfo            comet.Info
	headerInfo           header.Info
}

// Proposed rename, not done to avoid API breakage
type Request = Context

// Read-only accessors
func (c Context) Context() context.Context                      { return c.baseCtx }
func (c Context) MultiStore() storetypes.MultiStore             { return c.ms }
func (c Context) BlockHeight() int64                            { return c.header.Height }
func (c Context) BlockTime() time.Time                          { return c.headerInfo.Time } // Deprecated: use HeaderInfo().Time
func (c Context) ChainID() string                               { return c.chainID }
func (c Context) TxBytes() []byte                               { return c.txBytes }
func (c Context) Logger() log.Logger                            { return c.logger }
func (c Context) VoteInfos() []abci.VoteInfo                    { return c.voteInfo }
func (c Context) GasMeter() storetypes.GasMeter                 { return c.gasMeter }
func (c Context) BlockGasMeter() storetypes.GasMeter            { return c.blockGasMeter }
func (c Context) IsCheckTx() bool                               { return c.checkTx }   // Deprecated: use core/transaction service instead
func (c Context) IsReCheckTx() bool                             { return c.recheckTx } // Deprecated: use core/transaction service instead
func (c Context) IsSigverifyTx() bool                           { return c.sigverifyTx }
func (c Context) ExecMode() ExecMode                            { return c.execMode } // Deprecated: use core/transaction service instead
func (c Context) MinGasPrices() DecCoins                        { return c.minGasPrice }
func (c Context) EventManager() EventManagerI                   { return c.eventManager }
func (c Context) Priority() int64                               { return c.priority }
func (c Context) KVGasConfig() storetypes.GasConfig             { return c.kvGasConfig }
func (c Context) TransientKVGasConfig() storetypes.GasConfig    { return c.transientKVGasConfig }
func (c Context) StreamingManager() storetypes.StreamingManager { return c.streamingManager }
func (c Context) CometInfo() comet.Info                         { return c.cometInfo }
func (c Context) HeaderInfo() header.Info                       { return c.headerInfo }

// BlockHeader returns the header by value.
func (c Context) BlockHeader() cmtproto.Header {
	return c.header
}

// HeaderHash returns a copy of the header hash obtained during abci.RequestBeginBlock
func (c Context) HeaderHash() []byte {
	hash := make([]byte, len(c.headerHash))
	copy(hash, c.headerHash)
	return hash
}

// Deprecated: getting consensus params from the context is deprecated and will be removed after 0.52
// Querying the consensus module for the parameters is required in server/v2
func (c Context) ConsensusParams() cmtproto.ConsensusParams {
	return c.consParams
}

func (c Context) Deadline() (deadline time.Time, ok bool) {
	return c.baseCtx.Deadline()
}

func (c Context) Done() <-chan struct{} {
	return c.baseCtx.Done()
}

func (c Context) Err() error {
	return c.baseCtx.Err()
}

// create a new context
func NewContext(ms storetypes.MultiStore, isCheckTx bool, logger log.Logger) Context {
	h := cmtproto.Header{}
	h.Time = h.Time.UTC()
	return Context{
		baseCtx:              context.Background(),
		ms:                   ms,
		header:               h,
		chainID:              h.ChainID,
		checkTx:              isCheckTx,
		sigverifyTx:          true,
		logger:               logger,
		gasMeter:             storetypes.NewInfiniteGasMeter(),
		minGasPrice:          DecCoins{},
		eventManager:         NewEventManager(),
		kvGasConfig:          storetypes.KVGasConfig(),
		transientKVGasConfig: storetypes.TransientGasConfig(),
		headerInfo: header.Info{
			Time: h.Time,
		},
	}
}

// WithContext returns a Context with an updated context.Context.
func (c Context) WithContext(ctx context.Context) Context {
	c.baseCtx = ctx
	return c
}

// WithMultiStore returns a Context with an updated MultiStore.
func (c Context) WithMultiStore(ms storetypes.MultiStore) Context {
	c.ms = ms
	return c
}

// WithBlockHeader returns a Context with an updated CometBFT block header in UTC time.
func (c Context) WithBlockHeader(header cmtproto.Header) Context {
	// https://github.com/gogo/protobuf/issues/519
	header.Time = header.Time.UTC()
	c.header = header

	// when calling withBlockheader on a new context, chainID in the struct will be empty
	c.chainID = header.ChainID
	return c
}

// WithHeaderHash returns a Context with an updated CometBFT block header hash.
func (c Context) WithHeaderHash(hash []byte) Context {
	temp := make([]byte, len(hash))
	copy(temp, hash)

	c.headerHash = temp
	return c
}

// WithProposer returns a Context with an updated proposer consensus address.
func (c Context) WithProposer(addr ConsAddress) Context {
	newHeader := c.BlockHeader()
	newHeader.ProposerAddress = addr.Bytes()
	return c.WithBlockHeader(newHeader)
}

// WithBlockHeight returns a Context with an updated block height.
func (c Context) WithBlockHeight(height int64) Context {
	newHeader := c.BlockHeader()
	newHeader.Height = height
	return c.WithBlockHeader(newHeader)
}

// WithChainID returns a Context with an updated chain identifier.
func (c Context) WithChainID(chainID string) Context {
	c.chainID = chainID
	return c
}

// WithTxBytes returns a Context with an updated txBytes.
func (c Context) WithTxBytes(txBytes []byte) Context {
	c.txBytes = txBytes
	return c
}

// WithLogger returns a Context with an updated logger.
func (c Context) WithLogger(logger log.Logger) Context {
	c.logger = logger
	return c
}

// WithVoteInfos returns a Context with an updated consensus VoteInfo.
// Deprecated: use WithCometinfo() instead, will be removed after 0.52
func (c Context) WithVoteInfos(voteInfo []abci.VoteInfo) Context {
	c.voteInfo = voteInfo
	return c
}

// WithGasMeter returns a Context with an updated transaction GasMeter.
func (c Context) WithGasMeter(meter storetypes.GasMeter) Context {
	c.gasMeter = meter
	return c
}

// WithBlockGasMeter returns a Context with an updated block GasMeter
func (c Context) WithBlockGasMeter(meter storetypes.GasMeter) Context {
	c.blockGasMeter = meter
	return c
}

// WithKVGasConfig returns a Context with an updated gas configuration for
// the KVStore
func (c Context) WithKVGasConfig(gasConfig storetypes.GasConfig) Context {
	c.kvGasConfig = gasConfig
	return c
}

// WithTransientKVGasConfig returns a Context with an updated gas configuration for
// the transient KVStore
func (c Context) WithTransientKVGasConfig(gasConfig storetypes.GasConfig) Context {
	c.transientKVGasConfig = gasConfig
	return c
}

// WithIsCheckTx enables or disables CheckTx value for verifying transactions and returns an updated Context
func (c Context) WithIsCheckTx(isCheckTx bool) Context {
	c.checkTx = isCheckTx
	c.execMode = ExecModeCheck
	return c
}

// WithIsRecheckTx called with true will also set true on checkTx in order to
// enforce the invariant that if recheckTx = true then checkTx = true as well.
func (c Context) WithIsReCheckTx(isRecheckTx bool) Context {
	if isRecheckTx {
		c.checkTx = true
	}
	c.recheckTx = isRecheckTx
	c.execMode = ExecModeReCheck
	return c
}

// WithIsSigverifyTx called with true will sigverify in auth module
func (c Context) WithIsSigverifyTx(isSigverifyTx bool) Context {
	c.sigverifyTx = isSigverifyTx
	return c
}

// WithExecMode returns a Context with an updated ExecMode.
func (c Context) WithExecMode(m ExecMode) Context {
	c.execMode = m
	return c
}

// WithMinGasPrices returns a Context with an updated minimum gas price value
func (c Context) WithMinGasPrices(gasPrices DecCoins) Context {
	c.minGasPrice = gasPrices
	return c
}

// WithConsensusParams returns a Context with an updated consensus params
func (c Context) WithConsensusParams(params cmtproto.ConsensusParams) Context {
	c.consParams = params
	return c
}

// WithEventManager returns a Context with an updated event manager
func (c Context) WithEventManager(em EventManagerI) Context {
	c.eventManager = em
	return c
}

// WithPriority returns a Context with an updated tx priority
func (c Context) WithPriority(p int64) Context {
	c.priority = p
	return c
}

// WithStreamingManager returns a Context with an updated streaming manager
func (c Context) WithStreamingManager(sm storetypes.StreamingManager) Context {
	c.streamingManager = sm
	return c
}

// WithCometInfo returns a Context with an updated comet info
func (c Context) WithCometInfo(cometInfo comet.Info) Context {
	c.cometInfo = cometInfo
	return c
}

// WithHeaderInfo returns a Context with an updated header info
func (c Context) WithHeaderInfo(headerInfo header.Info) Context {
	// Set time to UTC
	headerInfo.Time = headerInfo.Time.UTC()
	c.headerInfo = headerInfo
	return c
}

// TODO: remove???
func (c Context) IsZero() bool {
	return c.ms == nil
}

func (c Context) WithValue(key, value interface{}) Context {
	c.baseCtx = context.WithValue(c.baseCtx, key, value)
	return c
}

func (c Context) Value(key interface{}) interface{} {
	if key == SdkContextKey {
		return c
	}

	return c.baseCtx.Value(key)
}

// ----------------------------------------------------------------------------
// Store / Caching
// ----------------------------------------------------------------------------

// KVStore fetches a KVStore from the MultiStore.
func (c Context) KVStore(key storetypes.StoreKey) storetypes.KVStore {
	return gaskv.NewStore(c.ms.GetKVStore(key), c.gasMeter, c.kvGasConfig)
}

// TransientStore fetches a TransientStore from the MultiStore.
func (c Context) TransientStore(key storetypes.StoreKey) storetypes.KVStore {
	return gaskv.NewStore(c.ms.GetKVStore(key), c.gasMeter, c.transientKVGasConfig)
}

// CacheContext returns a new Context with the multi-store cached and a new
// EventManager. The cached context is written to the context when writeCache
// is called. Note, events are automatically emitted on the parent context's
// EventManager when the caller executes the write.
func (c Context) CacheContext() (cc Context, writeCache func()) {
	cms := c.ms.CacheMultiStore()
	cc = c.WithMultiStore(cms).WithEventManager(NewEventManager())

	writeCache = func() {
		c.EventManager().EmitEvents(cc.EventManager().Events())
		cms.Write()
	}

	return cc, writeCache
}

var (
	_ context.Context    = Context{}
	_ storetypes.Context = Context{}
)

// ContextKey defines a type alias for a stdlib Context key.
type ContextKey string

// SdkContextKey is the key in the context.Context which holds the sdk.Context.
const SdkContextKey ContextKey = "sdk-context"

// WrapSDKContext returns a stdlib context.Context with the provided sdk.Context's internal
// context as a value. It is useful for passing an sdk.Context  through methods that take a
// stdlib context.Context parameter such as generated gRPC methods. To get the original
// sdk.Context back, call UnwrapSDKContext.
//
// Deprecated: there is no need to wrap anymore as the Cosmos SDK context implements context.Context.
func WrapSDKContext(ctx Context) context.Context {
	return ctx
}

// UnwrapSDKContext retrieves a Context from a context.Context instance
// attached with WrapSDKContext. It panics if a Context was not properly
// attached
func UnwrapSDKContext(ctx context.Context) Context {
	if sdkCtx, ok := ctx.(Context); ok {
		return sdkCtx
	}
	return ctx.Value(SdkContextKey).(Context)
}

// TryUnwrapSDKContext attempts to retrieve a Context from a context.Context
func TryUnwrapSDKContext(ctx context.Context) (Context, bool) {
	if sdkCtx, ok := ctx.(Context); ok {
		return sdkCtx, true
	}
	v := ctx.Value(SdkContextKey)
	if v == nil {
		return Context{}, false
	}
	c, ok := v.(Context)
	return c, ok
}

// ToSDKEvidence takes comet evidence and returns sdk evidence
func ToSDKEvidence(ev []abci.Misbehavior) []comet.Evidence {
	evidence := make([]comet.Evidence, len(ev))
	for i, e := range ev {
		evidence[i] = comet.Evidence{
			Type:             comet.MisbehaviorType(e.Type),
			Height:           e.Height,
			Time:             e.Time,
			TotalVotingPower: e.TotalVotingPower,
			Validator: comet.Validator{
				Address: e.Validator.Address,
				Power:   e.Validator.Power,
			},
		}
	}
	return evidence
}

// ToSDKCommitInfo takes comet commit info and returns sdk commit info
func ToSDKCommitInfo(commit abci.CommitInfo) comet.CommitInfo {
	ci := comet.CommitInfo{
		Round: commit.Round,
	}

	for _, v := range commit.Votes {
		ci.Votes = append(ci.Votes, comet.VoteInfo{
			Validator: comet.Validator{
				Address: v.Validator.Address,
				Power:   v.Validator.Power,
			},
			BlockIDFlag: comet.BlockIDFlag(v.BlockIdFlag),
		})
	}
	return ci
}

// ToSDKExtendedCommitInfo takes comet extended commit info and returns sdk commit info
func ToSDKExtendedCommitInfo(commit abci.ExtendedCommitInfo) comet.CommitInfo {
	ci := comet.CommitInfo{
		Round: commit.Round,
		Votes: make([]comet.VoteInfo, len(commit.Votes)),
	}

	for i, v := range commit.Votes {
		ci.Votes[i] = comet.VoteInfo{
			Validator: comet.Validator{
				Address: v.Validator.Address,
				Power:   v.Validator.Power,
			},
			BlockIDFlag: comet.BlockIDFlag(v.BlockIdFlag),
		}
	}

	return ci
}
