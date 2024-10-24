import {
  moveSameChainTransfer,
  transferAssetFromMove,
  transferAssetFromMoveSimulate
} from "../transfer/move.ts"
import { err, ok, type Result } from "neverthrow"
import type { Account } from "@aptos-labs/ts-sdk"
import type { TransferAssetsParameters } from "./types.ts"
import { createPfmMemo, getHubbleChainDetails } from "../pfm.ts"
import { createClient, fallback, type HttpTransport } from "viem"

export const moveChainId = ["2"] as const

export type MoveChainId = `${(typeof moveChainId)[number]}`

export interface MoveClientParameters {
  account?: Account
  chainId: MoveChainId
  transport: HttpTransport
}

export const createMoveClient = (parameters: MoveClientParameters) =>
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
    }: TransferAssetsParameters<MoveChainId>): Promise<Result<string, Error>> => {
      const rpcUrl = parameters.transport({}).value?.url

      if (!rpcUrl) return err(new Error("No Move RPC URL found"))
      if (!account) return err(new Error("No Move account found"))
      if (parameters.chainId === destinationChainId) {
        const transfer = await moveSameChainTransfer({
          amount,
          account,
          receiver,
          denomAddress,
          baseUrl: rpcUrl
        })
        if (transfer.isErr()) return err(transfer.error)
        return ok(transfer.value)
      }

      const chainDetails = await getHubbleChainDetails({
        destinationChainId,
        sourceChainId: parameters.chainId
      })
      if (chainDetails.isErr()) return err(chainDetails.error)

      if (chainDetails.value.transferType === "pfm") {
        if (!chainDetails.value.port) return err(new Error("Port not found in hubble"))
        const pfmMemo = createPfmMemo({
          receiver: "TODO",
          port: chainDetails.value.port,
          channel: chainDetails.value.destinationChannel
        })
        if (pfmMemo.isErr()) return err(pfmMemo.error)
        memo = pfmMemo.value
      }

      const sourceChannel = chainDetails.value.sourceChannel
      relayContractAddress ??= chainDetails.value.relayContractAddress

      const result = await transferAssetFromMove({
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
      if (result.isErr()) return err(new Error(`Move transfer failed: ${result.error.message}`))

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
    }: TransferAssetsParameters<MoveChainId>): Promise<Result<string, Error>> => {
      const rpcUrl = parameters.transport({}).value?.url

      if (!rpcUrl) return err(new Error("No Move RPC URL found"))
      if (!account) return err(new Error("No Move account found"))

      const chainDetails = await getHubbleChainDetails({
        destinationChainId,
        sourceChainId: parameters.chainId
      })

      if (chainDetails.isErr()) return err(chainDetails.error)

      if (chainDetails.value.transferType === "pfm") {
        if (!chainDetails.value.port) return err(new Error("Port not found in hubble"))
        const pfmMemo = createPfmMemo({
          receiver: "TODO",
          port: chainDetails.value.port,
          channel: chainDetails.value.destinationChannel
        })

        if (pfmMemo.isErr()) return err(pfmMemo.error)
        memo = pfmMemo.value
      }

      const sourceChannel = chainDetails.value.sourceChannel
      relayContractAddress ??= chainDetails.value.relayContractAddress

      const result = await transferAssetFromMoveSimulate({
        memo,
        amount,
        account,
        receiver,
        denomAddress,
        sourceChannel,
        baseUrl: rpcUrl,
        relayContractAddress
      })
      if (!result) return err(new Error(`Move transfer failed`))

      return result
    }
  }))
