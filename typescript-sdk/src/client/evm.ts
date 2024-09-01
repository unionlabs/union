import {
  erc20Abi,
  getAddress,
  type Account,
  type Address,
  publicActions,
  createWalletClient,
  type FallbackTransport
} from "viem"
import {
  transferAssetFromEvm,
  approveTransferAssetFromEvm,
  transferAssetFromEvmSimulate
} from "../transfer/evm.ts"
import { err, ok, type Result } from "neverthrow"
import { getHubbleChainDetails } from "../pfm.ts"
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

export const createEvmClient = (parameters: EvmClientParameters) => {
  return createWalletClient({ ...parameters, chain: chainIdToChain(parameters.chainId) })
    .extend(publicActions)
    .extend(client => ({
      transferAsset: async ({
        amount,
        account,
        recipient,
        denomAddress,
        approve = false,
        simulate = true,
        destinationChainId
      }: TransferAssetsParameters<EvmChainId>): Promise<Result<string, Error>> => {
        account ||= client.account

        const pfmDetails = await getHubbleChainDetails({
          recipient,
          destinationChainId,
          sourceChainId: parameters.chainId
        })

        if (pfmDetails.isErr()) return err(pfmDetails.error)

        return await transferAssetFromEvm(client, {
          amount,
          account,
          approve,
          simulate,
          recipient,
          denomAddress,
          destinationChainId,
          memo: pfmDetails.value.memo,
          sourceChannel: pfmDetails.value.sourceChannel,
          relayContractAddress: getAddress(pfmDetails.value.relayContractAddress)
        })
      },
      approveTransaction: async ({
        amount,
        account,
        denomAddress,
        simulate = true,
        relayContractAddress
      }: TransferAssetsParameters<EvmChainId> & {
        relayContractAddress: Address
      }): Promise<Result<string, Error>> =>
        await approveTransferAssetFromEvm(client, {
          amount,
          account,
          simulate,
          denomAddress,
          relayContractAddress
        }),
      simulateTransaction: async ({
        memo,
        amount,
        recipient,
        denomAddress,
        destinationChainId
      }: TransferAssetsParameters<EvmChainId>): Promise<Result<string, Error>> => {
        const sourceChainId = parameters.chainId

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

        const pfmDetails = await getHubbleChainDetails({
          recipient,
          sourceChainId,
          destinationChainId
        })

        if (pfmDetails.isErr()) return err(pfmDetails.error)

        const { relayContractAddress, sourceChannel } = pfmDetails.value

        if (!sourceChannel) return err(new Error("Source channel not found"))
        if (!relayContractAddress) return err(new Error("Relay contract address not found"))

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
}
