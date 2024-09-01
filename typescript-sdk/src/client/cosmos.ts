import {
  ibcTransfer,
  cosmwasmTransfer,
  ibcTransferSimulate,
  cosmosSameChainTransfer,
  cosmwasmTransferSimulate,
  cosmosSameChainTransferSimulate
} from "../transfer/cosmos.ts"
import { err, type Result } from "neverthrow"
import type { OfflineSigner } from "../types.ts"
import { timestamp } from "../utilities/index.ts"
import type { TransferAssetsParameters } from "./types.ts"
import { fallback, createClient, type HttpTransport } from "viem"

export const cosmosChainId = [
  "mocha-4",
  "elgafar-1",
  "osmo-test-5",
  "union-testnet-8",
  "stride-internal-1"
] as const

export type CosmosChainId = `${(typeof cosmosChainId)[number]}`

export interface CosmosClientParameters {
  chainId: CosmosChainId
  account?: OfflineSigner
  /**
   * TODO: support multiple transports through fallback
   */
  transport: HttpTransport
  gasPrice?: { amount: string; denom: string }
}

/**
 * TODO: add JSDoc with examples
 */
export const createCosmosClient = (parameters: CosmosClientParameters) =>
  createClient({ transport: fallback([]) })
    .extend(_ => ({
      transferAsset: async ({
        destinationChainId,
        amount,
        recipient,
        sourcePort,
        denomAddress,
        sourceChannel,
        memo = timestamp(),
        relayContractAddress,
        account = parameters.account,
        gasPrice = parameters.gasPrice
      }: TransferAssetsParameters<CosmosChainId>): Promise<Result<string, Error>> => {
        const sourceChainId = parameters.chainId
        const rpcUrl = parameters.transport({}).value?.url

        if (!account) return err(new Error("No cosmos signer found"))
        if (!rpcUrl) return err(new Error("No cosmos RPC URL found"))
        if (!gasPrice) return err(new Error("No gas price found"))

        if (sourceChainId === "union-testnet-8" && destinationChainId === "union-testnet-8") {
          const transfer = await cosmosSameChainTransfer({
            recipient,
            account,
            rpcUrl,
            gasPrice,
            asset: { denom: denomAddress, amount: amount.toString() }
          })
          return transfer
        }
        const stamp = timestamp()
        if (sourceChainId === "union-testnet-8") {
          if (!sourceChannel) return err(new Error("Source channel not found"))
          if (!relayContractAddress) return err(new Error("Relay contract address not found"))

          const transfer = await cosmwasmTransfer({
            account,
            rpcUrl,
            gasPrice,
            instructions: [
              {
                contractAddress: relayContractAddress,
                msg: {
                  transfer: {
                    channel: sourceChannel,
                    receiver: recipient.startsWith("0x") ? recipient.slice(2) : recipient,
                    memo: memo ?? `${stamp} Sending ${amount} ${denomAddress} to ${recipient}`
                  }
                },
                funds: [{ amount: amount.toString(), denom: denomAddress }]
              }
            ]
          })
          return transfer
        }
        if (destinationChainId === "union-testnet-8") {
          if (!sourceChannel) return err(new Error("Source channel not found"))

          const [account_] = await account.getAccounts()
          if (!account) return err(new Error("No account found"))

          sourcePort ||= "transfer"
          const transfer = await ibcTransfer({
            account,
            rpcUrl,
            gasPrice,
            messageTransfers: [
              {
                sourcePort,
                sourceChannel,
                sender: account_?.address,
                token: { denom: denomAddress, amount: amount.toString() },
                timeoutHeight: { revisionHeight: 888_888_888n, revisionNumber: 8n },
                receiver: recipient.startsWith("0x") ? recipient.slice(2) : recipient,
                memo: memo ?? `${stamp} Sending ${amount} ${denomAddress} to ${recipient}`
              }
            ]
          })
          return transfer
        }

        return err(new Error("Unsupported network"))
      }
    }))
    .extend(_ => ({
      simulateTransaction: async ({
        memo,
        amount,
        recipient,
        sourcePort,
        denomAddress,
        sourceChannel,
        relayContractAddress,
        account = parameters?.account,
        gasPrice = parameters?.gasPrice,
        destinationChainId
      }: TransferAssetsParameters<CosmosChainId>): Promise<Result<string, Error>> => {
        const sourceChainId = parameters.chainId
        const rpcUrl = parameters.transport({}).value?.url

        if (!rpcUrl) return err(new Error("No cosmos RPC URL found"))
        if (!account) return err(new Error("No cosmos signer found"))
        if (!gasPrice) return err(new Error("No gas price found"))
        if (!rpcUrl) return err(new Error("No cosmos RPC URL found"))

        if (sourceChainId === "union-testnet-8" && destinationChainId === "union-testnet-8") {
          // Union to Union
          return await cosmosSameChainTransferSimulate({
            recipient,
            account,
            rpcUrl,
            asset: { denom: denomAddress, amount: amount.toString() },
            gasPrice: gasPrice ?? { amount: "0.0025", denom: "muno" }
          })
        }

        if (sourceChainId === "union-testnet-8") {
          if (!relayContractAddress) return err(new Error("Relay contract address not found"))

          const stamp = timestamp()
          return await cosmwasmTransferSimulate({
            gasPrice,
            rpcUrl,
            account,
            instructions: [
              {
                contractAddress: relayContractAddress,
                msg: {
                  transfer: {
                    channel: sourceChannel,
                    receiver: recipient.startsWith("0x") ? recipient.slice(2) : recipient,
                    memo: memo ?? `${stamp} Sending ${amount} ${denomAddress} to ${recipient}`
                  }
                },
                funds: [{ amount: amount.toString(), denom: denomAddress }]
              }
            ]
          })
        }
        if (destinationChainId === "union-testnet-8") {
          if (!sourceChannel) return err(new Error("Source channel not found"))
          const [account_] = await account.getAccounts()
          if (!account) return err(new Error("No account found"))
          sourcePort ||= "transfer"
          const stamp = timestamp()
          return await ibcTransferSimulate({
            gasPrice,
            account,
            rpcUrl,
            messageTransfers: [
              {
                sourcePort,
                sourceChannel,
                sender: account_?.address,
                token: { denom: denomAddress, amount: amount.toString() },
                timeoutHeight: { revisionHeight: 888_888_888n, revisionNumber: 8n },
                receiver: recipient.startsWith("0x") ? recipient.slice(2) : recipient,
                memo: memo ?? `${stamp} Sending ${amount} ${denomAddress} to ${recipient}`
              }
            ]
          })
        }

        return err(new Error("Unsupported network"))
      }
    }))
