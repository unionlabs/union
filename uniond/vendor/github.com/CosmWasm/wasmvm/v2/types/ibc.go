package types

type IBCEndpoint struct {
	PortID    string `json:"port_id"`
	ChannelID string `json:"channel_id"`
}

type IBCChannel struct {
	Endpoint             IBCEndpoint `json:"endpoint"`
	CounterpartyEndpoint IBCEndpoint `json:"counterparty_endpoint"`
	Order                IBCOrder    `json:"order"`
	Version              string      `json:"version"`
	ConnectionID         string      `json:"connection_id"`
}

type IBCChannelOpenMsg struct {
	OpenInit *IBCOpenInit `json:"open_init,omitempty"`
	OpenTry  *IBCOpenTry  `json:"open_try,omitempty"`
}

// GetChannel returns the IBCChannel in this message.
func (msg IBCChannelOpenMsg) GetChannel() IBCChannel {
	if msg.OpenInit != nil {
		return msg.OpenInit.Channel
	}
	return msg.OpenTry.Channel
}

// GetCounterVersion checks if the message has a counterparty version and
// returns it if so.
func (msg IBCChannelOpenMsg) GetCounterVersion() (ver string, ok bool) {
	if msg.OpenTry != nil {
		return msg.OpenTry.CounterpartyVersion, true
	}
	return "", false
}

type IBCOpenInit struct {
	Channel IBCChannel `json:"channel"`
}

func (m *IBCOpenInit) ToMsg() IBCChannelOpenMsg {
	return IBCChannelOpenMsg{
		OpenInit: m,
	}
}

type IBCOpenTry struct {
	Channel             IBCChannel `json:"channel"`
	CounterpartyVersion string     `json:"counterparty_version"`
}

func (m *IBCOpenTry) ToMsg() IBCChannelOpenMsg {
	return IBCChannelOpenMsg{
		OpenTry: m,
	}
}

type IBCChannelConnectMsg struct {
	OpenAck     *IBCOpenAck     `json:"open_ack,omitempty"`
	OpenConfirm *IBCOpenConfirm `json:"open_confirm,omitempty"`
}

// GetChannel returns the IBCChannel in this message.
func (msg IBCChannelConnectMsg) GetChannel() IBCChannel {
	if msg.OpenAck != nil {
		return msg.OpenAck.Channel
	}
	return msg.OpenConfirm.Channel
}

// GetCounterVersion checks if the message has a counterparty version and
// returns it if so.
func (msg IBCChannelConnectMsg) GetCounterVersion() (ver string, ok bool) {
	if msg.OpenAck != nil {
		return msg.OpenAck.CounterpartyVersion, true
	}
	return "", false
}

type IBCOpenAck struct {
	Channel             IBCChannel `json:"channel"`
	CounterpartyVersion string     `json:"counterparty_version"`
}

func (m *IBCOpenAck) ToMsg() IBCChannelConnectMsg {
	return IBCChannelConnectMsg{
		OpenAck: m,
	}
}

type IBCOpenConfirm struct {
	Channel IBCChannel `json:"channel"`
}

func (m *IBCOpenConfirm) ToMsg() IBCChannelConnectMsg {
	return IBCChannelConnectMsg{
		OpenConfirm: m,
	}
}

type IBCChannelCloseMsg struct {
	CloseInit    *IBCCloseInit    `json:"close_init,omitempty"`
	CloseConfirm *IBCCloseConfirm `json:"close_confirm,omitempty"`
}

// GetChannel returns the IBCChannel in this message.
func (msg IBCChannelCloseMsg) GetChannel() IBCChannel {
	if msg.CloseInit != nil {
		return msg.CloseInit.Channel
	}
	return msg.CloseConfirm.Channel
}

type IBCCloseInit struct {
	Channel IBCChannel `json:"channel"`
}

func (m *IBCCloseInit) ToMsg() IBCChannelCloseMsg {
	return IBCChannelCloseMsg{
		CloseInit: m,
	}
}

type IBCCloseConfirm struct {
	Channel IBCChannel `json:"channel"`
}

func (m *IBCCloseConfirm) ToMsg() IBCChannelCloseMsg {
	return IBCChannelCloseMsg{
		CloseConfirm: m,
	}
}

type IBCPacketReceiveMsg struct {
	Packet  IBCPacket `json:"packet"`
	Relayer string    `json:"relayer"`
}

