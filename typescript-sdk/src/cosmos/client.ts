import {
  ibcTransfer,
  cosmwasmTransfer,
  ibcTransferSimulate,
  cosmosSameChainTransfer,
  cosmwasmTransferSimulate,
  cosmosSameChainTransferSimulate
} from "./transfer.ts"
import { err, type Result } from "neverthrow"
import { timestamp } from "../utilities/index.ts"
import { bech32AddressToHex } from "../convert.ts"
import { createPfmMemo, getHubbleChainDetails } from "../pfm.ts"
import { fallback, createClient, type HttpTransport } from "viem"
import type { OfflineSigner, TransferAssetsParameters } from "../types.ts"

export const cosmosChainId = [
  "mocha-4",
  "elgafar-1",
  "osmo-test-5",
  "union-testnet-8",
  "union-testnet-9",
  "stride-internal-1",
  "bbn-test-5"
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
      memo,
      amount,
      receiver,
      denomAddress,
      destinationChainId,
      relayContractAddress,
      account = parameters.account,
      gasPrice = parameters.gasPrice
    }: TransferAssetsParameters<CosmosChainId>): Promise<Result<string, Error>> => {
      const sourceChainId = parameters.chainId
      const rpcUrl = parameters.transport({}).value?.url

      if (!rpcUrl) return err(new Error("No cosmos RPC URL found"))
      if (!account) return err(new Error("No cosmos signer found"))
      if (!gasPrice) return err(new Error("No gas price found"))

      if (sourceChainId === destinationChainId) {
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
      const chainDetails = await getHubbleChainDetails({
        sourceChainId,
        destinationChainId
      })

      if (chainDetails.isErr()) return err(chainDetails.error)

      // if (chainDetails.value.transferType === "pfm") {
      //   if (!chainDetails.value.port) return err(new Error("Port not found in hubble"))
      //   const pfmMemo = createPfmMemo({
      //     port: chainDetails.value.port,
      //     channel: chainDetails.value.destinationChannel.toString(),
      //     receiver: cosmosChainId.includes(destinationChainId)
      //       ? bech32AddressToHex({ address: receiver })
      //       : receiver
      //   })
      //   if (pfmMemo.isErr()) return err(pfmMemo.error)
      //   memo = pfmMemo.value
      // }

      const sourceChannel = chainDetails.value.sourceChannel
      relayContractAddress ??= chainDetails.value.relayContractAddress

      // if (sourceChainId === "union-testnet-9") {
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
                channel_id: sourceChannel,
                receiver: receiver,
                base_token: denomAddress,
                base_amount: amount,
                quote_token: "muno",
                quote_amount: amount,
                timeout_height: 0,
                timeout_timestamp: "18446744073709551610",
                salt: "0x69fce040a41930d779c972da6cc8b8b418d86e1e41199f51ec71c864e1412099" //TODO: don't hardcode
                // memo: memo ?? `${stamp} Sending ${amount} ${denomAddress} to ${receiver}`
              }
            },
            funds: [{ amount: amount.toString(), denom: denomAddress }]
          }
        ]
      })
      return transfer
      // }

      // if (destinationChainId === "union-testnet-8") {
      //   if (!sourceChannel) return err(new Error("Source channel not found"))

      //   const [account_] = await account.getAccounts()
      //   if (!account) return err(new Error("No account found"))

      //   const transfer = await ibcTransfer({
      //     account,
      //     rpcUrl,
      //     gasPrice,
      //     messageTransfers: [
      //       {
      //         sourceChannel: sourceChannel.toString(),
      //         sourcePort: "transfer",
      //         sender: account_?.address,
      //         token: { denom: denomAddress, amount: amount.toString() },
      //         timeoutHeight: { revisionHeight: 888_888_888n, revisionNumber: 8n },
      //         receiver: receiver.startsWith("0x") ? receiver.slice(2) : receiver,
      //         memo: memo ?? `${stamp} Sending ${amount} ${denomAddress} to ${receiver}`
      //       }
      //     ]
      //   })
      //   return transfer
      // }

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

      if (sourceChainId === destinationChainId) {
        return await cosmosSameChainTransferSimulate({
          receiver,
          account,
          rpcUrl,
          asset: { denom: denomAddress, amount: amount.toString() },
          gasPrice: gasPrice ?? { amount: "0.0025", denom: "muno" }
        })
      }

      const chainDetails = await getHubbleChainDetails({
        sourceChainId,
        destinationChainId
      })

      if (chainDetails.isErr()) return err(chainDetails.error)

      if (chainDetails.value.transferType === "pfm") {
        if (!chainDetails.value.port) return err(new Error("Port not found in hubble"))
        const pfmMemo = createPfmMemo({
          port: chainDetails.value.port,
          channel: chainDetails.value.destinationChannel.toString(),
          receiver: cosmosChainId.includes(destinationChainId)
            ? bech32AddressToHex({ address: receiver })
            : receiver
        })
        if (pfmMemo.isErr()) return err(pfmMemo.error)
        memo = pfmMemo.value
      }
      const sourceChannel = chainDetails.value.sourceChannel
      // destinationChainId = chainDetails.value.destinationChainId
      relayContractAddress ??= chainDetails.value.relayContractAddress

      if (sourceChainId === "union-testnet-9") {
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
                  memo: memo ?? `${stamp} Sending ${amount} ${denomAddress} to ${receiver}`
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
              sourceChannel: sourceChannel.toString(),
              sourcePort: "transfer",
              sender: account_?.address,
              token: { denom: denomAddress, amount: amount.toString() },
              timeoutHeight: { revisionHeight: 888_888_888n, revisionNumber: 8n },
              receiver: receiver.startsWith("0x") ? receiver.slice(2) : receiver,
              memo: memo ?? `${stamp} Sending ${amount} ${denomAddress} to ${receiver}`
            }
          ]
        })
      }

      return err(new Error("Unsupported network"))
    }
  }))
