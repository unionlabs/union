import "./patch.ts"
import {
  ibcTransfer,
  cosmwasmTransfer,
  ibcTransferSimulate,
  cosmosSameChainTransfer,
  cosmwasmTransferSimulate,
  cosmosSameChainTransferSimulate
} from "../transfer/cosmos.ts"
import { timestamp } from "../utilities/index.ts"
import type { TransferAssetsParameters } from "./types.ts"
import { fallback, createClient, type HttpTransport } from "viem"
import type { OfflineSigner, TransactionResponse } from "../types.ts"

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

export const createCosmosClient = (parameters: CosmosClientParameters) =>
  createClient({ transport: fallback([]) })
    .extend(_ => ({
      transferAsset: async ({
        path,
        amount,
        recipient,
        sourcePort,
        denomAddress,
        cosmosSigner,
        sourceChannel,
        memo = timestamp(),
        relayContractAddress
      }: TransferAssetsParameters<
        CosmosClientParameters["chainId"]
      >): Promise<TransactionResponse> => {
        try {
          const [sourceChainId, destinationChainId] = path

          const cosmosRpcUrl = parameters.transport({}).value?.url
          if (!cosmosSigner) return { success: false, data: "No cosmos signer found" }
          if (!cosmosRpcUrl) return { success: false, data: "No cosmos RPC URL found" }
          if (!parameters.gasPrice) return { success: false, data: "No gas price found" }

          if (sourceChainId === "union-testnet-8" && destinationChainId === "union-testnet-8") {
            const transfer = await cosmosSameChainTransfer({
              recipient,
              cosmosSigner,
              cosmosRpcUrl,
              gasPrice: parameters.gasPrice,
              asset: { denom: denomAddress, amount: amount.toString() }
            })
            return transfer
          }
          const stamp = timestamp()
          if (sourceChainId === "union-testnet-8") {
            if (!sourceChannel) return { success: false, data: "Source channel not found" }
            if (!relayContractAddress) {
              return { success: false, data: "Relay contract address not found" }
            }
            const transfer = await cosmwasmTransfer({
              cosmosSigner,
              cosmosRpcUrl,
              gasPrice: parameters.gasPrice,
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
            if (!sourceChannel) return { success: false, data: "Source channel not found" }
            const [account] = await cosmosSigner.getAccounts()
            if (!account) return { success: false, data: "No account found" }
            sourcePort ||= "transfer"
            const transfer = await ibcTransfer({
              cosmosSigner,
              cosmosRpcUrl,
              gasPrice: parameters.gasPrice,
              messageTransfers: [
                {
                  sourcePort,
                  sourceChannel,
                  sender: account.address,
                  token: { denom: denomAddress, amount: amount.toString() },
                  timeoutHeight: { revisionHeight: 888_888_888n, revisionNumber: 8n },
                  receiver: recipient.startsWith("0x") ? recipient.slice(2) : recipient,
                  memo: memo ?? `${stamp} Sending ${amount} ${denomAddress} to ${recipient}`
                }
              ]
            })
            return transfer
          }
          return { success: false, data: "Unsupported network" }
        } catch (error) {
          console.error(error)
          return {
            success: false,
            data: error instanceof Error ? error.message : "An unknown error occurred"
          }
        }
      }
    }))
    .extend(_ => ({
      simulateTransaction: async ({
        path,
        memo,
        amount,
        recipient,
        sourcePort,
        denomAddress,
        sourceChannel,
        relayContractAddress,
        gasPrice = parameters?.gasPrice,
        cosmosSigner = parameters?.account
      }: TransferAssetsParameters<
        CosmosClientParameters["chainId"]
      >): Promise<TransactionResponse> => {
        const [sourceChainId, destinationChainId] = path

        if (!parameters.transport) return { success: false, data: "No transport found" }
        if (!cosmosSigner) return { success: false, data: "No cosmos signer found" }
        const cosmosRpcUrl = parameters.transport({}).value?.url

        if (!gasPrice) return { success: false, data: "No gas price found" }
        if (!cosmosRpcUrl) return { success: false, data: "No cosmos RPC URL found" }

        if (sourceChainId === "union-testnet-8" && destinationChainId === "union-testnet-8") {
          // Union to Union
          return await cosmosSameChainTransferSimulate({
            recipient,
            cosmosSigner,
            cosmosRpcUrl,
            asset: { denom: denomAddress, amount: amount.toString() },
            gasPrice: gasPrice ?? { amount: "0.0025", denom: "muno" }
          })
        }

        if (sourceChainId === "union-testnet-8") {
          if (!relayContractAddress) {
            return { success: false, data: "Relay contract address not found" }
          }
          const stamp = timestamp()
          return await cosmwasmTransferSimulate({
            gasPrice,
            cosmosRpcUrl,
            cosmosSigner,
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
          if (!sourceChannel) return { success: false, data: "Source channel not found" }
          const [account] = await cosmosSigner.getAccounts()
          if (!account) return { success: false, data: "No account found" }
          sourcePort ||= "transfer"
          const stamp = timestamp()
          return await ibcTransferSimulate({
            gasPrice,
            cosmosSigner,
            cosmosRpcUrl,
            messageTransfers: [
              {
                sourcePort,
                sourceChannel,
                sender: account.address,
                token: { denom: denomAddress, amount: amount.toString() },
                timeoutHeight: { revisionHeight: 888_888_888n, revisionNumber: 8n },
                receiver: recipient.startsWith("0x") ? recipient.slice(2) : recipient,
                memo: memo ?? `${stamp} Sending ${amount} ${denomAddress} to ${recipient}`
              }
            ]
          })
        }

        return { success: false, data: "Unsupported network" }
      }
    }))
