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
import { bech32AddressToHex } from "../convert.ts"
import type { TransferAssetsParameters } from "./types.ts"
import { createPfmMemo, getHubbleChainDetails } from "../pfm.ts"
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

export const createCosmosClient = (parameters: CosmosClientParameters) =>
  createClient({ transport: fallback([]) }).extend(_ => ({
    transferAsset: async ({
      amount,
      receiver,
      denomAddress,
      destinationChainId,
      memo = timestamp(),
      relayContractAddress,
      account = parameters.account,
      gasPrice = parameters.gasPrice
    }: TransferAssetsParameters<CosmosChainId>): Promise<Result<string, Error>> => {
      const sourceChainId = parameters.chainId
      const rpcUrl = parameters.transport({}).value?.url

      if (!rpcUrl) return err(new Error("No cosmos RPC URL found"))
      if (!account) return err(new Error("No cosmos signer found"))
      if (!gasPrice) return err(new Error("No gas price found"))

      if (sourceChainId === "union-testnet-8" && destinationChainId === "union-testnet-8") {
        const transfer = await cosmosSameChainTransfer({
          rpcUrl,
          account,
          gasPrice,
          receiver,
          asset: { denom: denomAddress, amount: amount.toString() }
        })
        return transfer
      }

      const stamp = timestamp()
      const ucsDetails = await getHubbleChainDetails({
        sourceChainId,
        destinationChainId
      })

      if (ucsDetails.isErr()) return err(ucsDetails.error)

      const pfmMemo = createPfmMemo({
        port: `${ucsDetails.value.port}`,
        channel: ucsDetails.value.destinationChannel,
        receiver: cosmosChainId.includes(destinationChainId)
          ? bech32AddressToHex({ address: `${receiver}` })
          : `${receiver}`
      })

      if (pfmMemo.isErr()) return err(pfmMemo.error)

      const sourceChannel = ucsDetails.value.sourceChannel
      destinationChainId = ucsDetails.value.destinationChainId
      relayContractAddress ??= ucsDetails.value.relayContractAddress

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
                  receiver: receiver.startsWith("0x") ? receiver.slice(2) : receiver,
                  memo: pfmMemo.value ?? `${stamp} Sending ${amount} ${denomAddress} to ${receiver}`
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

        const transfer = await ibcTransfer({
          account,
          rpcUrl,
          gasPrice,
          messageTransfers: [
            {
              sourceChannel,
              sourcePort: "transfer",
              sender: account_?.address,
              token: { denom: denomAddress, amount: amount.toString() },
              timeoutHeight: { revisionHeight: 888_888_888n, revisionNumber: 8n },
              receiver: receiver.startsWith("0x") ? receiver.slice(2) : receiver,
              memo: pfmMemo.value ?? `${stamp} Sending ${amount} ${denomAddress} to ${receiver}`
            }
          ]
        })
        return transfer
      }

      return err(new Error("Unsupported network"))
    },
    simulateTransaction: async ({
      memo,
      amount,
      receiver,
      denomAddress,
      destinationChainId,
      relayContractAddress,
      account = parameters?.account,
      gasPrice = parameters?.gasPrice
    }: TransferAssetsParameters<CosmosChainId>): Promise<Result<string, Error>> => {
      const sourceChainId = parameters.chainId
      const rpcUrl = parameters.transport({}).value?.url

      if (!rpcUrl) return err(new Error("No cosmos RPC URL found"))
      if (!account) return err(new Error("No cosmos signer found"))
      if (!gasPrice) return err(new Error("No gas price found"))

      // Union to Union
      if (sourceChainId === "union-testnet-8" && destinationChainId === "union-testnet-8") {
        return await cosmosSameChainTransferSimulate({
          receiver,
          account,
          rpcUrl,
          asset: { denom: denomAddress, amount: amount.toString() },
          gasPrice: gasPrice ?? { amount: "0.0025", denom: "muno" }
        })
      }

      const ucsDetails = await getHubbleChainDetails({
        sourceChainId,
        destinationChainId
      })

      if (ucsDetails.isErr()) return err(ucsDetails.error)

      const pfmMemo = createPfmMemo({
        port: `${ucsDetails.value.port}`,
        channel: ucsDetails.value.destinationChannel,
        receiver: cosmosChainId.includes(destinationChainId)
          ? bech32AddressToHex({ address: `${receiver}` })
          : `${receiver}`
      })

      if (pfmMemo.isErr()) return err(pfmMemo.error)

      const sourceChannel = ucsDetails.value.sourceChannel
      destinationChainId = ucsDetails.value.destinationChainId
      relayContractAddress ??= ucsDetails.value.relayContractAddress

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
                  receiver: receiver.startsWith("0x") ? receiver.slice(2) : receiver,
                  memo: pfmMemo.value ?? `${stamp} Sending ${amount} ${denomAddress} to ${receiver}`
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

        const stamp = timestamp()
        return await ibcTransferSimulate({
          gasPrice,
          account,
          rpcUrl,
          messageTransfers: [
            {
              sourceChannel,
              sourcePort: "transfer",
              sender: account_?.address,
              token: { denom: denomAddress, amount: amount.toString() },
              timeoutHeight: { revisionHeight: 888_888_888n, revisionNumber: 8n },
              receiver: receiver.startsWith("0x") ? receiver.slice(2) : receiver,
              memo: pfmMemo.value ?? `${stamp} Sending ${amount} ${denomAddress} to ${receiver}`
            }
          ]
        })
      }

      return err(new Error("Unsupported network"))
    }
  }))
