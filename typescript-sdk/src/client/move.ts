import { createClient, fallback, type HttpTransport } from "viem"
import { err, type Result } from "neverthrow"
import type { TransferAssetsParameters } from "./types.ts"
import { cosmosChainId } from "./cosmos.ts"
import { consola } from "scripts/logger"
import { transferAssetFromMove } from "src/transfer/move.ts" // Import the Move transfer function
import type { Account } from "@aptos-labs/ts-sdk"
import { createPfmMemo, getHubbleChainDetails } from "#pfm.ts"
import { bech32AddressToHex } from "#convert.ts"

// Define the list of supported Move chains
export const moveChainId = ["2"] as const

export type MoveChainId = `${(typeof moveChainId)[number]}`

export interface MoveClientParameters {
  chainId: MoveChainId
  account?: Account
  transport: HttpTransport
}

/* trunk-ignore(biome/lint/nursery/useExplicitFunctionReturnType) */
export const createMoveClient = (parameters: MoveClientParameters) =>
  createClient({ transport: fallback([]) }).extend(client => ({
    transferAsset: async ({
      memo,
      amount,
      receiver,
      denomAddress,
      destinationChainId,
      account = parameters.account,
      gasPrice = parameters.gasPrice
    }: TransferAssetsParameters<MoveChainId>): Promise<Result<string, Error>> => {
      const rpcUrl = parameters.transport({}).value?.url
      if (!rpcUrl) return err(new Error("No Move RPC URL found"))
      if (!account) return err(new Error("No Move account found"))
      consola.info(`Move client created for chainId: ${parameters.chainId}`)
      consola.info(`RPC URL: ${rpcUrl}`)
      consola.info(`account: ${account}`)

      // const chainDetails = await getHubbleChainDetails({
      //   destinationChainId,
      //   sourceChainId: parameters.chainId
      // })

      // if (chainDetails.isErr()) return err(chainDetails.error)

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

      // const sourceChannel = chainDetails.value.sourceChannel
      // const relayContractAddress = chainDetails.value.relayContractAddress
      const sourceChannel = "channel-0"
      const relayContractAddress =
        "0x52570c4292730a9d81aead22ac75d4bfca3f23d788f679ce72a11ca3fa7d6762"

      // priv key: 0xe992615114d70429d2920c9d106ac55ec16d9d36a5a017f14f9ee77a85f02467
      // account addr: 0xe3579557fd55ed8fab0d1e211eb1c05d56d74650e7070b703925493c38fe2aed
      // Transfer asset using the previously defined transferAssetFromMove
      const result = await transferAssetFromMove({
        memo,
        amount,
        account,
        receiver,
        denomAddress,
        sourceChannel,
        relayContractAddress,
        baseUrl: rpcUrl // Pass the fetched RPC URL here as the base URL
      })
      if (result.isErr()) {
        return err(new Error(`Move transfer failed: ${result.error.message}`))
      }
      return result // Return the success or error result from transferAssetFromMove
    }
  }))
