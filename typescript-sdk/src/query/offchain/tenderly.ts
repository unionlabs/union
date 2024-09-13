import { ofetch } from "ofetch"
import { ucs01RelayAbi } from "../../abi/ucs-01.ts"
import type { ChainId } from "../../client/types.ts"
import { bech32AddressToHex } from "../../convert.ts"
import { encodeFunctionData, getAddress, type Address } from "viem"

/**
 * get tenderly credentials
 * @example
 * ```ts
 * const { url, key } = getTenderlyCreds({
 *   account: "my-tenderly-account",
 *   project: "my-tenderly-project",
 * })
 * ```
 * @note WIP
 */
function getTenderlyCreds(slug: {
  account: string
  project: string
}): {
  url: string
  key: string
} {
  let url = "https://api.tenderly.co/api/v1/account"
  if (process["env"] === undefined || process["env"]?.["TENDERLY_URL"] === undefined) {
    return {
      key: "",
      url: `${url}/${slug.account}/project/${slug.project}`
    }
  }
  return {
    url: process["env"]?.["TENDERLY_URL"],
    key: ""
  }
}

/**
 * simulate a transaction on evm using Tenderly API
 * @example
 * ```ts
 * const gas = await simulateTransaction({
 *   memo: "test",
 *   amount: 1n,
 *   account: evmAccount,
 *   sourceChannel: "channel-1",
 *   receiver: "0x8478B37E983F520dBCB5d7D3aAD8276B82631aBd",
 *   denomAddress: "0x779877A7B0D9E8603169DdbD7836e478b4624789",
 *   relayContractAddress: "0x2222222222222222222222222222222222222222",
 * })
 * ```
 */
export async function simulateTransaction({
  memo,
  amount,
  account,
  receiver,
  denomAddress,
  sourceChannel,
  sourceChainId,
  destinationChainId,
  relayContractAddress
}: {
  memo?: string
  amount: bigint
  receiver: string
  account?: Address
  denomAddress: Address
  sourceChannel: string
  sourceChainId?: ChainId
  destinationChainId?: ChainId
  relayContractAddress: Address
}) {
  let { url: TENDERLY_URL, key: TENDERLY_KEY } = getTenderlyCreds({
    project: "project",
    account: "amor-fati"
  })

  // @note throwaway key
  TENDERLY_KEY ||= "zQs9t0eoXQybyVbGfV4dSihLElP0Uyl1"

  const queryHeaders = new Headers({
    Accept: "application/json",
    "User-Agent": "typescript-sdk",
    "Content-Type": "application/json",
    "X-Access-Key": TENDERLY_KEY
  })

  const tenderlyRequest = ofetch.create({
    retry: 2,
    retryDelay: 500,
    timeout: 6_000,
    headers: queryHeaders,
    baseURL: TENDERLY_URL
  })

  const encodedFunctionData = encodeFunctionData({
    abi: ucs01RelayAbi,
    functionName: "send",
    args: [
      sourceChannel,
      receiver.startsWith("0x") ? getAddress(receiver) : bech32AddressToHex({ address: receiver }),
      [{ denom: denomAddress, amount }],
      memo ?? "",
      { revision_number: 9n, revision_height: BigInt(999_999_999) + 100n },
      0n
    ]
  })

  const data = await tenderlyRequest<TenderlySimulationResponse>("/simulate", {
    method: "POST",
    body: JSON.stringify({
      save: true,
      estimate_gas: true,
      save_if_fails: false,
      network_id: "11155111",
      simulation_type: "quick",
      from: account,
      to: relayContractAddress,
      input: encodedFunctionData,
      gas: 8000000,
      gas_price: 0,
      value: amount.toString()
    })
  })

  return data.simulation.gas_used || data.transaction.gas_used
}

export interface TenderlySimulationResponse {
  transaction: Transaction
  simulation: { gas_used: number }
}

interface Transaction {
  hash: string
  block_number: number
  from: string
  gas: number
  gas_price: number
  gas_fee_cap: number
  gas_tip_cap: number
  cumulative_gas_used: number
  gas_used: number // one we care about
  effective_gas_price: number
  input: string
  nonce: number
  to: string
  index: number
  value: string
  access_list: null
  status: boolean
  addresses: null
  contract_ids: null
  network_id: string
  timestamp: string
  function_selector: string
  l1_block_number: number
  l1_timestamp: number
  deposit_tx: boolean
  system_tx: boolean

  sig: {
    v: string
    r: string
    s: string
  }

  transaction_info: {
    contract_id: string
    block_number: number
    transaction_id: string
    contract_address: string
    method: string
    parameters: string
    intrinsic_gas: number
    refund_gas: number
    call_trace: {
      hash: string
      contract_name: string
      function_pc: number
      function_op: string
      absolute_position: number
      caller_pc: number
      caller_op: string
      call_type: string
      address: string
      from: string
      from_balance: string
      to: string
      to_balance: string
      value: string
      block_timestamp: string
      gas: number
      gas_used: number
      intrinsic_gas: number
      storage_address: string
      input: string
      nonce_diff: Array<{
        address: string
        original: string
        dirty: string
      }>
      output: string
      decoded_output: string
      error_absolute_position: number
      error: string
      error_op: string
      network_id: string
      calls: Array<{
        hash: string
        function_pc: number
        function_op: string
        absolute_position: number
        caller_pc: number
        caller_op: string
        call_type: string
        address: string
        from: string
        from_balance: string
        to: string
        to_balance: string
        value: string
        caller: {
          address: string
          balance: string
        }
        block_timestamp: string
        gas: number
        gas_used: number
        refund_gas: number
        storage_address: string
        input: string
        output: string
        decoded_output: string
        error_absolute_position: number
        error: string
        error_op: string
        network_id: string
        calls: null
      }>
    }
  }
}
