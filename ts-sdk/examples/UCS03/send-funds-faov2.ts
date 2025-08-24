/**
 * @title Send Funds Holesky → Xion
 * @description Example transfer from Holesky to Xion.
 * @badge ✓:success
 */
/// <reference types="effect" />
/// <reference types="viem" />
// @paths: {"@unionlabs/sdk": ["../ts-sdk/src"], "@unionlabs/sdk/*": ["../ts-sdk/src/*"]}
// @ts-ignore
if (typeof BigInt.prototype.toJSON !== "function") {
  // @ts-ignore
  BigInt.prototype.toJSON = function() {
    return this.toString()
  }
}
// ---cut---
// EVM
import { http, toHex } from "viem"
import { bscTestnet } from "viem/chains"
// Union
import * as Evm from "@unionlabs/sdk/Evm"
import * as Ucs03 from "@unionlabs/sdk/Ucs03"
import * as Ucs05 from "@unionlabs/sdk/Ucs05"
import { Effect, pipe } from "effect"
import { privateKeyToAccount } from "viem/accounts"

const SENDER = Ucs05.AddressEvmZkgm.make("0xfaebe5bf141cc04a3f0598062b98d2df01ab3c4d")
const RECEIVER = Ucs05.AddressCosmosZkgm.make(toHex("bbn122ny3mep2l7nhtafpwav2y9e5jrslhekrn8frh"))

const walletClient = Evm.WalletClient.Live({
  account: privateKeyToAccount(
    "0x..." as const,
  ),
  chain: bscTestnet,
  transport: http("https://rpc.97.bsc.chain.kitchen"),
})

const sourceChannel = Evm.ChannelSource.Live({
  ucs03address: "0x5fbe74a283f7954f10aa04c2edf55578811aeb03",
  channelId: 1,
})

const main = pipe(
  Ucs03.TokenOrderV2.fromOperand([
    SENDER,
    RECEIVER,
    "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE",
    10n,
    1,
    "0x996be231a091877022ccdbf41da6e2f92e097c0ccc9480f8b3c630e5c2b14ff1",
    toHex("bbn1gm8473g2vszxepfyn884trrxtgkyf8572wa4csev5t8hjumja7csnllkkr"),
    10n,
  ]),
  Evm.sendInstruction,
  Effect.provide(walletClient),
  Effect.provide(sourceChannel),
)

// Run main program
Effect.runPromise(main)
  .then(console.log)
  .catch(console.error)
