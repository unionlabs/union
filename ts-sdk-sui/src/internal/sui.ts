import * as Utils from "@unionlabs/sdk/Utils"
import { Context, Effect, Layer, pipe } from "effect"
import * as V from "viem"
import * as Sui from "../Sui.js"
import { SuiClient, SuiClientOptions } from "@mysten/sui/client"
import { Ed25519Keypair } from "@mysten/sui/keypairs/ed25519"

/** @internal */
export const publicClientLayer = <
  Id,
>(tag: Context.Tag<Id, Sui.Sui.PublicClient>) =>
(
  // interface unchanged (variadic)
  ...options: Parameters<any>
): Layer.Layer<Id, Sui.CreatePublicClientError> =>
  Layer.effect(
    tag,
    pipe(
      Effect.try({
        try: () => new SuiClient(options[0] as SuiClientOptions),
        catch: (err) =>
          new Sui.CreatePublicClientError({
            // mirror your Sui error-detail extraction
            cause: Utils.extractErrorDetails(err as Sui.CreatePublicClientError),
          }),
      }),
      // keep the { client } shape
      Effect.map((client) => ({ client })),
    ),
  )


/** @internal */
export const walletClientLayer = <Id>(
  tag: Context.Tag<Id, Sui.Sui.WalletClient>,
) =>
(opts: { url: string; signer: Ed25519Keypair }): Layer.Layer<Id, Sui.CreateWalletClientError> =>
  Layer.effect(
    tag,
    Effect.try({
      try: () => {
        if (!opts?.signer || typeof opts.signer.getPublicKey !== "function") {
          throw new Error("Invalid Sui signer: expected Ed25519Keypair")
        }
        const client = new SuiClient({ url: opts.url } satisfies SuiClientOptions)
        return { client, signer: opts.signer } // <-- matches Sui.Sui.WalletClient interface
      },
      catch: (err) =>
        new Sui.CreateWalletClientError({
          cause: Utils.extractErrorDetails(err as Error),
        }),
    }),
  )