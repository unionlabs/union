import {
  erc20Abi,
  getAddress,
  type Address,
  type Account,
  publicActions,
  createWalletClient,
  type WalletClientConfig
} from "viem"
import {
  transferAssetFromEvm,
  approveTransferAssetFromEvm,
  transferAssetFromEvmSimulate,
  type ApproveTransferAssetFromEvmParams
} from "../transfer/evm.ts"
import type { TransactionResponse } from "../types.ts"
import type { TransferAssetsParameters } from "./types.ts"
import { sepolia, scrollSepolia, arbitrumSepolia, berachainTestnetbArtio } from "viem/chains"

export const evmChainId = [
  `${sepolia.id}`,
  `${scrollSepolia.id}`,
  `${arbitrumSepolia.id}`,
  `${berachainTestnetbArtio.id}`
] as const
export type EvmChainId = `${(typeof evmChainId)[number]}`

export interface EvmClientParameters extends WalletClientConfig {
  chainId: EvmChainId
}

/**
 * TODO: add JSDoc with examples
 */
export const createEvmClient = (parameters: EvmClientParameters) =>
  createWalletClient(parameters)
    .extend(publicActions)
    .extend(client => ({
      transferAsset: async ({
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
    .extend(client => ({
      simulateTransaction: async ({
        path,
        memo,
        amount,
        recipient,
        denomAddress,
        sourceChannel,
        relayContractAddress
      }: TransferAssetsParameters<
        EvmClientParameters["chainId"]
      >): Promise<TransactionResponse> => {
        const [sourceChainId, destinationChainId] = path

        if (!sourceChannel) return { success: false, data: "Source channel not found" }
        if (!relayContractAddress) {
          return { success: false, data: "Relay contract address not found" }
        }

        if (sourceChainId === destinationChainId) {
          const gas = await client.estimateContractGas({
            abi: erc20Abi,
            account: client.account,
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
          account: typeof client.account === "string" ? client.account : client.account?.address
        })
      }
    }))
