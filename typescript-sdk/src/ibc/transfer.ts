import {
  GasPrice,
  SigningStargateClient,
  assertIsDeliverTxSuccess,
  type MsgTransferEncodeObject
} from "@cosmjs/stargate"
import type {
  MessageTransferWithOptionals,
  OfflineSigner as CosmosOfflineSigner
} from "../types.ts"
import { timestamp } from "../utilities/index.ts"
import type { TransactionResponse } from "../types.ts"

/**
 * Make ICS-20 transfer:
 * - https://github.com/cosmos/ibc/blob/main/spec/app/ics-020-fungible-token-transfer/README.md
 * - transfer tokens from ibc-enabled chain to another ibc-enabled chain
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

export async function ibcTransferSimulate({
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
  console.info("ibcTransferSimulate")
  try {
    const signingClient = await SigningStargateClient.connectWithSigner(
      cosmosRpcUrl,
      cosmosSigner,
      { gasPrice: GasPrice.fromString(`${gasPrice.amount}${gasPrice.denom}`) }
    )

    const [account] = await cosmosSigner.getAccounts()
    if (!account) return { success: false, data: "No account found" }

    const gas = await signingClient.simulate(
      account.address,
      messageTransfers.map(
        ({ sender = account.address, timeoutTimestamp = 0n, ...messageTransfer }) => ({
          typeUrl: "/ibc.applications.transfer.v1.MsgTransfer",
          value: { sender, timeoutTimestamp, ...messageTransfer }
        })
      ) satisfies Array<MsgTransferEncodeObject>,
      "auto"
    )

    signingClient.disconnect()
    return { success: true, data: gas.toString() }
  } catch (error) {
    console.error(error)
    return {
      success: false,
      data: error instanceof Error ? error.message : "An unknown error occurred"
    }
  }
}
