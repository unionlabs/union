import { raise, timestamp } from "./utilities.ts"
import { SigningCosmWasmClient, type ExecuteInstruction } from "@cosmjs/cosmwasm-stargate"
import type { GasPrice, DeliverTxResponse, MsgTransferEncodeObject } from "@cosmjs/stargate"
import type {
  MessageTransferWithOptionals,
  OfflineSigner as CosmosOfflineSigner,
  Coin
} from "./types.ts"

/**
 * Transfer tokens from ibc-enabled chain to another ibc-enabled chain
 */
export async function ibcTransfer({
  messageTransfers,
  cosmosSigner,
  cosmosRpcUrl,
  gasPrice
}: {
  messageTransfers: Array<MessageTransferWithOptionals>
  cosmosSigner: CosmosOfflineSigner
  cosmosRpcUrl: string
  gasPrice: GasPrice
}): Promise<DeliverTxResponse> {
  const [account] = await cosmosSigner.getAccounts()
  if (!account) raise("No account found")
  const signingClient = await SigningCosmWasmClient.connectWithSigner(cosmosRpcUrl, cosmosSigner, {
    gasPrice
  })

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
  return response
}

/**
 * Transfer tokens where the source is Union
 */
export async function cosmwasmTransfer({
  instructions,
  cosmosSigner,
  cosmosRpcUrl,
  gasPrice
}: {
  instructions: Array<ExecuteInstruction>
  cosmosSigner: CosmosOfflineSigner
  cosmosRpcUrl: string
  gasPrice: GasPrice
}) {
  const [account] = await cosmosSigner.getAccounts()
  if (!account) raise("No account found")
  const signingClient = await SigningCosmWasmClient.connectWithSigner(cosmosRpcUrl, cosmosSigner, {
    gasPrice
  })
  const response = await signingClient.executeMultiple(account.address, instructions, "auto")
  return response
}

/**
 * Transfer tokens where where source and destination are the same chain
 */
export async function cosmosTransfer({
  receiver,
  cosmosSigner,
  cosmosRpcUrl,
  gasPrice,
  asset
}: {
  receiver: string
  cosmosSigner: CosmosOfflineSigner
  cosmosRpcUrl: string
  gasPrice: GasPrice
  asset: Coin
}) {
  const [account] = await cosmosSigner.getAccounts()
  if (!account) raise("No account found")
  const signingClient = await SigningCosmWasmClient.connectWithSigner(cosmosRpcUrl, cosmosSigner, {
    gasPrice
  })
  const stamp = timestamp()
  const response = await signingClient.sendTokens(
    account.address,
    receiver,
    [asset],
    "auto",
    `${stamp} Sending ${asset.amount} ${asset.denom} to ${receiver}`
  )
  return response
}
