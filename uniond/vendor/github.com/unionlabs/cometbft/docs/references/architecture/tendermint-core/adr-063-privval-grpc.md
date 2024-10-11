# ADR 063: Privval gRPC

## Changelog

- 23/11/2020: Initial Version (@marbar3778)

## Context

Validators use remote signers to help secure their keys. This system is Tendermint's recommended way to secure validators, but the path to integration with Tendermint's private validator client is plagued with custom protocols. 

Tendermint uses its own custom secure connection protocol (`SecretConnection`) and a raw tcp/unix socket connection protocol. The secure connection protocol until recently was exposed to man in the middle attacks and can take longer to integrate if not using Golang. The raw tcp connection protocol is less custom, but has been causing minute issues with users. 

Migrating Tendermint's private validator client to a widely adopted protocol, gRPC, will ease the current maintenance and integration burden experienced with the current protocol. 

## Decision

After discussing with multiple stake holders, [gRPC](https://grpc.io/) was decided on to replace the current private validator protocol. gRPC is a widely adopted protocol in the micro-service and cloud infrastructure world. gRPC uses [protocol-buffers](https://developers.google.com/protocol-buffers) to describe its services, providing a language agnostic implementation. Tendermint uses protobuf for on disk and over the wire encoding already making the integration with gRPC simpler. 

## Alternative Approaches

- JSON-RPC: We did not consider JSON-RPC because Tendermint uses protobuf extensively making gRPC a natural choice.

## Detailed Design

With the recent integration of [Protobuf](https://developers.google.com/protocol-buffers) into Tendermint the needed changes to migrate from the current private validator protocol to gRPC is not large. 

The [service definition](https://grpc.io/docs/what-is-grpc/core-concepts/#service-definition) for gRPC will be defined as:

```proto
  service PrivValidatorAPI {
    rpc GetPubKey(tendermint.proto.privval.PubKeyRequest) returns (tendermint.proto.privval.PubKeyResponse);
    rpc SignVote(tendermint.proto.privval.SignVoteRequest) returns (tendermint.proto.privval.SignedVoteResponse);
    rpc SignProposal(tendermint.proto.privval.SignProposalRequest) returns (tendermint.proto.privval.SignedProposalResponse);

    message PubKeyRequest {
    string chain_id = 1;
  }

  // PubKeyResponse is a response message containing the public key.
  message PubKeyResponse {
    tendermint.crypto.PublicKey pub_key = 1 [(gogoproto.nullable) = false];
  }

  // SignVoteRequest is a request to sign a vote
  message SignVoteRequest {
    tendermint.types.Vote vote     = 1;
    string                chain_id = 2;
  }

  // SignedVoteResponse is a response containing a signed vote or an error
  message SignedVoteResponse {
    tendermint.types.Vote vote  = 1 [(gogoproto.nullable) = false];
  }

  // SignProposalRequest is a request to sign a proposal
  message SignProposalRequest {
    tendermint.types.Proposal proposal = 1;
    string                    chain_id = 2;
  }

  // SignedProposalResponse is response containing a signed proposal or an error
  message SignedProposalResponse {
    tendermint.types.Proposal proposal = 1 [(gogoproto.nullable) = false];
  }
}
```

> Note: Remote Singer errors are removed in favor of [grpc status error codes](https://grpc.io/docs/guides/error/).

In previous versions of the remote signer, Tendermint acted as the server and the remote signer as the client. In this process the client established a long lived connection providing a way for the server to make requests to the client. In the new version it has been simplified. Tendermint is the client and the remote signer is the server. This follows client and server architecture and simplifies the previous protocol.

#### Keep Alive

If you have worked on the private validator system you will see that we are removing the `PingRequest` and `PingResponse` messages. These messages were used to create functionality which kept the connection alive. With gRPC there is a [keep alive feature](https://github.com/grpc/grpc/blob/master/doc/keepalive.md) that will be added along side the integration to provide the same functionality. 

#### Metrics

Remote signers are crucial to operating secure and consistently up Validators. In the past there were no metrics to tell the operator if something is wrong other than the node not signing. Integrating metrics into the client and provided server will be done with [prometheus](https://github.com/grpc-ecosystem/go-grpc-prometheus). This will be integrated into node's prometheus export for node operators. 

#### Security

[TLS](https://en.wikipedia.org/wiki/Transport_Layer_Security) is widely adopted with the use of gRPC. There are various forms of TLS (one-way & two-way). One way is the client identifying who the server is, while two way is both parties identifying the other. For Tendermint's use case having both parties identifying each other provides adds an extra layer of security. This requires users to generate both client and server certificates for a TLS connection. 

An insecure option will be provided for users who do not wish to secure the connection.

#### Upgrade Path

This is a largely breaking change for validator operators. The optimal upgrade path would be to release gRPC in a minor release, allow key management systems to migrate to the new protocol. In the next major release the current system (raw tcp/unix) is removed. This allows users to migrate to the new system and not have to coordinate upgrading the key management system alongside a network upgrade. 

The upgrade of [tmkms](https://github.com/iqlusioninc/tmkms) will be coordinated with Iqlusion. They will be able to make the necessary upgrades to allow users to migrate to gRPC from the current protocol. 

## Status

Accepted (tracked in
[\#9256](https://github.com/tendermint/tendermint/issues/9256))

### Positive

- Use an adopted standard for secure communication. (TLS)
- Use an adopted communication protocol. (gRPC)
- Requests are multiplexed onto the tcp connection. (http/2)
- Language agnostic service definition.

### Negative

- Users will need to generate certificates to use TLS. (Added step)
- Users will need to find a supported gRPC supported key management system

### Neutral
