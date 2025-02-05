import { erc20ReadMulticall } from "$lib/queries/balance/evm/multicall"
import type { Chain } from "$lib/types"
import { writable, type Writable } from "svelte/store"
import { isAddress, type Address } from "viem"

export type ChainId = string
export type Denom = string
export type Balance =
  | { kind: "loading" }
  | { kind: "balance"; amount: string | null; timestamp: number }
  | { kind: "error"; error: string; timestamp: number }

type Balances = Record<ChainId, Record<Denom, Balance>>
export let balances: Writable<Balances> = writable({})

export function updateBalance(chain: ChainId, denom: Denom, balance: Balance) {
  balances.update(val => {
    val = updateBalanceObject(chain, denom, balance, val)
    return val
  })
}

export function deleteBalancesForRpcType(chains: Array<Chain>, rpcType: string) {
  balances.update(val => {
    chains.filter(chain => chain.rpc_type === rpcType).forEach(chain => delete val[chain.chain_id])
    return val
  })
}

function updateBalanceObject(
  chain: ChainId,
  denom: Denom,
  balance: Balance,
  balances: Balances
): Balances {
  if (balances[chain] === undefined) {
    balances[chain] = {}
  }
  balances[chain][denom] = balance
  return balances
}

export async function queryBalances(chain: Chain, address: string) {
  switch (chain.rpc_type) {
    case "evm":
      await updateBalancesEvm(chain, address as Address)
      break
    case "cosmos":
      console.error("cosmos balance fetching currently unsupported")
      break
    case "aptos":
      console.error("aptos balance fetching currently unsupported")
      break
    default:
      console.error("invalid rpc type in balance fetching")
  }
}

export async function updateBalancesEvm(chain: Chain, address: Address) {
  const denoms = chain.tokens.filter(tokens => isAddress(tokens.denom)).map(token => token.denom)
  balances.update(val => {
    denoms.forEach(denom => updateBalanceObject(chain.chain_id, denom, { kind: "loading" }, val))
    return val
  })

  const multicallResults = await erc20ReadMulticall({
    chainId: chain.chain_id,
    functionNames: ["balanceOf"],
    address: address,
    contractAddresses: denoms as Array<Address>
  })

  balances.update(val => {
    multicallResults.forEach((result, index) => {
      let balance: Balance =
        result.balance !== undefined && result.balance.toString().length > 0
          ? { kind: "balance", amount: result.balance.toString(), timestamp: Date.now() }
          : { kind: "balance", amount: null, timestamp: Date.now() }
      val = updateBalanceObject(chain.chain_id, denoms[index], balance, val)
    })
    return val
  })
}

export async function updateBalancesCosmos(chain: Chain, address: Address) {
  const denoms = chain.tokens.filter(tokens => isAddress(tokens.denom)).map(token => token.denom)
  balances.update(val => {
    denoms.forEach(denom => updateBalanceObject(chain.chain_id, denom, { kind: "loading" }, val))
    return val
  })

  const multicallResults = await erc20ReadMulticall({
    chainId: chain.chain_id,
    functionNames: ["balanceOf"],
    address: address,
    contractAddresses: denoms as Array<Address>
  })

  balances.update(val => {
    multicallResults.forEach((result, index) => {
      let balance: Balance =
        result.balance !== undefined && result.balance.toString().length > 0
          ? { kind: "balance", amount: result.balance.toString(), timestamp: Date.now() }
          : { kind: "balance", amount: null, timestamp: Date.now() }
      val = updateBalanceObject(chain.chain_id, denoms[index], balance, val)
    })
    return val
  })
}
