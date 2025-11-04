import type { Ed25519Keypair } from "@mysten/sui/keypairs/ed25519"
import { Sui } from "@unionlabs/sdk-sui"
import type { Chain } from "@unionlabs/sdk/schema"
import { Effect, pipe } from "effect"

export class SuiSwitchChainError extends Error {
  constructor(readonly cause: unknown, readonly chainId?: string) {
    super(`Sui switch network failed: ${String(cause)}`)
  }
}
export class SuiWalletNotProvidedError extends Error {
  constructor() {
    super("Sui signer (Ed25519Keypair) not provided")
  }
}

type SwitchChainSuccess = {
  success: true
  rpcUrl: string
  publicClient: Sui.Sui.PublicClient
  walletClient: Sui.Sui.WalletClient
}

export const SwitchChain = (chain: Chain, signer: Ed25519Keypair) =>
  Effect.gen(function*() {
    if (!signer) {
      return yield* Effect.fail(new SuiWalletNotProvidedError())
    }

    const rpcUrl = yield* chain.getRpcUrl("rpc").pipe(
      Effect.mapError((cause) => new SuiSwitchChainError(cause, chain.universal_chain_id)),
    )

    const publicLayer = Sui.PublicClient.Live({ url: rpcUrl })
    const walletLayer = Sui.WalletClient.Live({ url: rpcUrl, signer: signer })

    const { pub, wal } = yield* Effect.all({
      pub: Sui.PublicClient,
      wal: Sui.WalletClient,
    }).pipe(Effect.provide(publicLayer), Effect.provide(walletLayer))

    yield* Effect.tryPromise({
      try: async () => pub.client.getReferenceGasPrice(),
      catch: (cause) => new SuiSwitchChainError(cause, chain.universal_chain_id),
    })

    yield* Effect.sleep("1.5 seconds")

    return {
      success: true,
      rpcUrl,
      publicClient: pub,
      walletClient: wal,
    } satisfies SwitchChainSuccess
  })
