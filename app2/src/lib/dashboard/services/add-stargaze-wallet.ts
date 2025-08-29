import type { Keplr } from "@keplr-wallet/types"
import type { Leap } from "@leapwallet/types"
import type { SupabaseClient } from "@supabase/supabase-js"
import { extractErrorDetails } from "@unionlabs/sdk/utils"
import { Data, Effect, Option, pipe } from "effect"
import { verifyStargazeWallet } from "../queries"
import { STARGAZE_VERIFICATION_MESSAGE } from "../queries/types"

export class StargazeWalletError extends Data.TaggedError("StargazeWalletError")<{
  cause: unknown
  operation: "connect" | "sign" | "verify"
}> {}

export type AddStargazeWalletState = Data.TaggedEnum<{
  SwitchChain: {
    chainId: string
  }
  Signing: {}
  Verifying: {
    address: string
    signature: string
    signingAddress: string
  }
  Updating: {}
}>

export const AddStargazeWalletState = Data.taggedEnum<AddStargazeWalletState>()
const { Signing, Verifying, Updating } = AddStargazeWalletState

export type StargazeWalletResult = {
  nextState: Option.Option<AddStargazeWalletState>
  message: string
  error: Option.Option<Error>
  completed: boolean
}

const fail = (msg: string, error?: Error): StargazeWalletResult => ({
  nextState: Option.none(),
  message: msg,
  error: Option.fromNullable(error),
  completed: false,
})

const ok = (state: AddStargazeWalletState, msg: string): StargazeWalletResult => ({
  nextState: Option.some(state),
  message: msg,
  error: Option.none(),
  completed: false,
})

const complete = (msg: string = "Stargaze wallet verified successfully"): StargazeWalletResult => ({
  nextState: Option.none(),
  message: msg,
  error: Option.none(),
  completed: true,
})

export const addStargazeWallet = (
  state: AddStargazeWalletState,
  walletClient: Keplr | Leap,
) => {
  return AddStargazeWalletState.$match(state, {
    SwitchChain: () => {
      // Skip chain switching - Stargaze is already available in most wallets
      return Effect.succeed(ok(Signing(), "Ready to sign message"))
    },

    Signing: () => {
      if (!walletClient) {
        return Effect.succeed(
          fail(
            "No Stargaze wallet found. Please try reconnecting.",
            new StargazeWalletError({
              cause: "Wallet client not provided",
              operation: "sign",
            }),
          ),
        )
      }

      return pipe(
        Effect.tryPromise(async () => {
          const walletKey = await walletClient.getKey("stargaze-1")
          const address = walletKey.bech32Address
          const signature = await walletClient.signArbitrary(
            "stargaze-1",
            address,
            STARGAZE_VERIFICATION_MESSAGE,
          )
          console.log("returning", { address, signature })
          return { address, signature }
        }),
        Effect.map(({ address, signature }) => {
          console.log("Effect.map called with:", { address, signature })
          const result = ok(
            Verifying({ address, signature, signingAddress: address }),
            "Signature received. Verifying...",
          )
          console.log("Effect.map returning result:", result)
          return result
        }),
        Effect.catchAll((error) =>
          Effect.succeed(
            fail(
              "Failed to sign message. Please try again.",
              new StargazeWalletError({
                cause: extractErrorDetails(error),
                operation: "sign",
              }),
            ),
          )
        ),
      )
    },

    Verifying: ({ address, signature, signingAddress }) => {
      return pipe(
        verifyStargazeWallet(address, STARGAZE_VERIFICATION_MESSAGE, signature, signingAddress),
        Effect.map((result) => {
          console.log("addStargazeWallet: Got verification result:", result)
          if (Option.isSome(result) && result.value.success) {
            console.log("addStargazeWallet: Verification successful!")
            return ok(Updating(), "Wallet verified. Updating data...")
          } else {
            console.log("addStargazeWallet: Verification failed")
            return fail("Failed to verify wallet signature")
          }
        }),
        Effect.catchAll((error) =>
          Effect.succeed(
            fail(
              "Failed to verify wallet",
              new StargazeWalletError({
                cause: extractErrorDetails(error),
                operation: "verify",
              }),
            ),
          )
        ),
      )
    },

    Updating: () => {
      // Simple updating state - just complete after a brief delay like Cosmos wallet
      return pipe(
        Effect.sleep("1 second"),
        Effect.map(() => complete("Stargaze wallet verified and saved!")),
      )
    },
  })
}