type IBCPacketAckMsg struct {
	Acknowledgement IBCAcknowledgement `json:"acknowledgement"`
	OriginalPacket  IBCPacket          `json:"original_packet"`
	Relayer         string             `json:"relayer"`
}

type IBCPacketTimeoutMsg struct {
	Packet  IBCPacket `json:"packet"`
	Relayer string    `json:"relayer"`
}

// The type of IBC source callback that is being called.
//
// IBC source callbacks are needed for cases where your contract triggers the sending of an IBC packet through some other message (i.e. not through [`IbcMsg::SendPacket`]) and needs to know whether or not the packet was successfully received on the other chain. A prominent example is the [`IbcMsg::Transfer`] message. Without callbacks, you cannot know whether the transfer was successful or not.
//
// Note that there are some prerequisites that need to be fulfilled to receive source callbacks: - The contract must implement the `ibc_source_callback` entrypoint. - The IBC application in the source chain must have support for the callbacks middleware. - You have to add serialized [`IbcCallbackRequest`] to a specific field of the message. For `IbcMsg::Transfer`, this is the `memo` field and it needs to be json-encoded. - The receiver of the callback must also be the sender of the message.
type IBCSourceCallbackMsg struct {
	Acknowledgement *IBCAckCallbackMsg     `json:"acknowledgement,omitempty"`
	Timeout         *IBCTimeoutCallbackMsg `json:"timeout,omitempty"`
}

type IBCAckCallbackMsg struct {
	Acknowledgement IBCAcknowledgement `json:"acknowledgement"`
	OriginalPacket  IBCPacket          `json:"original_packet"`
	Relayer         string             `json:"relayer"`
}

type IBCTimeoutCallbackMsg struct {
	Packet  IBCPacket `json:"packet"`
	Relayer string    `json:"relayer"`
}

// The message type of the IBC destination callback.
//
// The IBC destination callback is needed for cases where someone triggers the sending of an
// IBC packet through some other message (i.e. not through [`IbcMsg::SendPacket`]) and
// your contract needs to know that it received this.
// The callback is called after the packet was successfully acknowledged on the destination chain.
// A prominent example is the [`IbcMsg::Transfer`] message. Without callbacks, you cannot know
// that someone sent you IBC coins.
//
// Note that there are some prerequisites that need to be fulfilled to receive source callbacks:
//   - The contract must implement the `ibc_destination_callback` entrypoint.
//   - The module that receives the packet must be wrapped by an `IBCMiddleware`
//     (i.e. the destination chain needs to support callbacks for the message you are being sent).
//   - You have to add json-encoded [`IbcCallbackData`] to a specific field of the message.
//     For `IbcMsg::Transfer`, this is the `memo` field.
type IBCDestinationCallbackMsg struct {
	Ack    IBCAcknowledgement `json:"ack"`
	Packet IBCPacket          `json:"packet"`
}

// TODO: test what the sdk Order.String() represents and how to parse back
// Proto files: https://github.com/cosmos/cosmos-sdk/blob/v0.40.0/proto/ibc/core/channel/v1/channel.proto#L69-L80
// Auto-gen code: https://github.com/cosmos/cosmos-sdk/blob/v0.40.0/x/ibc/core/04-channel/types/channel.pb.go#L70-L101
type IBCOrder = string

// These are the only two valid values for IbcOrder
const (
	Unordered = "ORDER_UNORDERED"
	Ordered   = "ORDER_ORDERED"
)

// IBCTimeoutBlock Height is a monotonically increasing data type
// that can be compared against another Height for the purposes of updating and
// freezing clients.
// Ordering is (revision_number, timeout_height)
type IBCTimeoutBlock struct {
	// the version that the client is currently on
	// (eg. after reseting the chain this could increment 1 as height drops to 0)
	Revision uint64 `json:"revision"`
	// block height after which the packet times out.
	// the height within the given revision
	Height uint64 `json:"height"`
}

func (t IBCTimeoutBlock) IsZero() bool {
	return t.Revision == 0 && t.Height == 0
}

// IBCTimeout is the timeout for an IBC packet. At least one of block and timestamp is required.
type IBCTimeout struct {
	Block *IBCTimeoutBlock `json:"block"`
	// Nanoseconds since UNIX epoch
	Timestamp uint64 `json:"timestamp,string,omitempty"`
}

