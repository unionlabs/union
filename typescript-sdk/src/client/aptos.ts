import {
  aptosSameChainTransfer,
  transferAssetFromAptos,
  aptosSameChainTransferSimulate,
  transferAssetFromAptosSimulate
} from "../transfer/aptos.ts"
import { cosmosChainId } from "./cosmos.ts"
import { err, type Result } from "neverthrow"
import type { Account } from "@aptos-labs/ts-sdk"
import { bech32AddressToHex } from "../convert.ts"
import type { TransferAssetsParameters } from "./types.ts"
import { createPfmMemo, getHubbleChainDetails } from "../pfm.ts"
import { createClient, fallback, type HttpTransport } from "viem"

export const aptosChainId = ["2"] as const

export type AptosChainId = `${(typeof aptosChainId)[number]}`

export interface AptosClientParameters {
  account?: Account
  chainId: AptosChainId
  transport: HttpTransport
}

export const createAptosClient = (parameters: AptosClientParameters) =>
  createClient({ transport: fallback([]) }).extend(_ => ({
    transferAsset: async ({
      memo,
      amount,
      receiver,
      simulate,
      denomAddress,
      destinationChainId,
      relayContractAddress,
      account = parameters.account
    }: TransferAssetsParameters<AptosChainId>): Promise<Result<string, Error>> => {
      const rpcUrl = parameters.transport({}).value?.url

      if (!rpcUrl) return err(new Error("No Aptos RPC URL found"))
      if (!account) return err(new Error("No Aptos account found"))

      if (parameters.chainId === destinationChainId) {
        const transfer = await aptosSameChainTransfer({
          amount,
          account,
          receiver,
          denomAddress,
          baseUrl: rpcUrl
        })
        return transfer
      }

      const chainDetails = await getHubbleChainDetails({
        destinationChainId,
        sourceChainId: parameters.chainId
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

      const result = await transferAssetFromAptos({
        memo,
        amount,
        account,
        receiver,
        simulate,
        denomAddress,
        sourceChannel,
        baseUrl: rpcUrl,
        relayContractAddress
      })
      if (result.isErr()) return err(new Error(`Aptos transfer failed: ${result.error.message}`))

      return result
    },
    simulateTransaction: async ({
      memo,
      amount,
      receiver,
      denomAddress,
      destinationChainId,
      relayContractAddress,
      account = parameters.account
    }: TransferAssetsParameters<AptosChainId>): Promise<Result<string, Error>> => {
      const rpcUrl = parameters.transport({}).value?.url

      if (!rpcUrl) return err(new Error("No Aptos RPC URL found"))
      if (!account) return err(new Error("No Aptos account found"))

      if (parameters.chainId === destinationChainId) {
        return await aptosSameChainTransferSimulate({
          amount,
          account,
          receiver,
          denomAddress,
          baseUrl: rpcUrl
        })
      }

      const chainDetails = await getHubbleChainDetails({
        destinationChainId,
        sourceChainId: parameters.chainId
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

      const result = await transferAssetFromAptosSimulate({
        memo,
        amount,
        account,
        receiver,
        denomAddress,
        sourceChannel,
        baseUrl: rpcUrl,
        relayContractAddress
      })
      if (!result) return err(new Error(`Aptos transfer failed`))

      return result
    }
  }))
