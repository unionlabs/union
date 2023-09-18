---
sidebar_position: 1
---

# Generating, Signing and Broadcasting Transactions

:::note Synopsis
This document describes how to generate an (unsigned) transaction, signing it (with one or multiple keys), and broadcasting it to the network.
:::

## Using the CLI

The easiest way to send transactions is using the CLI, as we have seen in the previous page when [interacting with a node](./02-interact-node.md#using-the-cli). For example, running the following command

```bash
simd tx bank send $MY_VALIDATOR_ADDRESS $RECIPIENT 1000stake --chain-id my-test-chain --keyring-backend test
```

will run the following steps:

* generate a transaction with one `Msg` (`x/bank`'s `MsgSend`), and print the generated transaction to the console.
* ask the user for confirmation to send the transaction from the `$MY_VALIDATOR_ADDRESS` account.
* fetch `$MY_VALIDATOR_ADDRESS` from the keyring. This is possible because we have [set up the CLI's keyring](./00-keyring.md) in a previous step.
* sign the generated transaction with the keyring's account.
* broadcast the signed transaction to the network. This is possible because the CLI connects to the node's CometBFT RPC endpoint.

The CLI bundles all the necessary steps into a simple-to-use user experience. However, it's possible to run all the steps individually too.

### Generating a Transaction

Generating a transaction can simply be done by appending the `--generate-only` flag on any `tx` command, e.g.:

```bash
simd tx bank send $MY_VALIDATOR_ADDRESS $RECIPIENT 1000stake --chain-id my-test-chain --generate-only
```

This will output the unsigned transaction as JSON in the console. We can also save the unsigned transaction to a file (to be passed around between signers more easily) by appending `> unsigned_tx.json` to the above command.

### Signing a Transaction

Signing a transaction using the CLI requires the unsigned transaction to be saved in a file. Let's assume the unsigned transaction is in a file called `unsigned_tx.json` in the current directory (see previous paragraph on how to do that). Then, simply run the following command:

```bash
simd tx sign unsigned_tx.json --chain-id my-test-chain --keyring-backend test --from $MY_VALIDATOR_ADDRESS
```

This command will decode the unsigned transaction and sign it with `SIGN_MODE_DIRECT` with `$MY_VALIDATOR_ADDRESS`'s key, which we already set up in the keyring. The signed transaction will be output as JSON to the console, and, as above, we can save it to a file by appending `--output-document signed_tx.json`.

Some useful flags to consider in the `tx sign` command:

* `--sign-mode`: you may use `amino-json` to sign the transaction using `SIGN_MODE_LEGACY_AMINO_JSON`,
* `--offline`: sign in offline mode. This means that the `tx sign` command doesn't connect to the node to retrieve the signer's account number and sequence, both needed for signing. In this case, you must manually supply the `--account-number` and `--sequence` flags. This is useful for offline signing, i.e. signing in a secure environment which doesn't have access to the internet.

#### Signing with Multiple Signers

:::warning
Please note that signing a transaction with multiple signers or with a multisig account, where at least one signer uses `SIGN_MODE_DIRECT`, is not yet possible. You may follow [this Github issue](https://github.com/cosmos/cosmos-sdk/issues/8141) for more info.
:::

Signing with multiple signers is done with the `tx multisign` command. This command assumes that all signers use `SIGN_MODE_LEGACY_AMINO_JSON`. The flow is similar to the `tx sign` command flow, but instead of signing an unsigned transaction file, each signer signs the file signed by previous signer(s). The `tx multisign` command will append signatures to the existing transactions. It is important that signers sign the transaction **in the same order** as given by the transaction, which is retrievable using the `GetSigners()` method.

For example, starting with the `unsigned_tx.json`, and assuming the transaction has 4 signers, we would run:

```bash
# Let signer1 sign the unsigned tx.
simd tx multisign unsigned_tx.json signer_key_1 --chain-id my-test-chain --keyring-backend test > partial_tx_1.json
# Now signer1 will send the partial_tx_1.json to the signer2.
# Signer2 appends their signature:
simd tx multisign partial_tx_1.json signer_key_2 --chain-id my-test-chain --keyring-backend test > partial_tx_2.json
# Signer2 sends the partial_tx_2.json file to signer3, and signer3 can append his signature:
simd tx multisign partial_tx_2.json signer_key_3 --chain-id my-test-chain --keyring-backend test > partial_tx_3.json
```

### Broadcasting a Transaction

Broadcasting a transaction is done using the following command:

```bash
simd tx broadcast tx_signed.json
```

You may optionally pass the `--broadcast-mode` flag to specify which response to receive from the node:

* `sync`: the CLI waits for a CheckTx execution response only.
* `async`: the CLI returns immediately (transaction might fail).

### Encoding a Transaction

In order to broadcast a transaction using the gRPC or REST endpoints, the transaction will need to be encoded first. This can be done using the CLI.

Encoding a transaction is done using the following command:

```bash
simd tx encode tx_signed.json
```

This will read the transaction from the file, serialize it using Protobuf, and output the transaction bytes as base64 in the console.

### Decoding a Transaction

The CLI can also be used to decode transaction bytes.

Decoding a transaction is done using the following command:

```bash
simd tx decode [protobuf-byte-string]
```

This will decode the transaction bytes and output the transaction as JSON in the console. You can also save the transaction to a file by appending `> tx.json` to the above command.

## Programmatically with Go

It is possible to manipulate transactions programmatically via Go using the Cosmos SDK's `TxBuilder` interface.

### Generating a Transaction

Before generating a transaction, a new instance of a `TxBuilder` needs to be created. Since the Cosmos SDK supports both Amino and Protobuf transactions, the first step would be to decide which encoding scheme to use. All the subsequent steps remain unchanged, whether you're using Amino or Protobuf, as `TxBuilder` abstracts the encoding mechanisms. In the following snippet, we will use Protobuf.

```go
import (
	"github.com/cosmos/cosmos-sdk/simapp"
)

func sendTx() error {
    // Choose your codec: Amino or Protobuf. Here, we use Protobuf, given by the following function.
    app := simapp.NewSimApp(...)

    // Create a new TxBuilder.
    txBuilder := app.TxConfig().NewTxBuilder()

    // --snip--
}
```

We can also set up some keys and addresses that will send and receive the transactions. Here, for the purpose of the tutorial, we will be using some dummy data to create keys.

```go
import (
	"github.com/cosmos/cosmos-sdk/testutil/testdata"
)

priv1, _, addr1 := testdata.KeyTestPubAddr()
priv2, _, addr2 := testdata.KeyTestPubAddr()
priv3, _, addr3 := testdata.KeyTestPubAddr()
```

Populating the `TxBuilder` can be done via its methods:

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/client/tx_config.go#L33-L50
```

```go
import (
	banktypes "github.com/cosmos/cosmos-sdk/x/bank/types"
)

func sendTx() error {
    // --snip--

    // Define two x/bank MsgSend messages:
    // - from addr1 to addr3,
    // - from addr2 to addr3.
    // This means that the transactions needs two signers: addr1 and addr2.
    msg1 := banktypes.NewMsgSend(addr1, addr3, types.NewCoins(types.NewInt64Coin("atom", 12)))
    msg2 := banktypes.NewMsgSend(addr2, addr3, types.NewCoins(types.NewInt64Coin("atom", 34)))

    err := txBuilder.SetMsgs(msg1, msg2)
    if err != nil {
        return err
    }

    txBuilder.SetGasLimit(...)
    txBuilder.SetFeeAmount(...)
    txBuilder.SetMemo(...)
    txBuilder.SetTimeoutHeight(...)
}
```

At this point, `TxBuilder`'s underlying transaction is ready to be signed.

### Signing a Transaction

We set encoding config to use Protobuf, which will use `SIGN_MODE_DIRECT` by default. As per [ADR-020](https://github.com/cosmos/cosmos-sdk/blob/main/docs/architecture/adr-020-protobuf-transaction-encoding.md), each signer needs to sign the `SignerInfo`s of all other signers. This means that we need to perform two steps sequentially:

* for each signer, populate the signer's `SignerInfo` inside `TxBuilder`,
* once all `SignerInfo`s are populated, for each signer, sign the `SignDoc` (the payload to be signed).

In the current `TxBuilder`'s API, both steps are done using the same method: `SetSignatures()`. The current API requires us to first perform a round of `SetSignatures()` _with empty signatures_, only to populate `SignerInfo`s, and a second round of `SetSignatures()` to actually sign the correct payload.

```go
import (
    cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	"github.com/cosmos/cosmos-sdk/types/tx/signing"
	xauthsigning "github.com/cosmos/cosmos-sdk/x/auth/signing"
)

func sendTx() error {
    // --snip--

    privs := []cryptotypes.PrivKey{priv1, priv2}
    accNums:= []uint64{..., ...} // The accounts' account numbers
    accSeqs:= []uint64{..., ...} // The accounts' sequence numbers

    // First round: we gather all the signer infos. We use the "set empty
    // signature" hack to do that.
    var sigsV2 []signing.SignatureV2
    for i, priv := range privs {
        sigV2 := signing.SignatureV2{
            PubKey: priv.PubKey(),
            Data: &signing.SingleSignatureData{
                SignMode:  encCfg.TxConfig.SignModeHandler().DefaultMode(),
                Signature: nil,
            },
            Sequence: accSeqs[i],
        }

        sigsV2 = append(sigsV2, sigV2)
    }
    err := txBuilder.SetSignatures(sigsV2...)
    if err != nil {
        return err
    }

    // Second round: all signer infos are set, so each signer can sign.
    sigsV2 = []signing.SignatureV2{}
    for i, priv := range privs {
        signerData := xauthsigning.SignerData{
            ChainID:       chainID,
            AccountNumber: accNums[i],
            Sequence:      accSeqs[i],
        }
        sigV2, err := tx.SignWithPrivKey(
            encCfg.TxConfig.SignModeHandler().DefaultMode(), signerData,
            txBuilder, priv, encCfg.TxConfig, accSeqs[i])
        if err != nil {
            return nil, err
        }

        sigsV2 = append(sigsV2, sigV2)
    }
    err = txBuilder.SetSignatures(sigsV2...)
    if err != nil {
        return err
    }
}
```

The `TxBuilder` is now correctly populated. To print it, you can use the `TxConfig` interface from the initial encoding config `encCfg`:

```go
func sendTx() error {
    // --snip--

    // Generated Protobuf-encoded bytes.
    txBytes, err := encCfg.TxConfig.TxEncoder()(txBuilder.GetTx())
    if err != nil {
        return err
    }

    // Generate a JSON string.
    txJSONBytes, err := encCfg.TxConfig.TxJSONEncoder()(txBuilder.GetTx())
    if err != nil {
        return err
    }
    txJSON := string(txJSONBytes)
}
```

### Broadcasting a Transaction

The preferred way to broadcast a transaction is to use gRPC, though using REST (via `gRPC-gateway`) or the CometBFT RPC is also posible. An overview of the differences between these methods is exposed [here](../core/06-grpc_rest.md). For this tutorial, we will only describe the gRPC method.

```go
import (
    "context"
    "fmt"

	"google.golang.org/grpc"

	"github.com/cosmos/cosmos-sdk/types/tx"
)

func sendTx(ctx context.Context) error {
    // --snip--

    // Create a connection to the gRPC server.
    grpcConn := grpc.Dial(
        "127.0.0.1:9090", // Or your gRPC server address.
        grpc.WithInsecure(), // The Cosmos SDK doesn't support any transport security mechanism.
    )
    defer grpcConn.Close()

    // Broadcast the tx via gRPC. We create a new client for the Protobuf Tx
    // service.
    txClient := tx.NewServiceClient(grpcConn)
    // We then call the BroadcastTx method on this client.
    grpcRes, err := txClient.BroadcastTx(
        ctx,
        &tx.BroadcastTxRequest{
            Mode:    tx.BroadcastMode_BROADCAST_MODE_SYNC,
            TxBytes: txBytes, // Proto-binary of the signed transaction, see previous step.
        },
    )
    if err != nil {
        return err
    }

    fmt.Println(grpcRes.TxResponse.Code) // Should be `0` if the tx is successful

    return nil
}
```

#### Simulating a Transaction

Before broadcasting a transaction, we sometimes may want to dry-run the transaction, to estimate some information about the transaction without actually committing it. This is called simulating a transaction, and can be done as follows:

```go
import (
	"context"
	"fmt"
	"testing"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/types/tx"
	authtx "github.com/cosmos/cosmos-sdk/x/auth/tx"
)

func simulateTx() error {
    // --snip--

    // Simulate the tx via gRPC. We create a new client for the Protobuf Tx
    // service.
    txClient := tx.NewServiceClient(grpcConn)
    txBytes := /* Fill in with your signed transaction bytes. */

    // We then call the Simulate method on this client.
    grpcRes, err := txClient.Simulate(
        context.Background(),
        &tx.SimulateRequest{
            TxBytes: txBytes,
        },
    )
    if err != nil {
        return err
    }

    fmt.Println(grpcRes.GasInfo) // Prints estimated gas used.

    return nil
}
```

## Using gRPC

It is not possible to generate or sign a transaction using gRPC, only to broadcast one. In order to broadcast a transaction using gRPC, you will need to generate, sign, and encode the transaction using either the CLI or programmatically with Go.

### Broadcasting a Transaction

Broadcasting a transaction using the gRPC endpoint can be done by sending a `BroadcastTx` request as follows, where the `txBytes` are the protobuf-encoded bytes of a signed transaction:

```bash
grpcurl -plaintext \
    -d '{"tx_bytes":"{{txBytes}}","mode":"BROADCAST_MODE_SYNC"}' \
    localhost:9090 \
    cosmos.tx.v1beta1.Service/BroadcastTx
```

## Using REST

It is not possible to generate or sign a transaction using REST, only to broadcast one. In order to broadcast a transaction using REST, you will need to generate, sign, and encode the transaction using either the CLI or programmatically with Go.

### Broadcasting a Transaction

Broadcasting a transaction using the REST endpoint (served by `gRPC-gateway`) can be done by sending a POST request as follows, where the `txBytes` are the protobuf-encoded bytes of a signed transaction:

```bash
curl -X POST \
    -H "Content-Type: application/json" \
    -d'{"tx_bytes":"{{txBytes}}","mode":"BROADCAST_MODE_SYNC"}' \
    localhost:1317/cosmos/tx/v1beta1/txs
```

## Using CosmJS (JavaScript & TypeScript)

CosmJS aims to build client libraries in JavaScript that can be embedded in web applications. Please see [https://cosmos.github.io/cosmjs](https://cosmos.github.io/cosmjs) for more information. As of January 2021, CosmJS documentation is still work in progress.
