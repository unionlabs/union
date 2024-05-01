#!/usr/bin/env bun
import { fetcher } from "#/utilities"
import { getUnoFromFaucet } from "#/faucet"

interface Coin {
  readonly denom: string
  readonly amount: string
}

const REST_API_URL = "https://api.testnet.bonlulu.uno"

const [flag, address] = process.argv.slice(2)

main().catch(error => {
  console.error(error)
  process.exit(1)
})

async function main() {
  if (flag !== "--address" || !address) {
    throw new Error("Usage: bun ./scripts/faucet.ts --address <address>")
  }
  const faucetResponse = await getUnoFromFaucet({ address })
  if (!Object.hasOwn(faucetResponse, "union")) {
    console.error("Failed to get uno from faucet", JSON.stringify(faucetResponse, undefined, 2))
  }
  const {
    balances: [balance]
  } = await fetcher<{ balances: Array<Coin> }>(
    `${REST_API_URL}/cosmos/bank/v1beta1/balances/${address}`
  )
  if (!balance?.amount) {
    console.error("Failed to get uno balance", JSON.stringify(balance, undefined, 2))
  }
  console.info(
    `Deposited a nice sum of ${balance?.denom} into ${address}.\nCurrent balance: ${balance?.amount}`
  )
}
