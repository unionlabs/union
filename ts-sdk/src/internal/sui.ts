import { SuiClient, SuiClientOptions } from "@mysten/sui/client"
import { Ed25519Keypair } from "@mysten/sui/keypairs/ed25519"
import { Context, Effect, Layer, pipe } from "effect"
import * as Sui from "../Sui.js"
import { extractErrorDetails } from "../utils/extract-error-details.js"

export const publicClientLayer = <
  Id,
>(tag: Context.Tag<Id, Sui.Sui.PublicClient>) =>
(
  ...options: ConstructorParameters<typeof SuiClient>
): Layer.Layer<Id, Sui.CreatePublicClientError> =>
  Layer.effect(
    tag,
    pipe(
      Effect.try({
        try: () => new SuiClient(...options),
        catch: err =>
          new Sui.CreatePublicClientError({
            cause: extractErrorDetails<Sui.CreatePublicClientErrorType>(err as unknown as any),
          }),
      }),
      Effect.map((client) => ({ client })),
    ),
  )

/** @internal */
export const walletClientLayer = <
  Id,
>(tag: Context.Tag<Id, Sui.Sui.WalletClient>) =>
(
  options: SuiClientOptions,
  signer: Ed25519Keypair,
): Layer.Layer<Id, Sui.CreateWalletClientError> =>
  Layer.effect(
    tag,
    Effect.try({
      try: () => ({
        client: new SuiClient(options),
        signer,
      }),
      catch: (err) =>
        new Sui.CreateWalletClientError({
          cause: extractErrorDetails(err as Sui.CreateWalletClientErrorType),
        }),
    }),
  )
