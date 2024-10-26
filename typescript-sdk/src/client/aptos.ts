import {
  type AptosAccount,
  type AptosAuthAccess,
  aptosTransferSimulate,
  aptosSameChainTransfer,
  transferAssetFromAptos,
  type AptosBrowserWallet,
  type AptosTransferParams,
  waitForTransactionReceipt,
  type AptosPublicAccountInfo
} from "../transfer/aptos.ts"
import { cosmosChainId } from "./cosmos.ts"
import { err, type Result } from "neverthrow"
import { bech32AddressToHex } from "../convert.ts"
import type { TransferAssetsParameters } from "./types.ts"
import { createPfmMemo, getHubbleChainDetails } from "../pfm.ts"
import { Aptos, Network, AptosConfig } from "@aptos-labs/ts-sdk"
import { createClient, fallback, type HttpTransport } from "viem"

export type {
  AptosAccount,
  AptosAuthAccess,
  AptosBrowserWallet,
  AptosTransferParams,
  AptosPublicAccountInfo
}

export const aptosChainId = ["2"] as const

export type AptosChainId = `${(typeof aptosChainId)[number]}`

export interface AptosClientParameters {
  account?: AptosAccount
  chainId: AptosChainId
  transport: HttpTransport
}

export const createAptosClient = (clientParameters: AptosClientParameters) => {
  const rpcUrl = clientParameters.transport({}).value?.url

  if (!rpcUrl) throw new Error("No Aptos RPC URL found")

  const config = new AptosConfig({ fullnode: rpcUrl, network: Network.TESTNET })
  const aptos = new Aptos(config)

  return createClient({ transport: fallback([]) }).extend(_ => ({
    waitForTransactionReceipt: async ({ hash }: { hash: string }) =>
      waitForTransactionReceipt({ aptos, hash }),
    transferAsset: async (
      transferParameters: TransferAssetsParameters<AptosChainId>
    ): Promise<Result<string, Error>> => {
      let {
        memo,
        amount,
        simulate,
        receiver,
        denomAddress,
        destinationChainId,
        relayContractAddress
      } = transferParameters

      if (!transferParameters.account) return err(new Error("No Aptos account found"))
      if (!destinationChainId) return err(new Error("destinationChainId missing"))

      const account = transferParameters.account || clientParameters.account

      if (clientParameters.chainId === destinationChainId) {
        // @ts-expect-error TODO: fix account type
        const transfer = await aptosSameChainTransfer({
          ...transferParameters,
          aptos,
          amount,
          account,
          simulate,
          receiver,
          denomAddress
        })
        return transfer
      }

      const chainDetails = await getHubbleChainDetails({
        destinationChainId,
        sourceChainId: clientParameters.chainId
      })
      if (chainDetails.isErr()) return err(chainDetails.error)

      if (chainDetails.value.transferType === "pfm") {
        if (!chainDetails.value.port) return err(new Error("Port not found in hubble"))
        const pfmMemo = createPfmMemo({
          port: chainDetails.value.port,
          channel: chainDetails.value.destinationChannel,
          /**
           * TODO:
           * check if normal Aptos hex address is valid here or do we need to do some transformation
           */
          receiver: cosmosChainId.includes(destinationChainId)
            ? bech32AddressToHex({ address: receiver })
            : receiver
        })
        if (pfmMemo.isErr()) return err(pfmMemo.error)
        memo = pfmMemo.value
      }

      const sourceChannel = chainDetails.value.sourceChannel
      relayContractAddress ??= chainDetails.value.relayContractAddress

      // @ts-expect-error TODO: fix account type
      return await transferAssetFromAptos({
        ...transferParameters,
        memo,
        aptos,
        amount,
        account,
        simulate,
        receiver,
        denomAddress,
        sourceChannel,
        destinationChainId,
        relayContractAddress
      })
    },
    simulateTransaction: async (
      transferParameters: TransferAssetsParameters<AptosChainId>
    ): Promise<Result<string, Error>> => {
      let {
        memo,
        amount,
        receiver,
        denomAddress,
        autoApprove: _,
        destinationChainId,
        relayContractAddress
      } = transferParameters

      if (!transferParameters.account) return err(new Error("No Aptos account found"))
      if (!destinationChainId) return err(new Error("destinationChainId missing"))

      if (clientParameters.chainId === destinationChainId) {
        return await aptosTransferSimulate({
          aptos,
          path: "SAME_CHAIN",
          ...transferParameters
        })
      }

      const chainDetails = await getHubbleChainDetails({
        destinationChainId,
        sourceChainId: clientParameters.chainId
      })
      if (chainDetails.isErr()) return err(chainDetails.error)

      if (chainDetails.value.transferType === "pfm") {
        if (!chainDetails.value.port) return err(new Error("Port not found in hubble"))
        const pfmMemo = createPfmMemo({
          port: chainDetails.value.port,
          channel: chainDetails.value.destinationChannel,
          /**
           * TODO:
           * check if normal Aptos hex address is valid here or do we need to do some transformation
           */
          receiver: cosmosChainId.includes(destinationChainId)
            ? bech32AddressToHex({ address: receiver })
            : receiver
        })
        if (pfmMemo.isErr()) return err(pfmMemo.error)
        memo = pfmMemo.value
      }
      const sourceChannel = chainDetails.value.sourceChannel
      relayContractAddress ??= chainDetails.value.relayContractAddress

      return await aptosTransferSimulate({
        ...transferParameters,
        path: "CROSS_CHAIN",
        memo,
        aptos,
        amount,
        receiver,
        denomAddress,
        sourceChannel,
        destinationChainId,
        relayContractAddress
      })
    }
  }))
}
