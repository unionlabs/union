import { Effect } from "effect"
import { queryContract } from "./contract.js"
import { AptosPublicClient } from "./client.js"

export type Hex = `0x${string}`

export const readFaBalance = (contractAddress: string, address: string) =>
  Effect.gen(function* () {
    const client = (yield* AptosPublicClient).client

    const contract_address = "0x1"
    const function_name = "primary_fungible_store::balance"
    const type_arguments = ["0x1::fungible_asset::Metadata"]
    const function_arguments = [address, contractAddress]
    
    const result = yield* queryContract(client, contract_address, function_name, type_arguments, function_arguments);

    // Extract the address from the result tuple
    return result[0]
  })
