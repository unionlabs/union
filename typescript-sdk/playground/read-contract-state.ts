import { queryContractState } from "#query/on-chain"
import { consola } from "scripts/logger"

let contractSTate = await queryContractState({
  restUrl: "https://rest.testnet-9.union.build",
  contractAddress: "union16sjqs0duegrhqn6g20m2xr4tp6xtv0ymfu4cuauah44ly8qfkzmqt8yywx"
})
consola.log("contract state", contractSTate)
