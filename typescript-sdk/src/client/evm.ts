import {
  erc20Abi,
  type Hex,
  getAddress,
  type Account,
  publicActions,
  type HttpTransport,
  createWalletClient,
  type CustomTransport,
  type FallbackTransport
} from "viem"
import {
  transferAssetFromEvm,
  approveTransferAssetFromEvm,
  transferAssetFromEvmSimulate
} from "../transfer/evm.ts"
import { cosmosChainId } from "./cosmos.ts"
import { err, ok, type Result } from "neverthrow"
import { bech32AddressToHex } from "../convert.ts"
import type { TransferAssetsParameters } from "./types.ts"
import { createPfmMemo, getHubbleChainDetails } from "../pfm.ts"
import { sepolia, scrollSepolia, arbitrumSepolia, berachainTestnetbArtio } from "viem/chains"

export { sepolia, scrollSepolia, arbitrumSepolia, berachainTestnetbArtio }
export const evmChains = [sepolia, scrollSepolia, arbitrumSepolia, berachainTestnetbArtio] as const
export const evmChainId: ReadonlyArray<`${(typeof evmChains)[number]["id"]}`> = [
  `${sepolia.id}`,
  `${scrollSepolia.id}`,
  `${arbitrumSepolia.id}`,
  `${berachainTestnetbArtio.id}`
] as const
export type EvmChainId = `${(typeof evmChainId)[number]}`

export interface EvmClientParameters {
  chainId: EvmChainId
  account?: `0x${string}` | Account | undefined
  transport: FallbackTransport | HttpTransport | CustomTransport
}

export const chainIdToChain = (chainId: EvmChainId) =>
  [sepolia, scrollSepolia, arbitrumSepolia, berachainTestnetbArtio].find(
    chain => chain.id === Number(chainId)
  )

export const createEvmClient = (parameters: EvmClientParameters) => {
  return createWalletClient({
    ...parameters,
    chain: chainIdToChain(parameters.chainId)
  })
    .extend(publicActions)
    .extend(client => ({
      transferAsset: async ({
        amount,
        account,
        recipient,
        denomAddress,
        simulate = true,
        destinationChainId,
        autoApprove = false
      }: TransferAssetsParameters<EvmChainId>): Promise<Result<Hex, Error>> => {
        account ||= client.account

        const pfmDetails = await getHubbleChainDetails({
          destinationChainId,
          sourceChainId: parameters.chainId
        })

        if (pfmDetails.isErr()) return err(pfmDetails.error)

        const pfmMemo = createPfmMemo({
          channel: pfmDetails.value.destinationChannel,
          port: `${pfmDetails.value.port}`,
          receiver: cosmosChainId.includes(destinationChainId)
            ? bech32AddressToHex({ address: `${recipient}` })
            : `${recipient}`
        })

        if (pfmMemo.isErr()) return err(pfmMemo.error)

        return await transferAssetFromEvm(client, {
          amount,
          account,
          autoApprove,
          simulate,
          recipient,
          denomAddress,
          destinationChainId,
          memo: pfmMemo.value,
          sourceChannel: pfmDetails.value.sourceChannel,
          relayContractAddress: getAddress(pfmDetails.value.relayContractAddress)
        })
      },
      approveTransaction: async ({
        amount,
        account,
        denomAddress,
        simulate = true,
        destinationChainId
      }: TransferAssetsParameters<EvmChainId>): Promise<Result<Hex, Error>> => {
        const ucsDetails = await getHubbleChainDetails({
          destinationChainId,
          sourceChainId: parameters.chainId
        })
        if (ucsDetails.isErr()) return err(ucsDetails.error)

        return await approveTransferAssetFromEvm(client, {
          amount,
          account,
          simulate,
          denomAddress,
          relayContractAddress: getAddress(ucsDetails.value.relayContractAddress)
        })
      },
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
          destinationChainId,
          sourceChainId: parameters.chainId
        })

        if (pfmDetails.isErr()) return err(pfmDetails.error)

        const pfmMemo = createPfmMemo({
          channel: pfmDetails.value.destinationChannel,
          port: `${pfmDetails.value.port}`,
          receiver: cosmosChainId.includes(destinationChainId)
            ? bech32AddressToHex({ address: `${recipient}` })
            : `${recipient}`
        })

        if (pfmMemo.isErr()) return err(pfmMemo.error)

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
