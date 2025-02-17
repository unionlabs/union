import {
  type AptosAccount,
  // (These below helpers remain as before)
  waitForTransactionReceipt,
  type AptosPublicAccountInfo
} from "./transfer.ts"
import { err, type Result } from "neverthrow"
import { Aptos, Network, AptosConfig, AccountAddress, MoveVector } from "@aptos-labs/ts-sdk"
import { createClient, fallback, type HttpTransport } from "viem"
import type { AptosBrowserWallet, AuthAccess } from "./wallet.ts"

export type { AptosAccount, AptosBrowserWallet }

export const aptosChainId = [
  "2", // aptos testnet
  "177", // movement porto
  "250" // movement bardock
] as const
export type AptosChainId = `${(typeof aptosChainId)[number]}`

/**
 * This client supports two kinds of transports:
 * - A key-based transport (using an AptosAccount)
 * - A wallet-based transport (using an AptosBrowserWallet)
 */
type AptosWindowTransport = AptosBrowserWallet

export type AptosClientParameters = {
  chainId: AptosChainId
} & (
  | { account: AptosAccount; transport: HttpTransport }
  | { account?: AptosPublicAccountInfo; transport: AptosWindowTransport }
)

/**
 * Overloads for retrieving an Aptos client.
 */
async function getAptosClient(
  parameters: AptosClientParameters & { authAccess: "key" }
): Promise<{ authAccess: "key"; aptos: Aptos; signer: AptosAccount }>

// async function getAptosClient(
//   parameters: AptosClientParameters & { authAccess: "wallet" }
// ): Promise<{ authAccess: "wallet"; aptos: Aptos; signer: AptosBrowserWallet }>

async function getAptosClient(
  parameters: AptosClientParameters & { authAccess: AuthAccess }
): Promise<
  | { authAccess: "key"; aptos: Aptos; signer: AptosAccount }
  | { authAccess: "wallet"; aptos: Aptos; signer: AptosBrowserWallet }
> {
  if (parameters.authAccess === "key") {
    if (typeof parameters.transport !== "function") {
      throw new Error("Invalid Aptos transport")
    }
    const rpcUrl = parameters.transport({}).value?.url
    if (!rpcUrl) throw new Error("No Aptos RPC URL found")
    const config = new AptosConfig({
      fullnode: rpcUrl,
      network: Network.CUSTOM
    })
    return {
      authAccess: "key",
      aptos: new Aptos(config),
      signer: parameters.account as AptosAccount
    }
  }

  if (parameters.authAccess === "wallet") {
    if (typeof parameters.transport !== "object") {
      throw new Error("Invalid Aptos transport")
    }
    const networkInfo = await parameters.transport.getNetwork()
    const network = networkInfo.name.toLowerCase() === "mainnet" ? Network.MAINNET : Network.TESTNET
    const config = new AptosConfig({ fullnode: networkInfo.url, network })
    return {
      authAccess: "wallet",
      aptos: new Aptos(config),
      signer: parameters.transport as AptosBrowserWallet
    }
  }
  throw new Error("Invalid Aptos transport")
}

/**
 * New unified transfer parameters for Aptos,
 * matching the Cosmos & EVM clients.
 */
export interface TransferAssetParameters<AptosChainId> {
  baseAmount: bigint
  baseToken: string
  quoteAmount: bigint
  quoteToken: string
  receiver: string
  sourceChannelId: number
  ucs03address: string
}

/**
 * The Aptos client now exposes both a `transferAsset` and a `simulateTransaction`
 * function that accept the same parameters as in your other chains.
 */
export const createAptosClient = (clientParameters: AptosClientParameters) => {
  return createClient({ transport: fallback([]) })
    .extend(_ => ({
      // A helper to get the underlying Aptos client.
      // We default to "key" if an account was provided.
      getAptosClient: async () => await getAptosClient({ ...clientParameters, authAccess: "key" })
      // clientParameters.account
      //   ? await getAptosClient({ ...clientParameters, authAccess: "key" })
      //   : await getAptosClient({ ...clientParameters, authAccess: "wallet" })
    }))
    .extend(client => ({
      waitForTransactionReceipt: async ({ hash }: { hash: string }) => {
        const { aptos, signer } = await client.getAptosClient()
        return await waitForTransactionReceipt({ aptos, hash })
      },

      /**
       * Executes a transfer on Aptos by calling the UCS03 contract’s `transfer` function.
       * The parameters mirror those used on Cosmos and EVM.
       */
      transferAsset: async ({
        baseAmount,
        baseToken,
        quoteAmount,
        quoteToken,
        receiver,
        sourceChannelId,
        ucs03address
      }: TransferAssetParameters<AptosChainId>): Promise<Result<string, Error>> => {
        const { aptos, signer } = await client.getAptosClient()

        const baseTokenHex = baseToken.startsWith("0x") ? baseToken.slice(2) : baseToken // Remove "0x" if it exists
        // let my_addr = AccountAddress.fromHex(baseToken)

        const quoteTokenVec = MoveVector.U8(quoteToken)
        const receiverVec = MoveVector.U8(receiver)

        const rawSalt = new Uint8Array(32)
        crypto.getRandomValues(rawSalt)
        const salt = MoveVector.U8(rawSalt)

        const payload = await aptos.transaction.build.simple({
          sender: signer.accountAddress,
          data: {
            function: `${ucs03address}::ibc_app::transfer`,
            typeArguments: [],
            functionArguments: [
              sourceChannelId,
              receiverVec,
              AccountAddress.fromString(baseToken),
              baseAmount,
              quoteTokenVec,
              quoteAmount,
              18446744073709551615n,
              18446744073709551615n,
              salt
            ]
          }
        })

        try {
          const txn = await aptos.signAndSubmitTransaction({
            signer: signer,
            transaction: payload
          })
          const receipt = await waitForTransactionReceipt({ aptos, hash: txn.hash })
          return receipt
        } catch (error) {
          return err(new Error("failed to execute aptos call", { cause: error as Error }))
        }
      }
    }))
}
