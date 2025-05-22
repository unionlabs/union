import { getSupabaseClient } from "$lib/dashboard/client"
import { CACHE_VERSION } from "$lib/dashboard/config"
import { SupabaseError } from "$lib/dashboard/errors"
import { submitWalletVerification } from "$lib/dashboard/queries/private"
import { clearLocalStorageCacheEntry } from "$lib/dashboard/services/cache"
import { dashboard } from "$lib/dashboard/stores/user.svelte"
import type { WalletStore } from "$lib/dashboard/stores/wallets.svelte"
import { switchChain } from "$lib/services/transfer-ucs03-evm"
import { getWagmiConfig } from "$lib/wallet/evm/wagmi-config.svelte"
import { extractErrorDetails } from "@unionlabs/sdk/utils"
import { signMessage } from "@wagmi/core"
import { Data, Effect, Option, pipe } from "effect"
import { Siwe } from "ox"
import type { Chain, Hash, WalletClient } from "viem"

export class WalletVerificationError extends Data.TaggedError("WalletVerificationError")<{
  cause: unknown
  operation: "switchChain" | "sign" | "verify" | "update"
}> {}

export type AddEvmWalletState = Data.TaggedEnum<{
  SwitchChain: {
    chain: Chain
  }
  Signing: {
    walletClient: WalletClient
    chain: Chain
  }
  Verifying: {
    address: string
    chain: Chain
    signature: Hash
    message: string
  }
  Updating: {}
}>

export const AddEvmWalletState = Data.taggedEnum<AddEvmWalletState>()
const { Signing, Verifying, Updating } = AddEvmWalletState

export type StateResult = {
  nextState: Option.Option<AddEvmWalletState>
  message: string
  error: Option.Option<Error>
}

const fail = (msg: string, error?: Error): StateResult => ({
  nextState: Option.none(),
  message: msg,
  error: Option.fromNullable(error),
})

const ok = (state: AddEvmWalletState, msg: string): StateResult => ({
  nextState: Option.some(state),
  message: msg,
  error: Option.none(),
})

const complete = (): StateResult => ({
  nextState: Option.none(),
  message: "Wallet added successfully",
  error: Option.none(),
})

export const addEvmWallet = (
  state: AddEvmWalletState,
  walletClient: WalletClient,
): Effect.Effect<StateResult, never, never> => {
  return AddEvmWalletState.$match(state, {
    SwitchChain: ({ chain }) =>
      pipe(
        switchChain(chain),
        Effect.map(() => ok(Signing({ walletClient, chain }), "Chain switched successfully")),
        Effect.catchAll((error) =>
          Effect.succeed(
            fail(
              "Failed to switch chain",
              new WalletVerificationError({
                cause: extractErrorDetails(error),
                operation: "switchChain",
              }),
            ),
          )
        ),
      ),

    Signing: ({ walletClient, chain }) => {
      const address = walletClient.account?.address
      if (!address) {
        return Effect.succeed(fail("No wallet address found"))
      }

      const siweMessage = Siwe.createMessage({
        address: address as `0x${string}`,
        version: "1" as const,
        chainId: chain.id,
        nonce: Siwe.generateNonce(),
        domain: "app.union.build",
        uri: "https://app.union.build/dashboard/wallets",
        statement: "Sign this message to verify wallet ownership.",
      })

      const messageToSign = siweMessage.toString()

      return pipe(
        Effect.tryPromise(() =>
          signMessage(getWagmiConfig(), {
            account: address as `0x${string}`,
            message: messageToSign,
          })
        ),
        Effect.map((signature) =>
          ok(
            Verifying({ address, chain, signature, message: messageToSign }),
            "Signature received. Verifying...",
          )
        ),
        Effect.catchAll((error) =>
          Effect.succeed(
            fail(
              "Failed to sign message. Please try again.",
              new WalletVerificationError({
                cause: extractErrorDetails(error),
                operation: "sign",
              }),
            ),
          )
        ),
      )
    },

    Verifying: ({ address, chain, signature, message }) =>
      pipe(
        getSupabaseClient(),
        Effect.flatMap((client) => Effect.tryPromise(() => client.auth.refreshSession())),
        Effect.flatMap(({ data: { session } }) => {
          if (!session?.user.id) {
            return Effect.fail(
              new SupabaseError({
                cause: "No authenticated user found",
                operation: "authVerification",
              }),
            )
          }

          return submitWalletVerification({
            id: session.user.id,
            address,
            chainId: `evm:${chain.id}`,
            message,
            signature,
            selectedChains: null,
          })
        }),
        Effect.flatMap(() => getSupabaseClient()),
        Effect.flatMap((client) => Effect.tryPromise(() => client.auth.getSession())),
        Effect.flatMap(({ data: { session } }) => {
          if (!session?.user.id) {
            return Effect.fail(
              new WalletVerificationError({
                cause: "No user ID found",
                operation: "verify",
              }),
            )
          }

          return Effect.succeed(ok(Updating(), "Wallet verified. Updating data..."))
        }),
        Effect.catchAll((error) =>
          Effect.succeed(
            fail(
              "Failed to verify wallet",
              new WalletVerificationError({
                cause: extractErrorDetails(error),
                operation: "verify",
              }),
            ),
          )
        ),
      ),

    Updating: () =>
      pipe(
        getSupabaseClient(),
        Effect.flatMap((client) => Effect.tryPromise(() => client.auth.getSession())),
        Effect.flatMap(({ data: { session } }) => {
          if (!session?.user.id) {
            return Effect.fail(
              new WalletVerificationError({
                cause: "No user ID found",
                operation: "update",
              }),
            )
          }

          return pipe(
            clearLocalStorageCacheEntry("wallets", `${CACHE_VERSION}:${session.user.id}`),
            Effect.mapError((error) =>
              new WalletVerificationError({
                cause: extractErrorDetails(error),
                operation: "update",
              })
            ),
            Effect.flatMap(() => Effect.sleep("3 seconds")),
            Effect.flatMap(() =>
              Effect.sync(() => {
                Option.map(dashboard.wallets, (store: WalletStore) => store.refresh())
                return complete()
              })
            ),
          )
        }),
        Effect.catchAll((error) =>
          Effect.succeed(
            fail(
              "Failed to update wallet data",
              new WalletVerificationError({
                cause: extractErrorDetails(error),
                operation: "update",
              }),
            ),
          )
        ),
      ),
  })
}
