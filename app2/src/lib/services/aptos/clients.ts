import { Data, Effect } from "effect"
import { Aptos, AptosConfig, Network } from "@aptos-labs/ts-sdk"
// import { someWalletLibraryOrBrowserObject } from "some-aptos-wallet-lib"
import type { Chain } from "$lib/schema/chain"

// ─────────────────────────────────────────────────────────────────────────────
//  Errors
// ─────────────────────────────────────────────────────────────────────────────

/** Thrown if the given chain is not recognized as an Aptos chain. */
export class NoAptosChainError extends Data.TaggedError("NoAptosChain")<{
  chain: Chain
}> {}

/** Thrown if creating a public Aptos client fails. */
export class CreatePublicAptosClientError extends Data.TaggedError("CreatePublicAptosClientError")<{
  cause: unknown
}> {}

/** Thrown if creating a wallet-based Aptos client fails. */
export class CreateWalletAptosClientError extends Data.TaggedError("CreateWalletAptosClientError")<{
  cause: unknown
}> {}

export const getPublicClient = (chain: Chain) =>
  Effect.gen(function* () {
    if (chain.rpc_type !== "aptos") {
      throw new NoAptosChainError({ chain })
    }
    const aptosClient = yield* Effect.try({
      try: () => {
        const config = new AptosConfig({
          fullnode: "https://aptos.testnet.bardock.movementlabs.xyz/v1",
          network: Network.TESTNET
        })
        return new Aptos(config)
      },
      catch: err => new CreatePublicAptosClientError({ cause: err })
    })
    return aptosClient
  })

export const getWalletClient = (chain: Chain) =>
  Effect.gen(function* () {
    if (chain.rpc_type !== "aptos") {
      throw new NoAptosChainError({ chain })
    }

    const aptosClient = yield* Effect.try({
      try: () => {
        const config = new AptosConfig({
          fullnode: "https://aptos.testnet.bardock.movementlabs.xyz/v1",
          network: Network.TESTNET
        })
        const aptos = new Aptos(config)
        return aptos
      },
      catch: err => new CreateWalletAptosClientError({ cause: err })
    })

    return aptosClient
  })
