import { CosmWasmClient } from "@cosmjs/cosmwasm-stargate"
import { consola } from "scripts/logger"
import { ResultAsync } from "neverthrow"
import { toHex } from "viem"

let publicClient = await ResultAsync.fromPromise(
  CosmWasmClient.connect("https://rpc.bbn-test-5.babylon.chain.kitchen"),
  error => {
    return new Error("failed to create public cosmos client", { cause: error })
  }
)

if (publicClient.isErr()) {
  consola.error(publicClient.error)
  process.exit(1)
}

let client = publicClient.value

// TODO: don't haredcode this either.
let quoteToken = await ResultAsync.fromPromise(
  client.queryContractSmart("bbn1mvvl3jvyn8dh9whfzkdzgedk56cjge7stsfwdjrcsvczns5waqzs6023q4", {
    predict_wrapped_denom: { path: "0", channel: 6, token: toHex("ubbn") }
  }),
  error => {
    return new Error("failed to query predict wrapped denom", { cause: error })
  }
)

consola.info(quoteToken)

if (quoteToken.isOk()) {
  consola.info(toHex(quoteToken.value))
}
