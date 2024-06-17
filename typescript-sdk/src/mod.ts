export * from "./v0/mod.ts"
import {
  http,
  getAddress,
  type Address,
  type Account,
  publicActions,
  type Transport,
  createWalletClient,
  type WalletClientConfig
} from "viem"
import {
  bech32AddressToHex,
  hexAddressToBech32,
  hexStringToUint8Array,
  uint8ArrayToHexString,
  convertByteArrayToHex
} from "./convert.ts"
import { sepolia } from "viem/chains"
import { timestamp } from "./utilities/index.ts"
import { offchainQuery } from "./query/off-chain.ts"
import { transferAssetFromEvm } from "./transfer/evm.ts"
import { cosmosHttp, rankCosmosRpcProviders } from "./transport.ts"
import type { OfflineSigner, TransactionResponse } from "./types.ts"
import { cosmosSameChainTransfer, cosmwasmTransfer, ibcTransfer } from "./transfer/cosmos.ts"
import { truncateAddress, isValidEvmAddress, isValidBech32Address } from "./utilities/address.ts"

export {
  /**
   * We export this as a standalone so that it can be used to fetch data that get passed to `createUnionClient`
   */
  offchainQuery,
  cosmosHttp
}

export interface EvmClientParameters extends WalletClientConfig {}

export interface CosmosClientParameters {
  account: OfflineSigner
  gasPrice?: { amount: string; denom: string }
  transport: ReturnType<typeof cosmosHttp> | Array<ReturnType<typeof cosmosHttp>>
}

export function createUnionClient({
  evm,
  cosmos
}: {
  evm: EvmClientParameters
  cosmos: CosmosClientParameters
}) {
  const chain = evm.chain ?? sepolia
  const transport: Transport = evm.transport ?? http("https://rpc2.sepolia.org")

  return createWalletClient({ ...evm, transport, chain, account: evm.account })
    .extend(publicActions)
    .extend(() => ({ offchainQuery }))
    .extend(() => ({
      bech32AddressToHex,
      hexAddressToBech32,
      convertByteArrayToHex,
      hexStringToUint8Array,
      uint8ArrayToHexString,
      truncateAddress,
      isValidEvmAddress,
      isValidBech32Address
    }))
    .extend(client => ({
      transferAssetFromEvm: async ({
        amount,
        account,
        recipient,
        denomAddress,
        sourceChannel,
        simulate = true,
        relayContractAddress
      }: {
        amount: bigint
        account?: Account
        recipient: string
        simulate?: boolean
        denomAddress: Address
        sourceChannel: string
        relayContractAddress: Address
      }): Promise<TransactionResponse> => {
        account ||= client.account
        const transaction = await transferAssetFromEvm(client, {
          amount,
          account,
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
        relayContractAddress,
        evmSigner = evm.account,
        gasPrice = cosmos.gasPrice,
        cosmosSigner = cosmos.account,
        memo = timestamp()
      }: {
        memo?: string
        amount: bigint
        recipient: string
        sourcePort?: string
        denomAddress: string
        sourceChannel: string
        path: [string, string]
        network: "cosmos" | "evm"
        cosmosSigner?: OfflineSigner
        relayContractAddress?: string
        gasPrice?: { amount: string; denom: string }
        evmSigner?: `0x${string}` | Account | undefined
      }): Promise<TransactionResponse> => {
        try {
          if (!path.includes("union-testnet-8")) {
            return {
              success: false,
              data: "Either source or destination chain ID is not union-testnet-8. Must be union-testnet-8 until PFM is implemented"
            }
          }

          const [sourceChainId, destinationChainId] = path

          if (network === "evm") {
            if (!relayContractAddress) {
              return { success: false, data: "Relay contract address not found" }
            }

            evmSigner ||= client.account

            if (!evmSigner) return { success: false, data: "No evm signer found" }

            const transactionHash = await client.transferAssetFromEvm({
              memo,
              amount,
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
            transports: Array.isArray(cosmos.transport)
              ? cosmos.transport.flatMap(t => t({}).value?.url).filter(Boolean)
              : [cosmos.transport({}).value?.url].filter(Boolean),
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
                      memo: `${stamp} Sending ${amount} ${denomAddress} to ${recipient}`,
                      receiver: recipient.startsWith("0x") ? recipient.slice(2) : recipient
                    }
                  },
                  funds: [{ amount: amount.toString(), denom: denomAddress }]
                }
              ]
            })
            return transfer
          }

          if (network === "cosmos" && destinationChainId === "union-testnet-8") {
            if (!sourcePort) return { success: false, data: "Source port not found" }

            const [account] = await cosmosSigner.getAccounts()
            if (!account) return { success: false, data: "No account found" }

            const transfer = await ibcTransfer({
              gasPrice,
              cosmosSigner,
              cosmosRpcUrl,
              messageTransfers: [
                {
                  sourceChannel,
                  // receiver: recipient.startsWith("0x") ? recipient.slice(2) : recipient,
                  receiver: recipient,
                  sender: account.address,
                  sourcePort: sourcePort ?? "transfer",
                  token: { denom: denomAddress, amount: amount.toString() },
                  memo: `${stamp} Sending ${amount} ${denomAddress} to ${recipient}`,
                  timeoutHeight: { revisionHeight: 888_888_888n, revisionNumber: 8n }
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
}
