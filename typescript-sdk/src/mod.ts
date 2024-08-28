import "./patch.ts"
import {
  http,
  fallback,
  erc20Abi,
  getAddress,
  type Address,
  type Account,
  publicActions,
  type Transport,
  createWalletClient,
  type WalletClientConfig
} from "viem"
import {
  byteArrayToHex,
  bech32AddressToHex,
  hexAddressToBech32,
  bytesToBech32Address,
  bech32ToBech32Address,
  hexStringToUint8Array,
  uint8ArrayToHexString
} from "./convert.ts"
import {
  truncateAddress,
  isValidEvmTxHash,
  isValidEvmAddress,
  isValidCosmosTxHash,
  isValidBech32Address
} from "./utilities/address.ts"
import {
  ibcTransfer,
  cosmwasmTransfer,
  ibcTransferSimulate,
  cosmosSameChainTransfer,
  cosmwasmTransferSimulate,
  cosmosSameChainTransferSimulate
} from "./transfer/cosmos.ts"
import {
  transferAssetFromEvm,
  approveTransferAssetFromEvm,
  transferAssetFromEvmSimulate,
  type ApproveTransferAssetFromEvmParams
} from "./transfer/evm.ts"
import { sepolia } from "viem/chains"
import { createPfmMemo } from "./pfm.ts"
import { timestamp } from "./utilities/index.ts"
import { offchainQuery } from "./query/offchain/hubble.ts"
import { cosmosHttp, rankCosmosRpcProviders } from "./transport.ts"
import type { OfflineSigner, TransactionResponse } from "./types.ts"

export {
  /**
   * We export this as a standalone so that it can be used to fetch data that get passed to `createCosmosSdkClient`
   */

  cosmosHttp,
  offchainQuery,
  createPfmMemo,
  byteArrayToHex,
  truncateAddress,
  isValidEvmTxHash,
  isValidEvmAddress,
  bech32AddressToHex,
  hexAddressToBech32,
  isValidCosmosTxHash,
  bytesToBech32Address,
  isValidBech32Address,
  bech32ToBech32Address,
  hexStringToUint8Array,
  uint8ArrayToHexString
}

export type * from "./types.ts"

type Network = "evm" | "cosmos"

export interface EvmClientParameters extends WalletClientConfig {}

export interface CosmosClientParameters {
  account: OfflineSigner
  gasPrice?: { amount: string; denom: string }
  transport: ReturnType<typeof cosmosHttp> | Array<ReturnType<typeof cosmosHttp>>
}

export type TransferAssetsParameters<Network extends "evm" | "cosmos"> = {
  memo?: string
  amount: bigint
  network: Network
  recipient: string
  approve?: boolean
  sourcePort?: string
  denomAddress: string
  path: [string, string]
  sourceChannel?: string
  relayContractAddress?: string
} & (Network extends "evm"
  ? { evmSigner?: `0x${string}` | Account | undefined }
  : { cosmosSigner?: OfflineSigner; gasPrice?: { amount: string; denom: string } })

type EvmParameters<N extends Network = "evm"> = EvmClientParameters & {
  network: N
}

type CosmosParameters<N extends Network = "cosmos"> = CosmosClientParameters & {
  network: N
}

