import { extractErrorDetails } from "@unionlabs/sdk/Utils"
import { Context, Effect, Layer, pipe } from "effect"
import * as V from "viem"
import * as Wallet from "../Wallet.js"

/** @internal */
export const walletClientLayer = <
  Id,
>(tag: Context.Tag<Id, Wallet.Wallet>) =>
(
  options: Parameters<typeof V.createWalletClient>[0] & {
    account: V.Account
    chain: V.Chain
  },
): Layer.Layer<Id, Wallet.CreateWalletError> =>
  Layer.effect(
    tag,
    pipe(
      Effect.try({
        try: () => V.createWalletClient(options),
        catch: err =>
          new Wallet.CreateWalletError({
            cause: extractErrorDetails(err as V.CreateWalletClientErrorType),
          }),
      }),
      Effect.map((client) => ({ client, account: options.account, chain: options.chain })),
    ),
  )
