import type { Prettify } from "../types.ts"

type rpcUrlArgument = { rpcUrl: string }
export type RpcQueryPath = "height" | "block" | "transaction" | "net_info" | "health"

const queryHeaders = new Headers({
  Accept: "application/json",
  "User-Agent": "typescript-sdk",
  "Content-Type": "application/json"
})

type CosmosRpcBaseResponse = {
  id: number
  jsonrpc: "2.0"
}

type CosmosTransactionReceipt = {
  tx: string
  hash: string
  index: number
  height: string
  tx_result: {
    log: string
    code: number
    data: string
    info: string
    gas_used: string
    codespace: string
    gas_wanted: string
    events: Array<{
      type: string
      attributes: Array<{ key: string; value: string; index: boolean }>
    }>
  }
}

/**
 * get the current block height
 * @example
 * ```ts
 * const height = await getCosmosHeight({
 *   rpcUrl: "https://rpc.testnet-8.union.build",
 * })
 * ```
 */
export async function getCosmosHeight({ rpcUrl }: { rpcUrl: string }) {
  const response = await fetch(`${rpcUrl}/header`)
  const json = (await response.json()) as { result: { header: { height: string } } }
  return Number.parseInt(json.result.header.height)
}

type CosmosTransactionReceiptResponse = Prettify<
  CosmosRpcBaseResponse & { result: { txs: Array<CosmosTransactionReceipt> } }
>

/**
 * get the transaction receipt for a given transaction hash
 * @example
 * ```ts
 * const receipt = await getCosmosTransactionReceipt({
 *   rpcUrl: "https://rpc.testnet-8.union.build",
 *   hash: "A6E276CE66CDB35C0CAAC49EC9AAB3CB2CF8A34C807A4C729EA385E64C88D69B",
 * })
 * ```
 */
export async function getCosmosTransactionReceipt(params: {
  hash: string
  rpcUrl: string
}): Promise<CosmosTransactionReceiptResponse> {
  const url = `${params.rpcUrl}/tx_search?query="tx.hash='${params.hash}'"`
  const response = await fetch(url, { headers: queryHeaders })
  return (await response.json()) as CosmosTransactionReceiptResponse
}

type CosmosAccountTransactions = Prettify<
  CosmosRpcBaseResponse & {
    result: { total_count: string; txs: Array<CosmosTransactionReceipt> }
  }
>

/**
 * get the transactions sent and received by an address
 * @example
 * ```ts
 * const transactions = await getCosmosAccountTransactions({
 *   address: "union1qp0wtsfltjk9rnvyu3fkdv0s0skp4y5y3py96f",
 *   rpcUrl: "https://rpc.testnet-8.union.build",
 * })
 * ```
 */
export async function getCosmosAccountTransactions({
  address,
  rpcUrl
}: { address: string } & rpcUrlArgument): Promise<{
  total: number
  sent: CosmosAccountTransactions
  received: CosmosAccountTransactions
}> {
  const senderUrl = `${rpcUrl}/tx_search?query="transfer.sender='${address}'"`
  const recipientUrl = `${rpcUrl}/tx_search?query="transfer.recipient='${address}'"`
  const [sent, received] = (await Promise.all([
    fetch(senderUrl, { headers: queryHeaders })
      .then(_ => _.json())
      .catch(),
    fetch(recipientUrl, { headers: queryHeaders })
      .then(_ => _.json())
      .catch()
  ])) as [CosmosAccountTransactions, CosmosAccountTransactions]

  return {
    sent,
    received,
    total: Number.parseInt(sent.result.total_count) + Number.parseInt(received.result.total_count)
  }
}

export async function getAptosAccountTransactions({
  address,
  rpcUrl
}: { address: string; rpcUrl: string }): Promise<Array<AptosTransactionReceipt>> {
  const response = await fetch(`${rpcUrl}/accounts/${address}/transactions`, {
    headers: queryHeaders
  })
  const data = (await response.json()) as
    // TODO: add types
    Array<any> | { error_code: string; message: string; vm_error_code: null }

  if (!Array.isArray(data)) return data as Array<AptosTransactionReceipt>
  return data as Array<AptosTransactionReceipt>
}

interface AptosTransactionReceipt {
  version: string
  hash: string
  state_change_hash: string
  event_root_hash: string
  state_checkpoint_hash: any
  gas_used: string
  success: boolean
  vm_status: string
  accumulator_root_hash: string
  changes: Array<{
    address?: string
    state_key_hash: string
    data?: {
      type: string
      data: {
        balance?: string
        frozen?: boolean
        metadata?: {
          inner: string
        }
        allow_ungated_transfer?: boolean
        guid_creation_num?: string
        owner?: string
        transfer_events?: {
          counter: string
          guid: {
            id: {
              addr: string
              creation_num: string
            }
          }
        }
        coin?: {
          value: string
        }
        deposit_events?: {
          counter: string
          guid: {
            id: {
              addr: string
              creation_num: string
            }
          }
        }
        withdraw_events?: {
          counter: string
          guid: {
            id: {
              addr: string
              creation_num: string
            }
          }
        }
        authentication_key?: string
        coin_register_events?: {
          counter: string
          guid: {
            id: {
              addr: string
              creation_num: string
            }
          }
        }
        key_rotation_events?: {
          counter: string
          guid: {
            id: {
              addr: string
              creation_num: string
            }
          }
        }
        rotation_capability_offer?: {
          for: {
            vec: Array<any>
          }
        }
        sequence_number?: string
        signer_capability_offer?: {
          for: {
            vec: Array<any>
          }
        }
      }
    }
    type: string
    handle?: string
    key?: string
    value?: string
  }>
  sender: string
  sequence_number: string
  max_gas_amount: string
  gas_unit_price: string
  expiration_timestamp_secs: string
  payload: {
    function: string
    type_arguments: Array<string>
    arguments: [
      {
        inner: string
      },
      string,
      string
    ]
    type: string
  }
  signature: {
    public_key: string
    signature: string
    type: string
  }
  events: Array<{
    guid: {
      creation_number: string
      account_address?: string
    }
    sequence_number: string
    type: string
    data: {
      amount?: string
      store?: string
      execution_gas_units?: string
      io_gas_units?: string
      storage_fee_octas?: string
      storage_fee_refund_octas?: string
      total_charge_gas_units?: string
    }
  }>
  timestamp: string
  type: string
}