type IBCAcknowledgement struct {
	Data []byte `json:"data"`
}

type IBCPacket struct {
	Data     []byte      `json:"data"`
	Src      IBCEndpoint `json:"src"`
	Dest     IBCEndpoint `json:"dest"`
	Sequence uint64      `json:"sequence"`
	Timeout  IBCTimeout  `json:"timeout"`
}

// IBCChannelOpenResult is the raw response from the ibc_channel_open call.
// This is mirrors Rust's ContractResult<()>.
// Check if Err == "" to see if this is success
// On Success, IBCV3ChannelOpenResponse *may* be set if the contract is ibcv3 compatible and wishes to
// define a custom version in the handshake.
type IBCChannelOpenResult struct {
	Ok  *IBC3ChannelOpenResponse `json:"ok,omitempty"`
	Err string                   `json:"error,omitempty"`
}

// IBC3ChannelOpenResponse is version negotiation data for the handshake
type IBC3ChannelOpenResponse struct {
	Version string `json:"version"`
}

// This is the return value for the majority of the ibc handlers.
// That are able to dispatch messages / events on their own,
// but have no meaningful return value to the calling code.
//
// Callbacks that have return values (like ibc_receive_packet)
// or that cannot redispatch messages (like ibc_channel_open)
// will use other Response types
type IBCBasicResult struct {
	Ok  *IBCBasicResponse `json:"ok,omitempty"`
	Err string            `json:"error,omitempty"`
}

func (r *IBCBasicResult) SubMessages() []SubMsg {
	if r.Ok != nil {
		return r.Ok.Messages
	}
	return nil
}

// IBCBasicResponse defines the return value on a successful processing.
// This is the counterpart of [IbcBasicResponse](https://github.com/CosmWasm/cosmwasm/blob/v0.14.0-beta1/packages/std/src/ibc.rs#L194-L216).
type IBCBasicResponse struct {
	// Messages comes directly from the contract and is its request for action.
	// If the ReplyOn value matches the result, the runtime will invoke this
	// contract's `reply` entry point after execution. Otherwise, this is all
	// "fire and forget".
	Messages []SubMsg `json:"messages"`
	// attributes for a log event to return over abci interface
	Attributes []EventAttribute `json:"attributes"`
	// custom events (separate from the main one that contains the attributes
	// above)
	Events []Event `json:"events"`
}

// This is the return value for the majority of the ibc handlers.
// That are able to dispatch messages / events on their own,
// but have no meaningful return value to the calling code.
//
// Callbacks that have return values (like receive_packet)
// or that cannot redispatch messages (like the handshake callbacks)
// will use other Response types
type IBCReceiveResult struct {
	Ok  *IBCReceiveResponse `json:"ok,omitempty"`
	Err string              `json:"error,omitempty"`
}

func (r *IBCReceiveResult) SubMessages() []SubMsg {
	if r.Ok != nil {
		return r.Ok.Messages
	}
	return nil
}

// IBCReceiveResponse defines the return value on packet response processing.
// This "success" case should be returned even in application-level errors,
// Where the Acknowledgement bytes contain an encoded error message to be returned to
// the calling chain. (Returning IBCReceiveResult::Err will abort processing of this packet
// and not inform the calling chain).
// This is the counterpart of (IbcReceiveResponse)(https://github.com/CosmWasm/cosmwasm/blob/v0.15.0/packages/std/src/ibc.rs#L247-L267).
type IBCReceiveResponse struct {
	// Acknowledgement is binary encoded data to be returned to calling chain as the acknowledgement.
	// If this field is nil, no acknowledgement must be written. For contracts before CosmWasm 2.0, this
	// was always a non-nil value. See also https://github.com/CosmWasm/cosmwasm/pull/1892.
	Acknowledgement []byte `json:"acknowledgement"`
	// Messages comes directly from the contract and is it's request for action.
	// If the ReplyOn value matches the result, the runtime will invoke this
	// contract's `reply` entry point after execution. Otherwise, this is all
	// "fire and forget".
	Messages   []SubMsg         `json:"messages"`
	Attributes []EventAttribute `json:"attributes"`
	// custom events (separate from the main one that contains the attributes
	// above)
	Events []Event `json:"events"`
}
