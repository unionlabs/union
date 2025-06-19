import { switchChain } from "$lib/services/transfer-ucs03-evm"
import { getWagmiConfig } from "$lib/wallet/evm/wagmi-config.svelte"
import type { SupabaseClient } from "@supabase/supabase-js"
import { extractErrorDetails } from "@unionlabs/sdk/utils"
import { signMessage } from "@wagmi/core"
import { Data, Effect, Option, pipe } from "effect"
import type { Hash, WalletClient } from "viem"
import { mainnet, sepolia } from "viem/chains"
import { verifyAirdropWallet } from "../queries"
import { type SupportedChainId, WALLET_VERIFICATION_MESSAGE } from "../queries/types"

export class AirdropWalletError extends Data.TaggedError("AirdropWalletError")<{
  cause: unknown
  operation: "switchChain" | "sign" | "verify"
}> {}

export type AddAirdropWalletState = Data.TaggedEnum<{
  SwitchChain: {
    targetChainId: SupportedChainId
  }
  Signing: {
    walletClient: WalletClient
    chainId: SupportedChainId
  }
  Verifying: {
    address: string
    chainId: SupportedChainId
    signature: Hash
  }
}>

export const AddAirdropWalletState = Data.taggedEnum<AddAirdropWalletState>()
const { SwitchChain, Signing, Verifying } = AddAirdropWalletState

export type AirdropWalletResult = {
  nextState: Option.Option<AddAirdropWalletState>
  message: string
  error: Option.Option<Error>
  completed: boolean
}

const fail = (msg: string, error?: Error): AirdropWalletResult => ({
  nextState: Option.none(),
  message: msg,
  error: Option.fromNullable(error),
  completed: false,
})

const ok = (state: AddAirdropWalletState, msg: string): AirdropWalletResult => ({
  nextState: Option.some(state),
  message: msg,
  error: Option.none(),
  completed: false,
})

const complete = (msg: string = "Wallet verified successfully"): AirdropWalletResult => ({
  nextState: Option.none(),
  message: msg,
  error: Option.none(),
  completed: true,
})

export const addAirdropWallet = (
  state: AddAirdropWalletState,
  walletClient: WalletClient,
): Effect.Effect<AirdropWalletResult, never, SupabaseClient> => {
  return AddAirdropWalletState.$match(state, {
    SwitchChain: ({ targetChainId }) => {
      const targetChain = targetChainId === "1" ? mainnet : sepolia

      return pipe(
        switchChain(targetChain),
        Effect.map(() =>
          ok(Signing({ walletClient, chainId: targetChainId }), "Chain switched successfully")
        ),
        Effect.catchAll((error) =>
          Effect.succeed(
            fail(
              "Failed to switch to the correct network. Please try again.",
              new AirdropWalletError({
                cause: extractErrorDetails(error),
                operation: "switchChain",
              }),
            ),
          )
        ),
      )
    },

    Signing: ({ walletClient, chainId }) => {
      const address = walletClient.account?.address
      if (!address) {
        return Effect.succeed(fail("No wallet address found"))
      }

      return pipe(
        Effect.tryPromise(() =>
          signMessage(getWagmiConfig(), {
            account: address as `0x${string}`,
            message: WALLET_VERIFICATION_MESSAGE,
          })
        ),
        Effect.map((signature) =>
          ok(
            Verifying({ address, chainId, signature }),
            "Signature received. Verifying...",
          )
        ),
        Effect.catchAll((error) =>
          Effect.succeed(
            fail(
              "Failed to sign message. Please try again.",
              new AirdropWalletError({
                cause: extractErrorDetails(error),
                operation: "sign",
              }),
            ),
          )
        ),
      )
    },

    Verifying: ({ address, chainId, signature }) =>
      pipe(
        verifyAirdropWallet(address, WALLET_VERIFICATION_MESSAGE, signature, chainId),
        Effect.map((result) => {
          if (Option.isSome(result) && result.value.success) {
            return complete("EVM wallet verified and saved for airdrop!")
          } else {
            return fail("Failed to verify wallet signature")
          }
        }),
        Effect.catchAll((error) =>
          Effect.succeed(
            fail(
              "Failed to verify wallet",
              new AirdropWalletError({
                cause: extractErrorDetails(error),
                operation: "verify",
              }),
            ),
          )
        ),
      ),
  })
}
