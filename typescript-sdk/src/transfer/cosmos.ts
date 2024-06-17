import {
  GasPrice,
  SigningStargateClient,
  assertIsDeliverTxSuccess,
  type MsgTransferEncodeObject
} from "@cosmjs/stargate"
import type {
  Coin,
  MessageTransferWithOptionals,
  OfflineSigner as CosmosOfflineSigner
} from "../types.ts"
import { timestamp } from "../utilities/index.ts"
import type { TransactionResponse } from "../types.ts"
import { SigningCosmWasmClient, type ExecuteInstruction } from "@cosmjs/cosmwasm-stargate"

/**
 * TODO:
 * - [ ] prefix logs with context to make it easier to debug
 */

/**
 * Transfer tokens from ibc-enabled chain to another ibc-enabled chain
 */
export async function ibcTransfer({
  gasPrice,
  cosmosSigner,
  cosmosRpcUrl,
  messageTransfers
}: {
  gasPrice: { amount: string; denom: string }
  cosmosRpcUrl: string
  cosmosSigner: CosmosOfflineSigner
  messageTransfers: Array<MessageTransferWithOptionals>
}): Promise<TransactionResponse> {
  try {
    const [account] = await cosmosSigner.getAccounts()
    if (!account) return { success: false, data: "No account found" }

    const signingClient = await SigningStargateClient.connectWithSigner(
      cosmosRpcUrl,
      cosmosSigner,
      { gasPrice: GasPrice.fromString(`${gasPrice.amount}${gasPrice.denom}`) }
    )

    const response = await signingClient.signAndBroadcast(
      account.address,
      messageTransfers.map(
        ({ sender = account.address, timeoutTimestamp = 0n, ...messageTransfer }) => ({
          typeUrl: "/ibc.applications.transfer.v1.MsgTransfer",
          value: { sender, timeoutTimestamp, ...messageTransfer }
        })
      ) satisfies Array<MsgTransferEncodeObject>,
      "auto"
    )

    assertIsDeliverTxSuccess(response)

    signingClient.disconnect()
    return { success: true, data: response.transactionHash }
  } catch (error) {
    console.error(error)
    return {
      success: false,
      data: error instanceof Error ? error.message : "An unknown error occurred"
    }
  }
}

/**
 * Transfer tokens where the source is Union
 */
export async function cosmwasmTransfer({
  gasPrice,
  instructions,
  cosmosSigner,
  cosmosRpcUrl
}: {
  cosmosRpcUrl: string
  cosmosSigner: CosmosOfflineSigner
  instructions: Array<ExecuteInstruction>
  gasPrice: { amount: string; denom: string }
}): Promise<TransactionResponse> {
  try {
    const [account] = await cosmosSigner.getAccounts()
    if (!account) return { success: false, data: "No account found" }

    const signingClient = await SigningCosmWasmClient.connectWithSigner(
      cosmosRpcUrl,
      cosmosSigner,
      { gasPrice: GasPrice.fromString(`${gasPrice.amount}${gasPrice.denom}`) }
    )
    const response = await signingClient.executeMultiple(account.address, instructions, "auto")

    signingClient.disconnect()
    return { success: true, data: response.transactionHash }
  } catch (error) {
    console.error(error)
    return {
      success: false,
      data: error instanceof Error ? error.message : "An unknown error occurred"
    }
  }
}

/**
 * Transfer tokens where where source and destination are the same chain
 */
export async function cosmosSameChainTransfer({
  asset,
  gasPrice,
  recipient,
  cosmosSigner,
  cosmosRpcUrl
}: {
  asset: Coin
  recipient: string
  cosmosRpcUrl: string
  cosmosSigner: CosmosOfflineSigner
  gasPrice: { amount: string; denom: string }
}): Promise<TransactionResponse> {
  try {
    const [account] = await cosmosSigner.getAccounts()
    if (!account) return { success: false, data: "No account found" }

    const signingClient = await SigningStargateClient.connectWithSigner(
      cosmosRpcUrl,
      cosmosSigner,
      { gasPrice: GasPrice.fromString(`${gasPrice.amount}${gasPrice.denom}`) }
    )

    const response = await signingClient.sendTokens(
      account.address,
      recipient,
      [asset],
      "auto",
      `${timestamp()} Sending ${asset.amount} ${asset.denom} to ${recipient}`
    )

    assertIsDeliverTxSuccess(response)

    signingClient.disconnect()
    return { success: true, data: response.transactionHash }
  } catch (error) {
    console.error(error)
    return {
      success: false,
      data: error instanceof Error ? error.message : "An unknown error occurred"
    }
  }
}
