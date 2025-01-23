import {
  erc20Abi,
  getAddress,
  type Account,
  publicActions,
  type HttpTransport,
  createWalletClient,
  type CustomTransport,
  type FallbackTransport,
  createPublicClient,
  http,
  toHex
} from "viem"
import {
  // evmSameChainTransfer,
  transferAssetFromEvm,
  evmApproveTransferAsset,
  transferAssetFromEvmSimulate
} from "./transfer.ts"
import { err, ok, ResultAsync, type Result } from "neverthrow"
import { getHubbleChainDetails } from "../pfm.ts"
import {
  sepolia,
  holesky,
  scrollSepolia,
  arbitrumSepolia,
  berachainTestnetbArtio
} from "viem/chains"
import type { TransferAssetsParameters, LooseAutocomplete, Hex, HexAddress } from "../types.ts"
import { ucs03ZkgmAbi } from "../abi/ucs-03.ts"
import { bech32AddressToHex } from "#mod.ts"
import { generateSalt } from "#utilities/index.ts"
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
      transferAssetNew: async ({
        baseAmount,
        baseToken,
        quoteAmount,
        quoteToken,
        receiver,
        sourceChannelId,
        ucs03address
      }: {
        baseAmount: bigint
        baseToken: string
        quoteAmount: bigint
        quoteToken: string
        receiver: string
        sourceChannelId: number
        ucs03address: string
      }): Promise<Result<Hex, Error>> => {
        if (!client.account) return err(new Error("No account found"))

        /**
         * @dev
         * `UCS03` zkgm contract `transfer` function:
         * - https://github.com/unionlabs/union/blob/0fd24893d4a1173e9c6e150c826c162871d63262/evm/contracts/apps/ucs/03-zkgm/Zkgm.sol#L301
         */
        const writeContractParameters = {
          account: client.account,
          abi: ucs03ZkgmAbi,
          chain: client.chain,
          functionName: "transfer",
          address: ucs03address as `0x${string}`,
          /**
              "channelId": "uint32"
              "receiver": "bytes"
              "baseToken": "address"
              "baseAmount": "uint256"
              "quoteToken": "bytes"
              "quoteAmount": "uint256"
              "timeoutHeight": "uint64"
              "timeoutTimestamp": "uint64"
              "salt": "bytes32"
             */
          args: [
            sourceChannelId,
            receiver.startsWith("0x")
              ? getAddress(receiver)
              : bech32AddressToHex({ address: receiver }),
            baseToken,
            baseAmount,
            quoteToken,
            quoteAmount,
            0n, // TODO: customize timeoutheight
            "0x000000000000000000000000000000000000000000000000fffffffffffffffa", // TODO: make non-hexencoded timestamp
            generateSalt()
          ]
        } as const

        return ResultAsync.fromPromise(client.writeContract(writeContractParameters), error => {
          return new Error("failed to execute evm call", { cause: error })
        }).map(res => res)
      },

      transferAsset: async ({
        amount,
        account,
        receiver,
        denomAddress,
        simulate = true,
        destinationChainId,
        autoApprove = false
      }: TransferAssetsParameters<EvmChainId>): Promise<Result<Hex, Error>> => {
        account ||= client.account
        console.log(`EVM client created for chainId: ${parameters.chainId}`)

        const baseToken = denomAddress

        const chainDetails = await getHubbleChainDetails({
          sourceChainId: parameters.chainId,
          destinationChainId
        })

        if (chainDetails.isErr()) return err(chainDetails.error)

        // TODO: make resillient
        const destinationChainClient = createPublicClient({
          chain: evmChainFromChainId(destinationChainId),
          transport: http()
        })

        // We need to predict the askToken denom based on the sentToken (denomAddress in the transferAssetFromEvm args)
        // we do this by calling the ucs03 instance on the counterparty chain.
        const [quoteToken, _] = (await destinationChainClient.readContract({
          address: chainDetails.value.destinationUCS03Address as `0x${string}`,
          abi: ucs03ZkgmAbi,
          functionName: "predictWrappedToken",
          args: [0, chainDetails.value.destinationChannel, baseToken]
        })) as ["0x${string}", string]

        const sourceChannel = chainDetails.value.sourceChannel
        const ucs03address = getAddress(chainDetails.value.relayContractAddress)

        if (autoApprove) {
          const approveResponse = await evmApproveTransferAsset(client, {
            amount,
            account,
            denomAddress: baseToken,
            receiver: ucs03address
          })
          if (approveResponse.isErr()) {
            return approveResponse
          }
          console.log("approval", approveResponse.value)
        }

        console.log({ sourceChannel, ucs03address, baseToken, quoteToken, amount }) // useful for debugging app

        return await transferAssetFromEvm(client, {
          baseToken,
          baseAmount: amount,
          account,
          simulate,
          receiver,
          quoteToken,
          quoteAmount: amount,
          sourceChannel,
          ucs03address
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
