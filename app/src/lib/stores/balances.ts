import { erc20ReadMulticall } from "$lib/queries/balance/evm/multicall"
import * as v from "valibot"
import type { Chain } from "$lib/types"
import { bech32ToBech32Address, isValidBech32ContractAddress } from "@unionlabs/client"
import { writable, type Writable } from "svelte/store"
import { fromHex, isAddress, toHex, type Address } from "viem"
import { fetchJson } from "$lib/utilities/neverthrow"
import { err, ok, ResultAsync } from "neverthrow"
import { CosmWasmClient } from "@cosmjs/cosmwasm-stargate"
import { toast } from "svelte-sonner"

export type ChainId = string
export type Denom = string
export type Balance =
  | { kind: "loading" }
  | { kind: "balance"; amount: string | null; timestamp: number }
  | { kind: "error"; error: string; timestamp: number }

export type Balances = Record<ChainId, Record<Denom, Balance>>
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
      await updateBalancesCosmos(chain, address)
      break
    case "aptos":
      await updateBalancesAptos(chain, address as Address)
      // console.error("aptos balance fetching currently unsupported")
      break
    default:
      console.error("invalid rpc type in balance fetching")
  }
}
export async function updateBalancesAptos(chain: Chain, address: Address) {
  // Optionally mark expected tokens as "loading" (if chain.tokens exists)
  if (chain.tokens && chain.tokens.length > 0) {
    chain.tokens.forEach(token =>
      updateBalance(chain.chain_id, token.denom, { kind: "loading", timestamp: Date.now() })
    )
  }

  // Define the GraphQL query and variables.
  const query = `
    query CoinsData($owner_address: String, $limit: Int, $offset: Int) {
      current_fungible_asset_balances(
        where: {owner_address: {_eq: $owner_address}}
        limit: $limit
        offset: $offset
      ) {
        amount
        asset_type
        metadata {
          name
          decimals
          symbol
          token_standard
        }
      }
    }
  `
  const variables = {
    owner_address: address,
    limit: 200,
    offset: 0
  }

  // Set up the fetch options with appropriate headers.
  const fetchOptions: RequestInit = {
    method: "POST",
    body: JSON.stringify({ query, variables })
  }

  try {
    // Send the request to the Aptos indexer.
    const response = await fetchJson(
      "https://indexer.testnet.movementnetwork.xyz/v1/graphql",
      fetchOptions
    )
    if (response.isErr()) {
      throw new Error(response.error.message)
    }
    const data = response.value.data
    if (!data?.current_fungible_asset_balances) {
      throw new Error("Invalid response data")
    }

    const aptosBalances = data.current_fungible_asset_balances
      .filter((token: any) => token.metadata.token_standard === "v2")
      .map((token: any) => ({
        denom: token.asset_type,
        amount: token.amount
      }))

    console.info("aptosBalances: ", aptosBalances)
    aptosBalances.forEach(token => {
      updateBalance(chain.chain_id, token.denom, {
        kind: "balance",
        amount: token.amount,
        timestamp: Date.now()
      })
    })
  } catch (error: any) {
    console.error("Error fetching Aptos balances", error)
    // On error, update the balances for all tokens with an error state.
    if (chain.tokens?.length > 0) {
      chain.tokens.forEach(token =>
        updateBalance(chain.chain_id, token.denom, {
          kind: "error",
          error: error.message,
          timestamp: Date.now()
        })
      )
    }
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

const cosmosBalancesResponseSchema = v.object({
  balances: v.array(
    v.object({
      denom: v.string(),
      amount: v.string()
    })
  )
})

export async function updateBalancesCosmos(chain: Chain, address: string) {
  const addr = bech32ToBech32Address({
    address: address,
    toPrefix: chain.addr_prefix
  })
  const denoms = chain.tokens.map(token => token.denom.toLowerCase())
  const bankDenoms = denoms.filter(
    denom => !isValidBech32ContractAddress(fromHex(denom as Address, "string"))
  )
  const cw20Denoms = denoms.filter(denom =>
    isValidBech32ContractAddress(fromHex(denom as Address, "string"))
  )

  balances.update(val => {
    denoms.forEach(denom => updateBalanceObject(chain.chain_id, denom, { kind: "loading" }, val))
    return val
  })

  const restUrl = chain.rpcs.find(rpc => rpc.type === "rest")?.url
  if (!restUrl) {
    balances.update(val => {
      denoms.forEach(denom =>
        updateBalanceObject(
          chain.chain_id,
          denom,
          {
            kind: "error",
            error: `No REST RPC available for chain ${chain.chain_id}`,
            timestamp: Date.now()
          },
          val
        )
      )
      return val
    })
    return
  }

  const response = await fetchJson(`${restUrl}/cosmos/bank/v1beta1/balances/${addr}`).andThen(
    json => {
      const result = v.safeParse(cosmosBalancesResponseSchema, json)
      return result.success
        ? ok(result.output)
        : err(new Error("cosmos bank balances schema validation failed"))
    }
  )

  if (response.isErr()) {
    balances.update(val => {
      denoms.forEach(denom => {
        updateBalanceObject(
          chain.chain_id,
          denom,
          {
            kind: "error",
            error: response.error.message,
            timestamp: Date.now()
          },
          val
        )
      })
      return val
    })
    return
  }

  const rpcUrl = chain.rpcs.find(rpc => rpc.type === "rpc")?.url
  if (!rpcUrl) return toast.error("no rpc url found")
  balances.update(val => {
    bankDenoms.forEach(bankDenom => {
      const bankBalance = response.value.balances.find(b => toHex(b.denom) === bankDenom)
      updateBalanceObject(
        chain.chain_id,
        bankDenom,
        {
          kind: "balance",
          amount: bankBalance?.amount ? bankBalance.amount : "0",
          timestamp: Date.now()
        },
        val
      )
    })
    return val
  })

  let publicClient = await ResultAsync.fromPromise(CosmWasmClient.connect(rpcUrl), error => {
    return new Error(`failed to create public cosmwasm client with rpc ${rpcUrl}`, {
      cause: error
    })
  })

  if (publicClient.isErr()) return err(publicClient.error)

  const cw20Balances = await Promise.all(
    cw20Denoms.map(async denom => {
      const balance = await ResultAsync.fromPromise(
        publicClient.value.queryContractSmart(fromHex(denom as Address, "string"), {
          balance: { address: addr }
        }),
        error => {
          return new Error(`failed to query balance for contract ${denom}`, { cause: error })
        }
      ).andThen(balance => ok(balance.balance))

      balances.update(val => {
        updateBalanceObject(
          chain.chain_id,
          denom,
          balance.isErr()
            ? {
                kind: "error",
                error: `${balance.error.message}, cause: ${balance.error.cause}`,
                timestamp: Date.now()
              }
            : { kind: "balance", amount: balance.value, timestamp: Date.now() },
          val
        )
        return val
      })
    })
  )
}
