package types

import (
	"encoding/json"
	"fmt"
)

type replyOn int

const (
	UnsetReplyOn replyOn = iota // The default value. We never return this in any valid instance (see toReplyOn).
	ReplyAlways
	ReplySuccess
	ReplyError
	ReplyNever
)

var fromReplyOn = map[replyOn]string{
	ReplyAlways:  "always",
	ReplySuccess: "success",
	ReplyError:   "error",
	ReplyNever:   "never",
}

var toReplyOn = map[string]replyOn{
	"always":  ReplyAlways,
	"success": ReplySuccess,
	"error":   ReplyError,
	"never":   ReplyNever,
}

func (r replyOn) String() string {
	return fromReplyOn[r]
}

func (s replyOn) MarshalJSON() ([]byte, error) {
	return json.Marshal(s.String())
}

func (s *replyOn) UnmarshalJSON(b []byte) error {
	var j string
	err := json.Unmarshal(b, &j)
	if err != nil {
		return err
	}

	voteOption, ok := toReplyOn[j]
	if !ok {
		return fmt.Errorf("invalid reply_on value '%v'", j)
	}
	*s = voteOption
	return nil
}

// SubMsg wraps a CosmosMsg with some metadata for handling replies (ID) and optionally
// limiting the gas usage (GasLimit)
type SubMsg struct {
	// An arbitrary ID chosen by the contract.
	// This is typically used to match `Reply`s in the `reply` entry point to the submessage.
	ID  uint64    `json:"id"`
	Msg CosmosMsg `json:"msg"`
	// Some arbitrary data that the contract can set in an application specific way.
	// This is just passed into the `reply` entry point and is not stored to state.
	// Any encoding can be used. If `id` is used to identify a particular action,
	// the encoding can also be different for each of those actions since you can match `id`
	// first and then start processing the `payload`.
	//
	// The environment restricts the length of this field in order to avoid abuse. The limit
	// is environment specific and can change over time. The initial default is 128 KiB.
	//
	// Unset/nil/null cannot be differentiated from empty data.
	//
	// On chains running CosmWasm 1.x this field will be ignored.
	Payload []byte `json:"payload,omitempty"`
	// Gas limit measured in [Cosmos SDK gas](https://github.com/CosmWasm/cosmwasm/blob/main/docs/GAS.md).
	//
	// Setting this to `None` means unlimited. Then the submessage execution can consume all gas of
	// the current execution context.
	GasLimit *uint64 `json:"gas_limit,omitempty"`
	ReplyOn  replyOn `json:"reply_on"`
}

// The result object returned to `reply`. We always get the ID from the submessage back and then must handle success and error cases ourselves.
type Reply struct {
	// The amount of gas used by the submessage, measured in [Cosmos SDK gas](https://github.com/CosmWasm/cosmwasm/blob/main/docs/GAS.md).
	GasUsed uint64 `json:"gas_used"`
	// The ID that the contract set when emitting the `SubMsg`. Use this to identify which submessage triggered the `reply`.
	ID     uint64       `json:"id"`
	Result SubMsgResult `json:"result"`
	// Some arbitrary data that the contract set when emitting the `SubMsg`.
	// This is just passed into the `reply` entry point and is not stored to state.
	//
	// Unset/nil/null cannot be differentiated from empty data.
	//
	// On chains running CosmWasm 1.x this field is never filled.
	Payload []byte `json:"payload,omitempty"`
}

// SubMsgResult is the raw response we return from wasmd after executing a SubMsg.
// This mirrors Rust's SubMsgResult.
type SubMsgResult struct {
	Ok  *SubMsgResponse `json:"ok,omitempty"`
	Err string          `json:"error,omitempty"`
}

// SubMsgResponse contains information we get back from a successful sub message execution,
// with full Cosmos SDK events.
// This mirrors Rust's SubMsgResponse.
type SubMsgResponse struct {
	Events       Array[Event]       `json:"events"`
	Data         []byte             `json:"data,omitempty"`
	MsgResponses Array[MsgResponse] `json:"msg_responses"`
}

type MsgResponse struct {
	TypeURL string `json:"type_url"`
	Value   []byte `json:"value"`
}
