import { Registry } from "@cosmjs/proto-signing"
import { timestamp } from "../utilities/index.ts"
import type { TransactionResponse } from "../types.ts"
import { GasPrice, defaultRegistryTypes } from "@cosmjs/stargate"
import { MsgExecuteContract } from "cosmjs-types/cosmwasm/wasm/v1/tx"
import type { OfflineSigner as CosmosOfflineSigner } from "../types.ts"
import { SigningCosmWasmClient, type ExecuteInstruction } from "@cosmjs/cosmwasm-stargate"

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
 * TODO: fix - currently not working:
 *  "Query failed with (6): rpc error: code = Unknown desc = sender: empty address string is not allowed [CosmWasm/wasmd@v0.51.0/x/wasm/types/tx.go:123] with gas used: '1168': unknown request"
 */
export async function cosmwasmTransferSimulate({
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
    const registry = new Registry([
      ...defaultRegistryTypes,
      ["/cosmwasm.wasm.v1.MsgExecuteContract", MsgExecuteContract]
    ])

    const signingClient = await SigningCosmWasmClient.connectWithSigner(
      cosmosRpcUrl,
      cosmosSigner,
      {
        registry,
        gasPrice: GasPrice.fromString(`${gasPrice.amount}${gasPrice.denom}`)
      }
    )

    const [account] = await cosmosSigner.getAccounts()
    if (!account) return { success: false, data: "No account found" }

    const gas = await signingClient.simulate(
      account.address,
      instructions.map(instruction => ({
        // @ts-expect-error - TODO: why is it not happy?
        value: MsgExecuteContract.fromPartial(instruction),
        typeUrl: "/cosmwasm.wasm.v1.MsgExecuteContract"
      })),
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
