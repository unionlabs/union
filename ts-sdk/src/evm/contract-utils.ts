import { Effect } from "effect"
import type { 
  Abi, 
  PublicClient, 
  ReadContractErrorType, 
  ReadContractParameters, 
  ReadContractReturnType,
  ContractFunctionName,
  ContractFunctionArgs
} from "viem"
import { ReadContractError } from "./client.js"
import { extractErrorDetails } from "../utils/extract-error-details.js"

/**
 * A type-safe wrapper around viem's readContract that handles error cases
 * and returns an Effect with proper type inference
 * 
 * @param client - The viem PublicClient to use for the contract call
 * @param params - The parameters for the contract call
 * @returns An Effect that resolves to the properly typed return value
 */
export const readContract = <
  TAbi extends Abi,
  TFunctionName extends ContractFunctionName<TAbi, 'pure' | 'view'> = ContractFunctionName<TAbi, 'pure' | 'view'>,
  TArgs extends ContractFunctionArgs<TAbi, 'pure' | 'view', TFunctionName> = ContractFunctionArgs<TAbi, 'pure' | 'view', TFunctionName>
>(
  client: PublicClient,
  params: ReadContractParameters<TAbi, TFunctionName, TArgs>
) => Effect.tryPromise({
  try: () => client.readContract(params),
  catch: (error) =>
    new ReadContractError({ cause: extractErrorDetails(error as ReadContractErrorType) })
})
