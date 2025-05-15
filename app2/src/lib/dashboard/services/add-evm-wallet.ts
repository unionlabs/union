import { Data, Effect, Option, pipe } from "effect"
import { switchChain } from "$lib/services/transfer-ucs03-evm"
import type { Chain, WalletClient, Hash } from "viem"
import { signMessage } from "@wagmi/core"
import { Siwe } from "ox"
import { getWagmiConfig } from "$lib/wallet/evm/wagmi-config.svelte"
import { submitWalletVerification } from "$lib/dashboard/queries/private"
import { getSupabaseClient } from "$lib/dashboard/client"
import { clearLocalStorageCacheEntry } from "$lib/dashboard/services/cache"
import { CACHE_VERSION } from "$lib/dashboard/config"
import { dashboard } from "$lib/dashboard/stores/user.svelte"
import type { WalletStore } from "$lib/dashboard/stores/wallets.svelte"
import { extractErrorDetails } from "@unionlabs/sdk/utils"

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
  walletClient: WalletClient
) => {
  return AddEvmWalletState.$match(state, {
    SwitchChain: ({ chain }) => {
      return pipe(
        switchChain(chain),
        Effect.map(() => ok(Signing({ walletClient, chain }), "Chain switched successfully")),
        Effect.catchAll((error) => 
          Effect.fail(new WalletVerificationError({ 
            cause: extractErrorDetails(error), 
            operation: "switchChain" 
          }))
        ),
        Effect.match({
          onFailure: (error) => fail("Failed to switch chain", error),
          onSuccess: (result) => result
        })
      )
    },
    Signing: ({ walletClient, chain }) => {
      const address = walletClient.account?.address
      if (!address) {
        return fail("No wallet address found")
      }

      const siweMessage = Siwe.createMessage({
        address: address as `0x${string}`,
        version: "1" as const,
        chainId: chain.id,
        nonce: Siwe.generateNonce(),
        domain: 'dashboard.union.build',
        uri: 'https://dashboard.union.build/wallet',
        statement: "Sign this message to verify wallet ownership."
      })

      const messageToSign = siweMessage.toString()

      return pipe(
        Effect.tryPromise(() => 
          signMessage(getWagmiConfig(), {
            account: address as `0x${string}`,
            message: messageToSign,
          })
        ),
        Effect.map((signature) => ok(Verifying({ address, chain, signature, message: messageToSign }), "Signature received. Verifying...")),
        Effect.catchAll((error) => 
          Effect.fail(new WalletVerificationError({ 
            cause: extractErrorDetails(error), 
            operation: "sign" 
          }))
        ),
        Effect.match({
          onFailure: (error) => fail("Failed to sign message. Please try again.", error),
          onSuccess: (result) => result
        })
      )
    },
    Verifying: ({ address, chain, signature, message }) => {
      return pipe(
        getSupabaseClient(),
        Effect.flatMap(client => Effect.tryPromise(() => client.auth.refreshSession())),
        Effect.flatMap(({ data: { session } }) => {
          if (!session?.user.id) {
            return Effect.fail(new WalletVerificationError({ 
              cause: "No authenticated user found", 
              operation: "verify" 
            }))
          }
          return submitWalletVerification({
            id: session.user.id,
            address,
            chainId: `evm:${chain.id}`,
            message,
            signature,
            selectedChains: null
          })
        }),
        Effect.flatMap(() => getSupabaseClient()),
        Effect.flatMap(client => Effect.tryPromise(() => client.auth.getSession())),
        Effect.flatMap(({ data: { session } }) => {
          if (!session?.user.id) {
            return Effect.fail(new WalletVerificationError({ 
              cause: "No user ID found", 
              operation: "verify" 
            }))
          }
          return Effect.succeed(ok(Updating(), "Wallet verified. Updating data..."))
        }),
        Effect.catchAll((error) => 
          Effect.fail(new WalletVerificationError({ 
            cause: extractErrorDetails(error), 
            operation: "verify" 
          }))
        ),
        Effect.match({
          onFailure: (error) => fail("Failed to verify wallet", error),
          onSuccess: (result) => result
        })
      )
    },
    Updating: () => {
      return pipe(
        getSupabaseClient(),
        Effect.flatMap(client => Effect.tryPromise(() => client.auth.getSession())),
        Effect.flatMap(({ data: { session } }) => {
          if (!session?.user.id) {
            return Effect.fail(new WalletVerificationError({ 
              cause: "No user ID found", 
              operation: "update" 
            }))
          }
          return pipe(
            clearLocalStorageCacheEntry("wallets", `${CACHE_VERSION}:${session.user.id}`),
            Effect.mapError(error => new WalletVerificationError({ 
              cause: extractErrorDetails(error), 
              operation: "update" 
            })),
            Effect.flatMap(() => Effect.sleep("3 seconds")),
            Effect.flatMap(() => Effect.sync(() => {
              Option.map(dashboard.wallets, (store: WalletStore) => store.refresh())
              return complete()
            }))
          )
        }),
        Effect.catchAll((error) => 
          Effect.fail(new WalletVerificationError({ 
            cause: extractErrorDetails(error), 
            operation: "update" 
          }))
        ),
        Effect.match({
          onFailure: (error) => fail("Failed to update wallet data", error),
          onSuccess: (result) => result
        })
      )
    },
  })
}
