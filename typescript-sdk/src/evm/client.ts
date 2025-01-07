import {
  erc20Abi,
  getAddress,
  type Account,
  publicActions,
  type HttpTransport,
  createWalletClient,
  type CustomTransport,
  type FallbackTransport
} from "viem"
import {
  evmSameChainTransfer,
  transferAssetFromEvm,
  evmApproveTransferAsset,
  transferAssetFromEvmSimulate
} from "./transfer.ts"
import { err, ok, type Result } from "neverthrow"
import { bech32AddressToHex } from "../convert.ts"
import { cosmosChainId } from "../cosmos/client.ts"
import { createPfmMemo, getHubbleChainDetails } from "../pfm.ts"
import {
  sepolia,
  holesky,
  scrollSepolia,
  arbitrumSepolia,
  berachainTestnetbArtio
} from "viem/chains"
import type { TransferAssetsParameters, LooseAutocomplete, Hex, HexAddress } from "../types.ts"
export { sepolia, scrollSepolia, arbitrumSepolia, berachainTestnetbArtio }

export const evmChains = [
  sepolia,
  holesky,
  scrollSepolia,
  arbitrumSepolia,
  berachainTestnetbArtio
] as const

export const evmChainId = [
  `${sepolia.id}`,
  `${holesky.id}`,
  `${scrollSepolia.id}`,
  `${arbitrumSepolia.id}`,
  `${berachainTestnetbArtio.id}`
] as const

export type EvmChainId = `${(typeof evmChainId)[number]}`

export function evmChainFromChainId(chainId: LooseAutocomplete<EvmChainId>) {
  return evmChains.find(c => `${c.id}` === chainId)
}

export interface EvmClientParameters {
  chainId: EvmChainId
  account?: `0x${string}` | Account | undefined
  transport: FallbackTransport | HttpTransport | CustomTransport
}

export const createEvmClient = (parameters: EvmClientParameters) => {
  return createWalletClient({
    ...parameters,
    chain: evmChainFromChainId(parameters.chainId)
  })
    .extend(publicActions)
    .extend(client => ({
      transferAsset: async ({
        memo,
        amount,
        account,
        receiver,
        denomAddress,
        simulate = true,
        destinationChainId,
        autoApprove = false,
        relayContractAddress
      }: TransferAssetsParameters<EvmChainId>): Promise<Result<Hex, Error>> => {
        account ||= client.account
        console.log(`EVM client created for chainId: ${parameters.chainId}`)
        // first check if chain ids are the same, if yes then we can skip the hubble check and do a simple erc20 transfer
        if (parameters.chainId === destinationChainId) {
          const transfer = await evmSameChainTransfer(client, {
            amount,
            account,
            simulate,
            receiver,
            denomAddress
          })
          if (transfer.isErr()) return err(transfer.error)
          return ok(transfer.value)
        }

        const chainDetails = await getHubbleChainDetails({
          destinationChainId,
          sourceChainId: parameters.chainId
        })

        if (chainDetails.isErr()) return err(chainDetails.error)

        // if (chainDetails.value.transferType === "pfm") {
        //   if (!chainDetails.value.port) return err(new Error("Port not found in hubble"))
        //   const pfmMemo = createPfmMemo({
        //     channel: chainDetails.value.destinationChannel,
        //     port: chainDetails.value.port,
        //     receiver: cosmosChainId.includes(destinationChainId)
        //       ? bech32AddressToHex({ address: receiver })
        //       : receiver
        //   })

        //   if (pfmMemo.isErr()) return err(pfmMemo.error)
        //   memo = pfmMemo.value
        // }

        const sourceChannel = chainDetails.value.sourceChannel
        relayContractAddress ??= getAddress(chainDetails.value.relayContractAddress)

        return await transferAssetFromEvm(client, {
          memo,
          amount,
          account,
          simulate,
          receiver,
          autoApprove,
          denomAddress,
          sourceChannel,
          relayContractAddress
        })
      },
      approveTransaction: async ({
        amount,
        account,
        receiver,
        denomAddress,
        simulate = true,
        destinationChainId
      }: TransferAssetsParameters<EvmChainId>): Promise<Result<Hex, Error>> => {
        let _receiver: HexAddress

        // check if chain ids are the same, if yes then `receiver` is `receiver`,
        // otherwise, it's the relayer contract address from ucs config
        if (parameters.chainId !== destinationChainId) {
          // TODO: don't hardcode
          const ucsDetails = await getHubbleChainDetails({
            destinationChainId,
            sourceChainId: parameters.chainId
          })
          if (ucsDetails.isErr()) return err(ucsDetails.error)
          _receiver = ucsDetails.value.relayContractAddress as `0x${string}`
        } else _receiver = getAddress(receiver)

        return await evmApproveTransferAsset(client, {
          amount,
          account,
          simulate,
          denomAddress,
          receiver: _receiver
        })
      },
      simulateTransaction: async ({
        memo,
        amount,
        receiver,
        denomAddress,
        destinationChainId,
        relayContractAddress
      }: TransferAssetsParameters<EvmChainId>): Promise<Result<string, Error>> => {
        const sourceChainId = parameters.chainId

        if (sourceChainId === destinationChainId) {
          const gas = await client.estimateContractGas({
            abi: erc20Abi,
            account: client.account,
            functionName: "transfer",
            address: getAddress(denomAddress),
            args: [getAddress(receiver), amount]
          })
          return ok(gas.toString())
        }

        const chainDetails = await getHubbleChainDetails({
          sourceChainId: parameters.chainId,
          destinationChainId
        })
        // const chainDetails = {
        //   value: {
        //     sourceChannel: "3",
        //     relayContractAddress: "0x84F074C15513F15baeA0fbEd3ec42F0Bd1fb3efa",
        //     transferType: "direct",
        //     destinationChainId
        //   }
        // }

        if (chainDetails.isErr()) return err(chainDetails.error)

        // if (chainDetails.value.transferType === "pfm") {
        //   if (!chainDetails.value.port) return err(new Error("Port not found in hubble"))
        //   const pfmMemo = createPfmMemo({
        //     port: chainDetails.value.port,
        //     channel: chainDetails.value.destinationChannel,
        //     receiver: cosmosChainId.includes(destinationChainId)
        //       ? bech32AddressToHex({ address: receiver })
        //       : receiver
        //   })

        //   if (pfmMemo.isErr()) return err(pfmMemo.error)
        //   memo = pfmMemo.value
        // }

        const sourceChannel = chainDetails.value.sourceChannel
        relayContractAddress ??= getAddress(chainDetails.value.relayContractAddress)

        if (!sourceChannel) return err(new Error("Source channel not found"))
        if (!relayContractAddress) return err(new Error("Relay contract address not found"))

        return await transferAssetFromEvmSimulate(client, {
          memo,
          amount,
          receiver,
          sourceChannel,
          relayContractAddress,
          denomAddress: getAddress(denomAddress),
          account: typeof client.account === "string" ? client.account : client.account?.address
        })
      }
    }))
}
