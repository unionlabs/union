import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing"
import { GasPrice } from "@cosmjs/stargate"
import {
  CosmosChannelDestination,
  CosmWasmClientDestination,
  CosmWasmClientSource,
  createCosmWasmClient,
  createSigningCosmWasmClient,
  SigningCosmWasmClientContext,
} from "@unionlabs/sdk/cosmos"
import { CosmosChannelSource } from "@unionlabs/sdk/cosmos"
import { AddressCosmosZkgm } from "@unionlabs/sdk/schema/address"
import { UniversalChainId } from "@unionlabs/sdk/schema/chain"
import { ChannelId } from "@unionlabs/sdk/schema/channel"
import {
  createCosmosToCosmosFungibleAssetOrder,
  Instruction,
  sendInstructionCosmos,
} from "@unionlabs/sdk/ucs03"
import { Effect } from "effect"
import { toHex } from "viem"

// @ts-ignore
BigInt["prototype"].toJSON = function() {
  return this.toString()
}

const MNEMONIC = process.env.MNEMONIC || "memo memo memo"

const SENDER = AddressCosmosZkgm.make(toHex("bbn122ny3mep2l7nhtafpwav2y9e5jrslhekrn8frh"))
const RECEIVER = AddressCosmosZkgm.make(toHex("osmo122ny3mep2l7nhtafpwav2y9e5jrslhekuk9g2u"))
const SOURCE_UCS03_ADDRESS = "bbn1336jj8ertl8h7rdvnz4dh5rqahd09cy0x43guhsxx6xyrztx292q77945h"

const OSMOSIS_RPC = "https://rpc.osmo-test-5.osmosis.chain.kitchen"
const BABYLON_RPC = "https://rpc.bbn-test-5.babylon.chain.kitchen"

// Define token transfers
const TRANSFERS = [
  {
    sender: SENDER,
    receiver: RECEIVER,
    baseToken: "ubbn",
    baseAmount: 123n,
    quoteAmount: 123n,
    sourceChainId: UniversalChainId.make("babylon.bbn-test-5"),
    sourceChannelId: ChannelId.make(5),
  },
] as const

const createBatch = Effect.gen(function*() {
  yield* Effect.log("creating transfer 1")
  const transfer1 = yield* createCosmosToCosmosFungibleAssetOrder(TRANSFERS[0])
  yield* Effect.log("Created transfer", JSON.stringify(transfer1, null, 2))
  return Instruction.Batch.make({ operand: [transfer1] })
}).pipe(Effect.withLogSpan("batch creation"))

const program = Effect.gen(function*() {
  // Create clients and setup
  yield* Effect.log("Transferring from Babylon to Osmosis")

  yield* Effect.log("Creating clients...")
  const cosmWasmSourceClient = yield* createCosmWasmClient(BABYLON_RPC)
  const cosmWasmDestinationClient = yield* createCosmWasmClient(OSMOSIS_RPC)

  // Create a wallet from mnemonic (in a real app, use a secure method to get this)
  yield* Effect.log("Divining wallet...")
  const wallet = yield* Effect.tryPromise(() =>
    DirectSecp256k1HdWallet.fromMnemonic(MNEMONIC, { prefix: "bbn" })
  )

  // Get the first account address
  const [firstAccount] = yield* Effect.tryPromise(() => wallet.getAccounts())
  yield* Effect.log("Got wallet account", firstAccount)

  // Create a signing client
  const signingClient = yield* createSigningCosmWasmClient(
    BABYLON_RPC,
    wallet,
    { gasPrice: GasPrice.fromString("0.025ubbn") },
  )

  yield* Effect.log("Clients created")

  // Main effect: create the batch and send it
  return yield* Effect.gen(function*() {
    yield* Effect.log("Creating batch...")
    const batch = yield* createBatch
    yield* Effect.log("Batch created:", JSON.stringify(batch, null, 2))
    yield* Effect.log("Batch ABI:", Instruction.encodeAbi(batch))

    yield* Effect.log("Sending batch...")
    return yield* sendInstructionCosmos(batch, [{
      denom: "ubbn",
      amount: TRANSFERS[0].baseAmount.toString(),
    }])
  }).pipe(
    Effect.provideService(SigningCosmWasmClientContext, {
      client: signingClient,
      address: firstAccount.address,
    }),
    Effect.provideService(CosmWasmClientSource, { client: cosmWasmSourceClient }),
    Effect.provideService(CosmWasmClientDestination, { client: cosmWasmDestinationClient }),
    Effect.provideService(CosmosChannelDestination, {
      ucs03address: "osmo1336jj8ertl8h7rdvnz4dh5rqahd09cy0x43guhsxx6xyrztx292qs2uecc",
      channelId: 1,
    }),
    Effect.provideService(CosmosChannelSource, {
      ucs03address: SOURCE_UCS03_ADDRESS,
      channelId: 5,
    }),
  )
})

Effect.runPromiseExit(program).then(console.log)