export function createUnionClient(parameters: EvmParameters | CosmosParameters) {
  if (parameters.network === "evm") {
    const chain = parameters.chain ?? sepolia
    const transport: Transport = fallback([
      parameters?.transport ?? http("https://rpc2.sepolia.org")
    ])
    return (
      createWalletClient({
        ...parameters,
        chain,
        transport,
        account: parameters?.account
      })
        // .extend(publicActions)
        // .extend(() => ({
        //   offchainQuery,
        //   byteArrayToHex,
        //   bech32AddressToHex,
        //   hexAddressToBech32,
        //   bech32ToBech32Address,
        //   bytesToBech32Address,
        //   hexStringToUint8Array,
        //   uint8ArrayToHexString,
        //   truncateAddress,
        //   createPfmMemo,
        //   isValidEvmAddress,
        //   isValidBech32Address
        // }))
        .extend(client => ({
          transferAssetFromEvm: async ({
            amount,
            account,
            recipient,
            denomAddress,
            sourceChannel,
            approve = false,
            simulate = true,
            relayContractAddress
          }: {
            amount: bigint
            account?: Account
            recipient: string
            approve?: boolean
            simulate?: boolean
            denomAddress: Address
            sourceChannel: string
            relayContractAddress: Address
          }): Promise<TransactionResponse> => {
            account ||= client.account
            const transaction = await transferAssetFromEvm(client, {
              amount,
              account,
              approve,
              simulate,
              recipient,
              denomAddress,
              sourceChannel,
              relayContractAddress
            })
            return transaction
          }
        }))
        .extend(client => ({
          transferAsset: async ({
            path,
            amount,
            network,
            recipient,
            sourcePort,
            denomAddress,
            sourceChannel,
            approve = false,
            memo = timestamp(),
            relayContractAddress,
            evmSigner = parameters.account
          }: TransferAssetsParameters<Parameters<typeof createUnionClient>[number]["network"]> & {
            evmSigner?: `0x${string}` | Account | undefined
          }): Promise<TransactionResponse> => {
            try {
              const [sourceChainId, destinationChainId] = path
              if (network === "evm") {
                if (!sourceChannel) return { success: false, data: "Source channel not found" }
                if (!relayContractAddress) {
                  return { success: false, data: "Relay contract address not found" }
                }
                evmSigner ||= client.account
                if (!evmSigner) return { success: false, data: "No evm signer found" }
                const transactionHash = await client.transferAssetFromEvm({
                  memo,
                  amount,
                  approve,
                  recipient,
                  sourceChannel,
                  simulate: true,
                  // @ts-expect-error TODO: fix this
                  account: evmSigner,
                  denomAddress: getAddress(denomAddress),
                  relayContractAddress: getAddress(relayContractAddress)
                })
                return transactionHash
              }
              console.info(
                `Transferring ${amount} ${denomAddress} to ${recipient} on ${network} from ${sourceChainId} to ${destinationChainId}`
              )
              const cosmosRpcTransport = await rankCosmosRpcProviders({
                transports: Array.isArray(cosmos?.transport)
                  ? cosmos?.transport.flatMap(t => t({}).value?.url).filter(Boolean)
                  : [cosmos?.transport({}).value?.url].filter(Boolean),
                interval: 1_000,
                sampleCount: 10,
                timeout: 1_000
              }).rank()
              const cosmosRpcUrl = cosmosRpcTransport.at(0)?.rpcUrl
              if (!cosmosSigner) return { success: false, data: "No cosmos signer found" }
              if (!cosmosRpcUrl) return { success: false, data: "No cosmos RPC URL found" }
              if (!gasPrice) return { success: false, data: "No gas price found" }
              if (sourceChainId === "union-testnet-8" && destinationChainId === "union-testnet-8") {
                const transfer = await cosmosSameChainTransfer({
                  gasPrice,
                  recipient,
                  cosmosSigner,
                  cosmosRpcUrl,
                  asset: { denom: denomAddress, amount: amount.toString() }
                })
                return transfer
              }
              const stamp = timestamp()
              if (network === "cosmos" && sourceChainId === "union-testnet-8") {
                if (!sourceChannel) return { success: false, data: "Source channel not found" }
                if (!relayContractAddress) {
                  return { success: false, data: "Relay contract address not found" }
                }
                const transfer = await cosmwasmTransfer({
                  gasPrice,
                  cosmosSigner,
                  cosmosRpcUrl,
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
              if (network === "cosmos" && destinationChainId === "union-testnet-8") {
                if (!sourceChannel) return { success: false, data: "Source channel not found" }
                const [account] = await cosmosSigner.getAccounts()
                if (!account) return { success: false, data: "No account found" }
                sourcePort ||= "transfer"
                const transfer = await ibcTransfer({
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
        .extend(client => ({
          simulateTransaction: async ({
            path,
            memo,
            amount,
            network,
            recipient,
            sourcePort,
            denomAddress,
            sourceChannel,
            relayContractAddress,
            gasPrice = cosmos?.gasPrice,
            cosmosSigner = cosmos?.account,
            evmSigner = typeof evm?.account === "string" ? evm.account : client.account?.address
          }: TransferAssetsParameters): Promise<TransactionResponse> => {
            const [sourceChainId, destinationChainId] = path
            if (network === "evm") {
              evmSigner ||= client.account
              if (!sourceChannel) return { success: false, data: "Source channel not found" }
              if (!relayContractAddress) {
                return { success: false, data: "Relay contract address not found" }
              }
              if (sourceChainId === destinationChainId) {
                const gas = await client.estimateContractGas({
                  abi: erc20Abi,
                  account: evmSigner,
                  functionName: "transfer",
                  address: getAddress(denomAddress),
                  args: [getAddress(recipient), amount]
                })
                return { success: true, data: gas.toString() }
              }
              return await transferAssetFromEvmSimulate(client, {
                memo,
                amount,
                recipient,
                sourceChannel,
                denomAddress: getAddress(denomAddress),
                relayContractAddress: getAddress(relayContractAddress),
                account: typeof evmSigner === "string" ? evmSigner : evmSigner?.address
              })
            }
            if (!(cosmos && Object.hasOwn(cosmos, "transport"))) {
              return { success: false, data: "No cosmos transport found" }
            }
            const cosmosRpcTransport = await rankCosmosRpcProviders({
              transports: Array.isArray(cosmos.transport)
                ? cosmos.transport.flatMap(t => t({}).value?.url).filter(Boolean)
                : [cosmos.transport({}).value?.url].filter(Boolean),
              interval: 1_000,
              sampleCount: 10,
              timeout: 1_000
            }).rank()
            if (!cosmosSigner) return { success: false, data: "No cosmos signer found" }
            const cosmosRpcUrl = cosmosRpcTransport.at(0)?.rpcUrl
            if (!gasPrice) return { success: false, data: "No gas price found" }
            if (!cosmosRpcUrl) return { success: false, data: "No cosmos RPC URL found" }
            if (
              network === "cosmos" &&
              sourceChainId === "union-testnet-8" &&
              destinationChainId === "union-testnet-8"
            ) {
              // Union to Union
              return await cosmosSameChainTransferSimulate({
                recipient,
                cosmosSigner,
                cosmosRpcUrl,
                asset: { denom: denomAddress, amount: amount.toString() },
                gasPrice: gasPrice ?? { amount: "0.0025", denom: "muno" }
              })
            }
            if (network !== "cosmos") return { success: false, data: "Unsupported network" }
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
        .extend(client => ({
          /** evm only */
          approveTransaction: async ({
            amount,
            account,
            denomAddress,
            simulate = true,
            relayContractAddress
          }: ApproveTransferAssetFromEvmParams): Promise<TransactionResponse> => {
            return await approveTransferAssetFromEvm(client, {
              amount,
              account,
              simulate,
              denomAddress,
              relayContractAddress
            })
          }
        }))
    )
  }
}
