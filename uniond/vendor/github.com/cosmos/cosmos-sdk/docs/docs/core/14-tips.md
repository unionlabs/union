---
sidebar_position: 1
---

# Transaction Tips

:::note Synopsis
Transaction tips are a mechanism to pay for transaction fees using another denom than the native fee denom of the chain. They are still in beta, and are not included by default in the SDK.
:::

## Context

In a Cosmos ecosystem where more and more chains are connected via [IBC](https://ibc.cosmos.network/), it happens that users want to perform actions on chains where they don't have native tokens yet. An example would be an Osmosis user who wants to vote on a proposal on the Cosmos Hub, but they don't have ATOMs in their wallet. A solution would be to swap OSMO for ATOM just for voting on this proposal, but that is cumbersome. Cross-chain DeFi project [Emeris](https://emeris.com/) is another use case.

Transaction tips is a new solution for cross-chain transaction fees payment, whereby the transaction initiator signs a transaction without specifying fees, but uses a new `Tip` field. They send this signed transaction to a fee relayer who will choose the transaction fees and broadcast the final transaction, and the SDK provides a mechanism that will transfer the pre-defined `Tip` to the fee payer, to cover for fees.

Assuming we have two chains, A and B, we define the following terms:

* **the tipper**: this is the initiator of the transaction, who wants to execute a `Msg` on chain A, but doesn't have any native chain A tokens, only chain B tokens. In our example above, the tipper is the Osmosis (chain B) user wanting to vote on a Cosmos Hub (chain A) proposal.
* **the fee payer**: this is the party that will relay and broadcast the final transaction on chain A, and has chain A tokens. The tipper doesn't need to trust the feepayer.
* **the target chain**: the chain where the `Msg` is executed, chain A in this case.

## Transaction Tips Flow

The transaction tips flow happens in multiple steps.

1. The tipper sends via IBC some chain B tokens to chain A. These tokens will cover for fees on the target chain A. This means that chain A's bank module holds some IBC tokens under the tipper's address.

2. The tipper drafts a transaction to be executed on the chain A. It can include chain A `Msg`s. However, instead of creating a normal transaction, they create the following `AuxSignerData` document:

	```protobuf reference
	https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/proto/cosmos/tx/v1beta1/tx.proto#L237-L256
	```

	where we have defined `SignDocDirectAux` as:

	```protobuf reference
	https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/proto/cosmos/tx/v1beta1/tx.proto#L67-L97
	```

	where `Tip` is defined as

	```protobuf reference
	https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/proto/cosmos/tx/v1beta1/tx.proto#L226-L235
	```

	Notice that this document doesn't sign over the final chain A fees. Instead, it includes a `Tip` field. It also doesn't include the whole `AuthInfo` object as in `SIGN_MODE_DIRECT`, only the minimum information needed by the tipper

3. The tipper signs the `SignDocDirectAux` document and attaches the signature to the `AuxSignerData`, then sends the signed `AuxSignerData` to the fee payer.

4. From the signed `AuxSignerData` document, the fee payer constructs a transaction, using the following algorithm:

* use as `TxBody` the exact `AuxSignerData.SignDocDirectAux.body_bytes`, to not alter the original intent of the tipper,
* create an `AuthInfo` with:
    * `AuthInfo.Tip` copied from `AuxSignerData.SignDocDirectAux.Tip`,
    * `AuthInfo.Fee` chosen by the fee payer, which should cover for the transaction gas, but also be small enough so that the tip/fee exchange rate is economically interesting for the fee payer,
    * `AuthInfo.SignerInfos` has two signers: the first signer is the tipper, using the public key, sequence and sign mode specified in `AuxSignerData`; and the second signer is the fee payer, using their favorite sign mode,
* a `Signatures` array with two items: the tipper's signature from `AuxSignerData.Sig`, and the final fee payer's signature.

5. Broadcast the final transaction signed by the two parties to the target chain. Once included, the Cosmos SDK will trigger a transfer of the `Tip` specified in the transaction from the tipper address to the fee payer address.

### Fee Payers Market

The benefit of transaction tips for the tipper is clear: there is no need to swap tokens before executing a cross-chain message.

For the fee payer, the benefit is in the tip v.s. fee exchange. Put simply, the fee payer pays the fees of an unknown tipper's transaction, and gets in exchange the tip that the tipper chose. There is an economic incentive for the fee payer to do so only when the tip is greater than the transaction fees, given the exchange rates between the two tokens.

In the future, we imagine a market where fee payers will compete to include transactions from tippers, who on their side will optimize by specifying the lowest tip possible. A number of automated services might spin up to perform transaction gas simulation and exchange rate monitoring to optimize both the tip and fee values in real-time.

### Tipper and Fee Payer Sign Modes

As we mentioned in the flow above, the tipper signs over the `SignDocDirectAux`, and the fee payer signs over the whole final transaction. As such, both parties might use different sign modes.

* The tipper MUST use `SIGN_MODE_DIRECT_AUX` or `SIGN_MODE_LEGACY_AMINO_JSON`. That is because the tipper needs to sign over the body, the tip, but not the other signers' information and not over the fee (which is unknown to the tipper).
* The fee payer MUST use `SIGN_MODE_DIRECT` or `SIGN_MODE_LEGACY_AMINO_JSON`. The fee payer signs over the whole transaction.

For example, if the fee payer signs the whole transaction with `SIGN_MODE_DIRECT_AUX`, it will be rejected by the node, as that would introduce malleability issues (`SIGN_MODE_DIRECT_AUX` doesn't sign over fees).

In both cases, using `SIGN_MODE_LEGACY_AMINO_JSON` is recommended only if hardware wallet signing is needed.

## Enabling Tips on your Chain

The transaction tips functionality is introduced in Cosmos SDK v0.46, so earlier versions do not have support for tips. It is however not included by default in a v0.46 app. Sending a transaction with tips to a chain which didn't enable tips will result in a no-op, i.e. the `tip` field in the transaction will be ignored.

Enabling tips on a chain is done by adding the `TipDecorator` in the posthandler chain:

```go
// HandlerOptions are the options required for constructing a SDK PostHandler which supports tips.
type HandlerOptions struct {
	BankKeeper types.BankKeeper
}

// MyPostHandler returns a posthandler chain with the TipDecorator.
func MyPostHandler(options HandlerOptions) (sdk.AnteHandler, error) {
    if options.BankKeeper == nil {
		return nil, sdkerrors.Wrap(sdkerrors.ErrLogic, "bank keeper is required for posthandler")
	}

	postDecorators := []sdk.AnteDecorator{
		posthandler.NewTipDecorator(options.bankKeeper),
	}

	return sdk.ChainAnteDecorators(postDecorators...), nil
}

func (app *SimApp) setPostHandler() {
	postHandler, err := MyPostHandler(
		HandlerOptions{
			BankKeeper: app.BankKeeper,
		},
	)
	if err != nil {
		panic(err)
	}

	app.SetPostHandler(postHandler)
}
```

Notice that `NewTipDecorator` needs a reference to the BankKeeper, for transferring the tip to the fee payer.

## CLI Usage

The Cosmos SDK also provides some CLI tooling for the transaction tips flow, both for the tipper and for the feepayer.

For the tipper, the CLI `tx` subcommand has two new flags: `--aux` and `--tip`. The `--aux` flag is used to denote that we are creating an `AuxSignerData` instead of a `Tx`, and the `--tip` is used to populate its `Tip` field.

```bash
$ simd tx gov vote 16 yes --from <tipper_address> --aux --tip 50ibcdenom


### Prints the AuxSignerData as JSON:
### {"address":"cosmos1q0ayf5vq6fd2xxrwh30upg05hxdnyw2h5249a2","sign_doc":{"body_bytes":"CosBChwvY29zbW9zLmJhbmsudjFiZXRhMS5Nc2dTZW5kEmsKLWNvc21vczFxMGF5ZjV2cTZmZDJ4eHJ3aDMwdXBnMDVoeGRueXcyaDUyNDlhMhItY29zbW9zMXdlNWoyZXI2MHV5OXF3YzBta3ptdGdtdHA5Z3F5NXY2bjhnZGdlGgsKBXN0YWtlEgIxMA==","public_key":{"@type":"/cosmos.crypto.secp256k1.PubKey","key":"AojOF/1luQ5H/nZDSrE1w3CyzGJhJdQuS7hFX5wAA6uJ"},"chain_id":"","account_number":"0","sequence":"1","tip":{"amount":[{"denom":"ibcdenom","amount":"50"}],"tipper":"cosmos1q0ayf5vq6fd2xxrwh30upg05hxdnyw2h5249a2"}},"mode":"SIGN_MODE_DIRECT_AUX","sig":"v/d/bGq9FGdecs6faMG2t//nRirFTiqwFtUB65M6kh0QdUeM6jg3r8oJX1o17xkoDxJ09EyJiSyvo6fbU7vUxg=="}
```

It is useful to pipe the JSON output to a file, `> aux_signed_tx.json`

For the fee payer, the Cosmos SDK added a `tx aux-to-fee` subcommand to include an `AuxSignerData` into a transaction, add fees to it, and broadcast it.

```bash
$ simd tx aux-to-fee aux_signed_tx.json --from <fee_payer_address> --fees 30atom

### Prints the broadcasted tx response:
### code: 0
### codespace: sdk
### data: ""
### events: []
### gas_used: "0"
### gas_wanted: "0"
### height: "0"
### info: ""
### logs: []
### timestamp: ""
### tx: null
```

Upon completion of the second command, the fee payer's balance will be down the `30atom` fees, and up the `50ibcdenom` tip.

For both commands, the flag `--sign-mode=amino-json` is still available for hardware wallet signing.

## Programmatic Usage

For the tipper, the SDK exposes a new transaction builder, the `AuxTxBuilder`, for generating an `AuxSignerData`. The API of `AuxTxBuilder` is defined [in `client/tx`](https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/client/tx/aux_builder.go#L16), and can be used as follows:

```go
// Note: there's no need to use clientCtx.TxConfig anymore.

bldr := clienttx.NewAuxTxBuilder()
err := bldr.SetMsgs(msgs...)
bldr.SetAddress("cosmos1...")
bldr.SetMemo(...)
bldr.SetTip(...)
bldr.SetPubKey(...)
err := bldr.SetSignMode(...) // DIRECT_AUX or AMINO, or else error
// ... other setters are also available

// Get the bytes to sign.
signBz, err := bldr.GetSignBytes()

// Sign the bz using your favorite method.
sig, err := privKey.sign(signBz)

// Set the signature
bldr.SetSig(sig)

// Get the final auxSignerData to be sent to the fee payer
auxSignerData, err:= bldr.GetAuxSignerData()
```

For the fee payer, the SDK added a new method on the existing `TxBuilder` to import data from an `AuxSignerData`:

```go
// get `auxSignerData` from tipper, see code snippet above.

txBuilder := clientCtx.TxConfig.NewTxBuilder()
err := txBuilder.AddAuxSignerData(auxSignerData)
if err != nil {
	return err
}

// A lot of fields will be populated in txBuilder, such as its Msgs, tip
// memo, etc...

// The fee payer choses the fee to set on the transaction.
txBuilder.SetFeePayer(<fee_payer_address>)
txBuilder.SetFeeAmount(...)
txBuilder.SetGasLimit(...)

// Usual signing code
err = authclient.SignTx(...)
if err != nil {
	return err
}
```
