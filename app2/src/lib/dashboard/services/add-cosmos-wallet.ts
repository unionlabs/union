import { CACHE_VERSION } from "$lib/dashboard/config"
import { SupabaseError } from "$lib/dashboard/errors"
import { submitWalletVerification } from "$lib/dashboard/queries/private"
import { clearLocalStorageCacheEntry } from "$lib/dashboard/services/cache"
import { dashboard } from "$lib/dashboard/stores/user.svelte"
import type { WalletStore } from "$lib/dashboard/stores/wallets.svelte"
import { switchChain } from "$lib/services/transfer-ucs03-cosmos"
import { cosmosStore, type CosmosWalletId } from "$lib/wallet/cosmos"
import { Chain } from "@unionlabs/sdk/schema"
import { extractErrorDetails } from "@unionlabs/sdk/utils"
import { Data, Effect, Option, pipe } from "effect"
import type { Hash } from "viem"
import { SupabaseClient } from "../client"

const allegianceMessage =
  "I'm signing this message to prove account ownership and to pledge allegiance to zkgm."

export class WalletVerificationError extends Data.TaggedError("WalletVerificationError")<{
  cause: unknown
  operation: "switchChain" | "sign" | "verify" | "update"
}> {}

export type AddCosmosWalletState = Data.TaggedEnum<{
  SwitchChain: { chain: Chain }
  Signing: { chain: Chain }
  Verifying: { address: string; chain: Chain; signature: Hash; message: string }
  Updating: {}
}>

export const AddCosmosWalletState = Data.taggedEnum<AddCosmosWalletState>()
const { Signing, Verifying, Updating } = AddCosmosWalletState

export type StateResult = {
  nextState: Option.Option<AddCosmosWalletState>
  message: string
  error: Option.Option<Error>
}

const fail = (msg: string, error?: Error): StateResult => ({
  nextState: Option.none(),
  message: msg,
  error: Option.fromNullable(error),
})

const ok = (state: AddCosmosWalletState, msg: string): StateResult => ({
  nextState: Option.some(state),
  message: msg,
  error: Option.none(),
})

const complete = (): StateResult => ({
  nextState: Option.none(),
  message: "Wallet added successfully",
  error: Option.none(),
})

export const addCosmosWallet = (
  state: AddCosmosWalletState,
  selectedChains: Array<string | null>,
): Effect.Effect<StateResult, never, SupabaseClient> => {
  return AddCosmosWalletState.$match(state, {
    SwitchChain: ({ chain }) =>
      pipe(
        switchChain(chain),
        Effect.map(() => ok(Signing({ chain }), "Chain switched successfully")),
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

    Signing: ({ chain }) => {
      const connectedWalletId = cosmosStore.connectedWallet

      if (!connectedWalletId) {
        return Effect.succeed(
          fail(
            "No Cosmos wallet selected. Please connect a wallet.",
            new WalletVerificationError({
              cause: "No connected Cosmos wallet ID",
              operation: "sign",
            }),
          ),
        )
      }

      const wallet = window[connectedWalletId as CosmosWalletId]

      if (!wallet) {
        return Effect.succeed(
          fail(
            `Selected Cosmos wallet (${connectedWalletId}) not found. Please try reconnecting.`,
            new WalletVerificationError({
              cause: `Wallet instance for ${connectedWalletId} not found on window`,
              operation: "sign",
            }),
          ),
        )
      }

      return pipe(
        Effect.tryPromise(async () => {
          const walletKey = await wallet.getKey("union-testnet-9")
          const address = walletKey.bech32Address
          const signature = await wallet.signArbitrary(
            "union-testnet-9",
            walletKey.bech32Address,
            allegianceMessage,
          )
          return { address, signature }
        }),
        Effect.map(({ address, signature }) =>
          ok(
            Verifying({ address, chain, signature, message: allegianceMessage }),
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
        SupabaseClient,
        Effect.andThen((client) =>
          pipe(
            Effect.tryPromise(() => client.auth.refreshSession()),
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
                chainId: `cosmos:${chain.chain_id}`,
                message,
                signature: JSON.stringify(signature),
                selectedChains,
              })
            }),
            Effect.flatMap((response) => {
              if (Option.isNone(response)) {
                return Effect.fail(
                  new WalletVerificationError({
                    cause: "Wallet verification failed",
                    operation: "verify",
                  }),
                )
              }

              return Effect.succeed(response.value)
            }),
            Effect.andThen(() => Effect.tryPromise(() => client.auth.getSession())),
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
                  "Failed to verify wallet. Please try again.",
                  new WalletVerificationError({
                    cause: extractErrorDetails(error),
                    operation: "verify",
                  }),
                ),
              )
            ),
          )
        ),
      ),

    Updating: () =>
      pipe(
        SupabaseClient,
        Effect.andThen((client) =>
          pipe(
            Effect.tryPromise(() => client.auth.getSession()),
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
          )
        ),
      ),
  })
}
