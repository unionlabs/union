# ADR 024: SignBytes and validator types in privval

## Context

Currently, the messages exchanged between tendermint and a (potentially remote) signer/validator, 
namely votes, proposals, and heartbeats, are encoded as a JSON string 
(e.g., via `Vote.SignBytes(...)`) and then 
signed . JSON encoding is sub-optimal for both, hardware wallets 
and for usage in ethereum smart contracts. Both is laid down in detail in [issue#1622].  

Also, there are currently no differences between sign-request and -replies. Also, there is no possibility 
for a remote signer to include an error code or message in case something went wrong.
The messages exchanged between tendermint and a remote signer currently live in 
[privval/socket.go] and encapsulate the corresponding types in [types].


[privval/socket.go]: https://github.com/tendermint/tendermint/blob/d419fffe18531317c28c29a292ad7d253f6cafdf/privval/socket.go#L496-L502
[issue#1622]: https://github.com/tendermint/tendermint/issues/1622
[types]: https://github.com/tendermint/tendermint/tree/main/types
 

## Decision

- restructure vote, proposal, and heartbeat such that their encoding is easily parseable by 
hardware devices and smart contracts using a  binary encoding format ([amino] in this case)
- split up the messages exchanged between tendermint and remote signers into requests and 
responses (see details below)
- include an error type in responses

### Overview
```
+--------------+                      +----------------+
|              |     SignXRequest     |                |
|Remote signer |<---------------------+  tendermint    |
| (e.g. KMS)   |                      |                |
|              +--------------------->|                |
+--------------+    SignedXReply      +----------------+


SignXRequest {
    x: X
}

SignedXReply {
    x: X
  sig: Signature // []byte
  err: Error{ 
    code: int
    desc: string
  }
}
```

TODO: Alternatively, the type `X` might directly include the signature. A lot of places expect a vote with a 
signature and do not necessarily deal with "Replies".
Still exploring what would work best here. 
This would look like (exemplified using X = Vote):
```
Vote {
    // all fields besides signature
}

SignedVote {
 Vote Vote
 Signature []byte
}

SignVoteRequest {
   Vote Vote
}

SignedVoteReply {
    Vote SignedVote
    Err  Error
}
```

**Note:** There was a related discussion around including a fingerprint of, or, the whole public-key 
into each sign-request to tell the signer which corresponding private-key to 
use to sign the message. This is particularly relevant in the context of the KMS
but is currently not considered in this ADR. 


[amino]: https://github.com/tendermint/go-amino/

### Vote

As explained in [issue#1622] `Vote` will be changed to contain the following fields 
(notation in protobuf-like syntax for easy readability):

```proto
// vanilla protobuf / amino encoded
message Vote {
    Version       fixed32                      
    Height        sfixed64       
    Round         sfixed32
    VoteType      fixed32
    Timestamp     Timestamp         // << using protobuf definition
    BlockID       BlockID           // << as already defined 
    ChainID       string            // at the end because length could vary a lot
}

// this is an amino registered type; like currently privval.SignVoteMsg: 
// registered with "tendermint/socketpv/SignVoteRequest"
message SignVoteRequest {
   Vote vote
}

//  amino registered type
// registered with "tendermint/socketpv/SignedVoteReply"
message SignedVoteReply { 
   Vote      Vote
   Signature Signature 
   Err       Error
}

// we will use this type everywhere below
message Error {
  Type        uint  // error code
  Description string  // optional description
}

```

The `ChainID` gets moved into the vote message directly. Previously, it was injected 
using the [Signable] interface method `SignBytes(chainID string) []byte`. Also, the 
signature won't be included directly, only in the corresponding `SignedVoteReply` message.

[Signable]: https://github.com/tendermint/tendermint/blob/d419fffe18531317c28c29a292ad7d253f6cafdf/types/signable.go#L9-L11
 
### Proposal

```proto
// vanilla protobuf / amino encoded
message Proposal {                      
    Height            sfixed64       
    Round             sfixed32
    Timestamp         Timestamp         // << using protobuf definition
    BlockPartsHeader  PartSetHeader     // as already defined
    POLRound          sfixed32
    POLBlockID        BlockID           // << as already defined    
}
 
// amino registered with "tendermint/socketpv/SignProposalRequest"
message SignProposalRequest {
   Proposal proposal
}

// amino registered with "tendermint/socketpv/SignProposalReply"
message SignProposalReply { 
   Prop   Proposal
   Sig    Signature 
   Err    Error     // as defined above
}
```

### Heartbeat

**TODO**: clarify if heartbeat also needs a fixed offset and update the fields accordingly: 

```proto
message Heartbeat {
	ValidatorAddress Address 
	ValidatorIndex   int     
	Height           int64   
	Round            int     
	Sequence         int     
}
// amino registered with "tendermint/socketpv/SignHeartbeatRequest"
message SignHeartbeatRequest {
   Hb Heartbeat
}

// amino registered with "tendermint/socketpv/SignHeartbeatReply"
message SignHeartbeatReply { 
   Hb     Heartbeat
   Sig    Signature 
   Err    Error     // as defined above
}

```

## PubKey

TBA -  this needs further thoughts: e.g. what todo like in the case of the KMS which holds
several keys? How does it know with which key to reply?

## SignBytes
`SignBytes` will not require a `ChainID` parameter:

```golang
type Signable interface {
	SignBytes() []byte
}

```
And the implementation for vote, heartbeat, proposal will look like:
```golang
// type T is one of vote, sign, proposal
func (tp *T) SignBytes() []byte {
	bz, err := cdc.MarshalBinary(tp)
	if err != nil {
		panic(err)
	}
	return bz
}
```

## Status

Partially Accepted

## Consequences



### Positive

The most relevant positive effect is that the signing bytes can easily be parsed by a 
hardware module and a smart contract. Besides that:
 
- clearer separation between requests and responses
- added error messages enable better error handling 


### Negative

- relatively huge change / refactoring touching quite some code
- lot's of places assume a `Vote` with a signature included -> they will need to 
- need to modify some interfaces 

### Neutral

not even the swiss are neutral
