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
export const walletClientLayer = <
  Id,
>(tag: Context.Tag<Id, Sui.Sui.WalletClient>) =>
(
  // interface unchanged
  options: Parameters<typeof Object>[0] & {
    account: Ed25519Keypair
    chain: unknown
  },
): Layer.Layer<Id, Sui.CreateWalletClientError> =>
  Layer.effect(
    tag,
    pipe(
      Effect.try({
        try: () => ({
          client: new SuiClient(options as unknown as SuiClientOptions),
          signer: options.account as Ed25519Keypair,
        }),
        catch: (err) =>
          new Sui.CreateWalletClientError({
            cause: Utils.extractErrorDetails(err as Sui.SuiCreateWalletClientErrorType),
          }),
      }),
      // return the *same* payload shape you had before
      Effect.map(({ client }) => ({
        client,
        account: options.account,
        chain: options.chain,
      })),
    ),
  )
