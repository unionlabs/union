import { Sui } from "@unionlabs/sdk-sui"
import type { Chain } from "@unionlabs/sdk/schema"
import { Data, Effect, Option } from "effect"
import { Ed25519Keypair } from "@mysten/sui/keypairs/ed25519"

export class NoSuiRpcError extends Data.TaggedError("NoSuiRpcError")<{ chain: Chain }> {}

export const getSuiPublicClient = (chain: Chain) =>
  Effect.gen(function* () {
    const maybeRpc = chain.getRpcUrl("rpc")
    if (Option.isNone(maybeRpc)) {
      return yield* new NoSuiRpcError({ chain })
    }
    const url = maybeRpc.value.toString()

    const layer = Sui.PublicClient.Live({ url })
    const client = yield* Sui.PublicClient.pipe(Effect.provide(layer))
    return client 
  })

export const getSuiWalletClient = (chain: Chain, signer: Ed25519Keypair) =>
  Effect.gen(function* () {
    const maybeRpc = chain.getRpcUrl("rpc")
    if (Option.isNone(maybeRpc)) {
      return yield* new NoSuiRpcError({ chain })
    }
    const url = maybeRpc.value.toString()

    const layer = Sui.WalletClient.Live({ url, signer })
    const wallet = yield* Sui.WalletClient.pipe(Effect.provide(layer))
    return wallet 
  })
