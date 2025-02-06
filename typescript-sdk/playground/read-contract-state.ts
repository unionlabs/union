import { queryContractState } from "#query/on-chain"
import { consola } from "scripts/logger"

let contractState = await queryContractState({
  restUrl: "https://rest.testnet-9.union.build",
  contractAddress: "union16sjqs0duegrhqn6g20m2xr4tp6xtv0ymfu4cuauah44ly8qfkzmqt8yywx"
})
consola.log("contract state", contractState)

const CW20_DENOM = "union10y75w84ecnqwx4v8xdn00tppgxckxeu80n3nhy8qdt66slhrtevs789d4k"

let contractStateCw20 = await queryContractState({
  restUrl: "https://rest.testnet-9.union.build",
  contractAddress: CW20_DENOM
})
consola.log("contract state cw20", contractStateCw20)
