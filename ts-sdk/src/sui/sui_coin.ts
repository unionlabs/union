import { Effect } from "effect"
import { SuiPublicClient } from "./client.js"
import { readContract } from "./contract.js"

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

export const readCoinBalances = (contractAddress: string, address: string) =>
  Effect.gen(function*() {
    const client = (yield* SuiPublicClient).client
    let params = {
      owner: address,
      coinType: contractAddress
    };
   
    const coins = yield * Effect.tryPromise({
      try: async () => {
        const result = await client.getCoins(params);
        return result.data
      },
      catch: error => new Error(`Failed to read FA balance: ${error}`)
    })
    return coins
  })

  export const readTotalCoinBalance = (contractAddress: string, address: string) =>
    Effect.gen(function*() {
      const client = (yield* SuiPublicClient).client
      let params = {
        owner: address,
        coinType: contractAddress
      };
     
  
      const coins = yield * Effect.tryPromise({
        try: async () => {
          const result = await client.getCoins(params);
          return result.data
        },
        catch: error => new Error(`Failed to read FA balance: ${error}`)
      })
      // Calculate total balance
      const totalBalance = coins.reduce((acc, coin) => acc + BigInt(coin.balance), BigInt(0));
  
      return totalBalance
    })

  
  export const getAllCoins = (address: string) =>
    Effect.gen(function*() {
      const client = (yield* SuiPublicClient).client
      let params = {
        owner: address,
      };
  
      const coins = yield * Effect.tryPromise({
        try: async () => {
          const result = await client.getAllCoins(params);
          return result.data
        },
        catch: error => new Error(`Failed to read FA balance: ${error}`)
      })
      return coins;
    })
    
    
  export const getAllCoinsUnique = (address: string) =>
    Effect.gen(function*() {
      const client = (yield* SuiPublicClient).client;
  
      const params = {
        owner: address,
      };
  
      const coins = yield* Effect.tryPromise({
        try: async () => {
          const result = await client.getAllCoins(params);
          return result.data;
        },
        catch: error => new Error(`Failed to read FA balance: ${error}`),
      });
  
      // Group by coinType and sum balances
      const coinMap: Record<string, bigint> = {};
  
      for (const coin of coins) {
        const coinType = coin.coinType;
        const balance = BigInt(coin.balance);
  
        if (!coinMap[coinType]) {
          coinMap[coinType] = balance;
        } else {
          coinMap[coinType] += balance;
        }
      }
  
      // Convert to array of objects
      const result = Object.entries(coinMap).map(([coinType, totalBalance]) => ({
        coinType,
        balance: totalBalance.toString(), // or keep as BigInt if preferred
      }));
  
      return result;
    });
          
        
// export const readFaName = (contractAddress: string) =>
//   Effect.gen(function*() {
//     const client = (yield* AptosPublicClient).client

//     const module_name = "fungible_asset"
//     const contract_address = "0x1"
//     const function_name = "name"
//     const type_arguments = ["0x1::fungible_asset::Metadata"]
//     const function_arguments = [contractAddress]

//     const result = yield* readContract(
//       client,
//       contract_address,
//       module_name,
//       function_name,
//       type_arguments,
//       function_arguments,
//     )

//     // Extract the address from the result tuple
//     return result[0]
//   })

// export const readFaDecimals = (contractAddress: string) =>
//   Effect.gen(function*() {
//     const client = (yield* AptosPublicClient).client

//     const contract_address = "0x1"
//     const module_name = "fungible_asset"
//     const function_name = "decimals"
//     const type_arguments = ["0x1::fungible_asset::Metadata"]
//     const function_arguments = [contractAddress]

//     const result = yield* readContract(
//       client,
//       contract_address,
//       module_name,
//       function_name,
//       type_arguments,
//       function_arguments,
//     )

//     // Extract the address from the result tuple
//     return result[0]
//   })

// export const readFaSymbol = (contractAddress: string) =>
//   Effect.gen(function*() {
//     const client = (yield* AptosPublicClient).client

//     const contract_address = "0x1"
//     const module_name = "fungible_asset"
//     const function_name = "symbol"
//     const type_arguments = ["0x1::fungible_asset::Metadata"]
//     const function_arguments = [contractAddress]

//     const result = yield* readContract(
//       client,
//       contract_address,
//       module_name,
//       function_name,
//       type_arguments,
//       function_arguments,
//     )

//     // Extract the address from the result tuple
//     return result[0]
//   })

// export const readFaTokenInfo = (contractAddress: string) =>
//   Effect.gen(function*() {
//     const client = (yield* AptosPublicClient).client

//     const contract_address = "0x1"
//     const module_name = "fungible_asset"
//     const function_name = "metadata"
//     const type_arguments = ["0x1::fungible_asset::Metadata"]
//     const function_arguments = [contractAddress]

//     const result = yield* readContract(
//       client,
//       contract_address,
//       module_name,
//       function_name,
//       type_arguments,
//       function_arguments,
//     )

//     const token_info = result[0] as FaTokenInfo

//     return token_info
//   })
