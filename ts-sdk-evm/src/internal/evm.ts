import * as Utils from "@unionlabs/sdk/Utils"
import { Context, Effect, Layer, pipe } from "effect"
import * as V from "viem"
import * as Evm from "../Evm.js"

/** @internal */
export const publicClientLayer = <
  Id,
>(tag: Context.Tag<Id, Evm.Evm.PublicClient>) =>
(
  ...options: Parameters<typeof V.createPublicClient>
): Layer.Layer<Id, Evm.CreatePublicClientError> =>
  Layer.effect(
    tag,
    pipe(
      Effect.try({
        try: () => V.createPublicClient(...options),
        catch: err =>
          new Evm.CreatePublicClientError({
            cause: Utils.extractErrorDetails(err as V.CreatePublicClientErrorType),
          }),
      }),
      Effect.map((client) => ({ client })),
    ),
  )

/** @internal */
export const walletClientLayer = <
  Id,
>(tag: Context.Tag<Id, Evm.Evm.WalletClient>) =>
(
  options: Parameters<typeof V.createWalletClient>[0] & {
    account: V.Account
    chain: V.Chain
  },
): Layer.Layer<Id, Evm.CreateWalletClientError> =>
  Layer.effect(
    tag,
    pipe(
      Effect.try({
        try: () => V.createWalletClient(options),
        catch: err =>
          new Evm.CreateWalletClientError({
            cause: Utils.extractErrorDetails(err as V.CreateWalletClientErrorType),
          }),
      }),
      Effect.map((client) => ({ client, account: options.account, chain: options.chain })),
    ),
  )
