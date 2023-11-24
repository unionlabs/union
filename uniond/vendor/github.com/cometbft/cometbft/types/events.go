package types

import (
	"fmt"

	abci "github.com/cometbft/cometbft/abci/types"
	cmtjson "github.com/cometbft/cometbft/libs/json"
	cmtpubsub "github.com/cometbft/cometbft/libs/pubsub"
	cmtquery "github.com/cometbft/cometbft/libs/pubsub/query"
)

// Reserved event types (alphabetically sorted).
const (
	// Block level events for mass consumption by users.
	// These events are triggered from the state package,
	// after a block has been committed.
	// These are also used by the tx indexer for async indexing.
	// All of this data can be fetched through the rpc.
	EventNewBlock            = "NewBlock"
	EventNewBlockHeader      = "NewBlockHeader"
	EventNewBlockEvents      = "NewBlockEvents"
	EventNewEvidence         = "NewEvidence"
	EventTx                  = "Tx"
	EventValidatorSetUpdates = "ValidatorSetUpdates"

	// Internal consensus events.
	// These are used for testing the consensus state machine.
	// They can also be used to build real-time consensus visualizers.
	EventCompleteProposal = "CompleteProposal"
	EventLock             = "Lock"
	EventNewRound         = "NewRound"
	EventNewRoundStep     = "NewRoundStep"
	EventPolka            = "Polka"
	EventRelock           = "Relock"
	EventTimeoutPropose   = "TimeoutPropose"
	EventTimeoutWait      = "TimeoutWait"
	EventUnlock           = "Unlock"
	EventValidBlock       = "ValidBlock"
	EventVote             = "Vote"
)

// ENCODING / DECODING

// TMEventData implements events.EventData.
type TMEventData interface {
	// empty interface
}

func init() {
	cmtjson.RegisterType(EventDataNewBlock{}, "tendermint/event/NewBlock")
	cmtjson.RegisterType(EventDataNewBlockHeader{}, "tendermint/event/NewBlockHeader")
	cmtjson.RegisterType(EventDataNewBlockEvents{}, "tendermint/event/NewBlockEvents")
	cmtjson.RegisterType(EventDataNewEvidence{}, "tendermint/event/NewEvidence")
	cmtjson.RegisterType(EventDataTx{}, "tendermint/event/Tx")
	cmtjson.RegisterType(EventDataRoundState{}, "tendermint/event/RoundState")
	cmtjson.RegisterType(EventDataNewRound{}, "tendermint/event/NewRound")
	cmtjson.RegisterType(EventDataCompleteProposal{}, "tendermint/event/CompleteProposal")
	cmtjson.RegisterType(EventDataVote{}, "tendermint/event/Vote")
	cmtjson.RegisterType(EventDataValidatorSetUpdates{}, "tendermint/event/ValidatorSetUpdates")
	cmtjson.RegisterType(EventDataString(""), "tendermint/event/ProposalString")
}

// Most event messages are basic types (a block, a transaction)
// but some (an input to a call tx or a receive) are more exotic

type EventDataNewBlock struct {
	Block               *Block                     `json:"block"`
	BlockID             BlockID                    `json:"block_id"`
	ResultFinalizeBlock abci.ResponseFinalizeBlock `json:"result_finalize_block"`
}

type EventDataNewBlockHeader struct {
	Header Header `json:"header"`
}

type EventDataNewBlockEvents struct {
	Height int64        `json:"height"`
	Events []abci.Event `json:"events"`
	NumTxs int64        `json:"num_txs,string"` // Number of txs in a block
}

type EventDataNewEvidence struct {
	Height   int64    `json:"height"`
	Evidence Evidence `json:"evidence"`
}

// All txs fire EventDataTx
type EventDataTx struct {
	abci.TxResult
}

// NOTE: This goes into the replay WAL
type EventDataRoundState struct {
	Height int64  `json:"height"`
	Round  int32  `json:"round"`
	Step   string `json:"step"`
}

type ValidatorInfo struct {
	Address Address `json:"address"`
	Index   int32   `json:"index"`
}

type EventDataNewRound struct {
	Height int64  `json:"height"`
	Round  int32  `json:"round"`
	Step   string `json:"step"`

	Proposer ValidatorInfo `json:"proposer"`
}

type EventDataCompleteProposal struct {
	Height int64  `json:"height"`
	Round  int32  `json:"round"`
	Step   string `json:"step"`

	BlockID BlockID `json:"block_id"`
}

type EventDataVote struct {
	Vote *Vote
}

type EventDataString string

type EventDataValidatorSetUpdates struct {
	ValidatorUpdates []*Validator `json:"validator_updates"`
}

// PUBSUB

const (
	// EventTypeKey is a reserved composite key for event name.
	EventTypeKey = "tm.event"

	// TxHashKey is a reserved key, used to specify transaction's hash.
	// see EventBus#PublishEventTx
	TxHashKey = "tx.hash"

	// TxHeightKey is a reserved key, used to specify transaction block's height.
	// see EventBus#PublishEventTx
	TxHeightKey = "tx.height"

	// BlockHeightKey is a reserved key used for indexing FinalizeBlock events.
	BlockHeightKey = "block.height"
)

var (
	EventQueryCompleteProposal    = QueryForEvent(EventCompleteProposal)
	EventQueryLock                = QueryForEvent(EventLock)
	EventQueryNewBlock            = QueryForEvent(EventNewBlock)
	EventQueryNewBlockHeader      = QueryForEvent(EventNewBlockHeader)
	EventQueryNewBlockEvents      = QueryForEvent(EventNewBlockEvents)
	EventQueryNewEvidence         = QueryForEvent(EventNewEvidence)
	EventQueryNewRound            = QueryForEvent(EventNewRound)
	EventQueryNewRoundStep        = QueryForEvent(EventNewRoundStep)
	EventQueryPolka               = QueryForEvent(EventPolka)
	EventQueryRelock              = QueryForEvent(EventRelock)
	EventQueryTimeoutPropose      = QueryForEvent(EventTimeoutPropose)
	EventQueryTimeoutWait         = QueryForEvent(EventTimeoutWait)
	EventQueryTx                  = QueryForEvent(EventTx)
	EventQueryUnlock              = QueryForEvent(EventUnlock)
	EventQueryValidatorSetUpdates = QueryForEvent(EventValidatorSetUpdates)
	EventQueryValidBlock          = QueryForEvent(EventValidBlock)
	EventQueryVote                = QueryForEvent(EventVote)
)

func EventQueryTxFor(tx Tx) cmtpubsub.Query {
	return cmtquery.MustCompile(fmt.Sprintf("%s='%s' AND %s='%X'", EventTypeKey, EventTx, TxHashKey, tx.Hash()))
}

func QueryForEvent(eventType string) cmtpubsub.Query {
	return cmtquery.MustCompile(fmt.Sprintf("%s='%s'", EventTypeKey, eventType))
}

// BlockEventPublisher publishes all block related events
type BlockEventPublisher interface {
	PublishEventNewBlock(block EventDataNewBlock) error
	PublishEventNewBlockHeader(header EventDataNewBlockHeader) error
	PublishEventNewBlockEvents(events EventDataNewBlockEvents) error
	PublishEventNewEvidence(evidence EventDataNewEvidence) error
	PublishEventTx(EventDataTx) error
	PublishEventValidatorSetUpdates(EventDataValidatorSetUpdates) error
}

type TxEventPublisher interface {
	PublishEventTx(EventDataTx) error
}
