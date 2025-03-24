import { Effect } from "effect"
import { queryContract } from "./contract.js"
import { AptosPublicClient } from "./client.js"

export type Hex = `0x${string}`

/**
 * Interface for FA token metadata
 */
export interface FaTokenInfo {
  decimals: number
  icon_uri: string
  name: string
  project_uri: string
  symbol: string
}



export const readFaBalance = (contractAddress: string, address: string) =>
  Effect.gen(function* () {
    const client = (yield* AptosPublicClient).client

    const balance_module_name = "fungible_asset"

    const contract_address = "0x1"
    const function_name = "balance"
    const type_arguments = ["0x1::fungible_asset::Metadata"]
    const function_arguments = [address, contractAddress]

    const result = yield* queryContract(
      client,
      contract_address,
      balance_module_name,
      function_name,
      type_arguments,
      function_arguments
    )

    // Extract the address from the result tuple
    return result[0]
  })

export const readFaName = (contractAddress: string) =>
  Effect.gen(function* () {
    const client = (yield* AptosPublicClient).client

    const module_name = "fungible_asset"
    const contract_address = "0x1"
    const function_name = "name"
    const type_arguments = ["0x1::fungible_asset::Metadata"]
    const function_arguments = [contractAddress]

    const result = yield* queryContract(
      client,
      contract_address,
      module_name,
      function_name,
      type_arguments,
      function_arguments
    )

    // Extract the address from the result tuple
    return result[0]
  })

export const readFaDecimals = (contractAddress: string) =>
  Effect.gen(function* () {
    const client = (yield* AptosPublicClient).client

    const contract_address = "0x1"
    const module_name = "fungible_asset"
    const function_name = "decimals"
    const type_arguments = ["0x1::fungible_asset::Metadata"]
    const function_arguments = [contractAddress]

    const result = yield* queryContract(
      client,
      contract_address,
      module_name,
      function_name,
      type_arguments,
      function_arguments
    )

    // Extract the address from the result tuple
    return result[0]
  })

export const readFaSymbol = (contractAddress: string) =>
  Effect.gen(function* () {
    const client = (yield* AptosPublicClient).client

    const contract_address = "0x1"
    const module_name = "fungible_asset"
    const function_name = "symbol"
    const type_arguments = ["0x1::fungible_asset::Metadata"]
    const function_arguments = [contractAddress]

    const result = yield* queryContract(
      client,
      contract_address,
      module_name,
      function_name,
      type_arguments,
      function_arguments
    )

    // Extract the address from the result tuple
    return result[0]
  })

  export const readFaTokenInfo = (contractAddress: string) =>
    Effect.gen(function* () {
      const client = (yield* AptosPublicClient).client
  
      const contract_address = "0x1"
      const module_name = "fungible_asset"
      const function_name = "metadata"
      const type_arguments = ["0x1::fungible_asset::Metadata"]
      const function_arguments = [contractAddress]
  
      const result = yield* queryContract(
        client,
        contract_address,
        module_name,
        function_name,
        type_arguments,
        function_arguments
      )
  
      const token_info = result[0] as FaTokenInfo

      return token_info
    })
