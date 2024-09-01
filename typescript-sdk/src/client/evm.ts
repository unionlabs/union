import {
  erc20Abi,
  getAddress,
  type Account,
  publicActions,
  createWalletClient,
  type FallbackTransport
} from "viem"
import {
  transferAssetFromEvm,
  approveTransferAssetFromEvm,
  transferAssetFromEvmSimulate,
  type TransferAssetFromEvmParams,
  type ApproveTransferAssetFromEvmParams
} from "../transfer/evm.ts"
import { err, ok, type Result } from "neverthrow"
import type { TransferAssetsParameters } from "./types.ts"
import { sepolia, scrollSepolia, arbitrumSepolia, berachainTestnetbArtio } from "viem/chains"

export const evmChainId = [
  `${sepolia.id}`,
  `${scrollSepolia.id}`,
  `${arbitrumSepolia.id}`,
  `${berachainTestnetbArtio.id}`
] as const
export type EvmChainId = `${(typeof evmChainId)[number]}`

export interface EvmClientParameters {
  chainId: EvmChainId
  transport: FallbackTransport
  account?: `0x${string}` | Account | undefined
}

export const chainIdToChain = (chainId: EvmChainId) =>
  [sepolia, scrollSepolia, arbitrumSepolia, berachainTestnetbArtio].find(
    chain => chain.id === Number(chainId)
  )

/**
 * TODO: add JSDoc with examples
 */
export const createEvmClient = (parameters: EvmClientParameters) =>
  createWalletClient({ ...parameters, chain: chainIdToChain(parameters.chainId) })
    .extend(publicActions)
    .extend(client => ({
      transferAsset: async ({
        memo,
        amount,
        account,
        recipient,
        denomAddress,
        sourceChannel,
        approve = false,
        simulate = true,
        destinationChainId,
        relayContractAddress
      }: TransferAssetFromEvmParams): Promise<Result<string, Error>> => {
        account ||= client.account
        const transaction = await transferAssetFromEvm(client, {
          memo,
          amount,
          account,
          approve,
          simulate,
          recipient,
          denomAddress,
          sourceChannel,
          destinationChainId,
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
      }: ApproveTransferAssetFromEvmParams): Promise<Result<string, Error>> => {
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
        memo,
        amount,
        recipient,
        denomAddress,
        sourceChannel,
        destinationChainId,
        relayContractAddress
      }: TransferAssetsParameters<EvmChainId>): Promise<Result<string, Error>> => {
        const sourceChainId = parameters.chainId

        if (!sourceChannel) return err(new Error("Source channel not found"))
        if (!relayContractAddress) return err(new Error("Relay contract address not found"))

        if (sourceChainId === destinationChainId) {
          const gas = await client.estimateContractGas({
            abi: erc20Abi,
            account: client.account,
            functionName: "transfer",
            address: getAddress(denomAddress),
            args: [getAddress(recipient), amount]
          })
          return ok(gas.toString())
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
