import { consola } from "scripts/logger"
import { getCosmosAccountTransactions } from "#query/on-chain.ts"

const [, , address] = process.argv

if (!address?.startsWith("union")) {
  consola.info("only union atm")
  process.exit(1)
}

const txs = await getCosmosAccountTransactions({
  address,
  rpcUrl: "https://rpc.testnet-8.union.build"
})

console.log(JSON.stringify(txs, undefined, 2))
