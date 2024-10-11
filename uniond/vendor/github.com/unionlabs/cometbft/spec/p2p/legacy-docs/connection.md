# P2P Multiplex Connection

## MConnection

`MConnection` is a multiplex connection that supports multiple independent streams
with distinct quality of service guarantees atop a single TCP connection.
Each stream is known as a `Channel` and each `Channel` has a globally unique _byte id_.
Each `Channel` also has a relative priority that determines the quality of service
of the `Channel` compared to other `Channel`s.
The _byte id_ and the relative priorities of each `Channel` are configured upon
initialization of the connection.

The `MConnection` supports three packet types:

- Ping
- Pong
- Msg

### Ping and Pong

The ping and pong messages consist of writing a single byte to the connection; 0x1 and 0x2, respectively.

When we haven't received any messages on an `MConnection` in time `pingTimeout`, we send a ping message.
When a ping is received on the `MConnection`, a pong is sent in response only if there are no other messages
to send and the peer has not sent us too many pings (TODO).

If a pong or message is not received in sufficient time after a ping, the peer is disconnected from.

### Msg

Messages in channels are chopped into smaller `msgPacket`s for multiplexing.

```go
type msgPacket struct {
 ChannelID byte
 EOF       byte // 1 means message ends here.
 Bytes     []byte
}
```

The `msgPacket` is serialized using [Proto3](https://developers.google.com/protocol-buffers/docs/proto3).
The received `Bytes` of a sequential set of packets are appended together
until a packet with `EOF=1` is received, then the complete serialized message
is returned for processing by the `onReceive` function of the corresponding channel.

### Multiplexing

Messages are sent from a single `sendRoutine`, which loops over a select statement and results in the sending
of a ping, a pong, or a batch of data messages. The batch of data messages may include messages from multiple channels.
Message bytes are queued for sending in their respective channel, with each channel holding one unsent message at a time.
Messages are chosen for a batch one at a time from the channel with the lowest ratio of recently sent bytes to channel priority.

## Sending Messages

There are two methods for sending messages:

```go
func (m MConnection) Send(chID byte, msg interface{}) bool {}
func (m MConnection) TrySend(chID byte, msg interface{}) bool {}
```

`Send(chID, msg)` is a blocking call that waits until `msg` is successfully queued
for the channel with the given id byte `chID`. The message `msg` is serialized
using protobuf marshalling.

`TrySend(chID, msg)` is a nonblocking call that queues the message msg in the channel
with the given id byte chID if the queue is not full; otherwise it returns false immediately.

`Send()` and `TrySend()` are also exposed for each `Peer`.

## Peer

Each peer has one `MConnection` instance, and includes other information such as whether the connection
was outbound, whether the connection should be recreated if it closes, various identity information about the node,
and other higher level thread-safe data used by the reactors.

## Switch/Reactor

The `Switch` handles peer connections and exposes an API to receive incoming messages
on `Reactors`. Each `Reactor` is responsible for handling incoming messages of one
or more `Channels`. So while sending outgoing messages is typically performed on the peer,
incoming messages are received on the reactor.

```go
// Declare a MyReactor reactor that handles messages on MyChannelID.
type MyReactor struct{}

func (reactor MyReactor) GetChannels() []*ChannelDescriptor {
    return []*ChannelDescriptor{ChannelDescriptor{ID:MyChannelID, Priority: 1}}
}

func (reactor MyReactor) Receive(chID byte, peer *Peer, msgBytes []byte) {
    r, n, err := bytes.NewBuffer(msgBytes), new(int64), new(error)
    msgString := ReadString(r, n, err)
    fmt.Println(msgString)
}

// Other Reactor methods omitted for brevity
...

switch := NewSwitch([]Reactor{MyReactor{}})

...

// Send a random message to all outbound connections
for _, peer := range switch.Peers().Copy() {
    if peer.IsOutbound() {
        peer.Send(MyChannelID, "Here's a random message")
    }
}
```
