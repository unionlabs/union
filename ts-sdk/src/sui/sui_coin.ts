import { Effect, Data } from "effect"
import { SuiPublicClient } from "./client.js"
import { extractErrorDetails } from "../utils/extract-error-details.js"

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

export class ReadCoinError extends Data.TaggedError("ReadContractError")<{
  cause: unknown
}> {}

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
      catch: err => new ReadCoinError({
        cause: extractErrorDetails(err as ReadCoinError),
      }),
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
        catch: err => new ReadCoinError({
          cause: extractErrorDetails(err as ReadCoinError),
        }),
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
        catch: err => new ReadCoinError({
          cause: extractErrorDetails(err as ReadCoinError),
        }),
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
        catch: err => new ReadCoinError({
          cause: extractErrorDetails(err as ReadCoinError),
        }),
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
          
  export const getCoinName = (address: string) =>
    Effect.gen(function*() {
      const client = (yield* SuiPublicClient).client

      const name = yield * Effect.tryPromise({
        try: async () => {
          const result = await client.getCoinMetadata({coinType: address});
          return result?.name
        },
        catch: err => new ReadCoinError({
          cause: extractErrorDetails(err as ReadCoinError),
        }),
      })
      return name;
    })

  export const getCoinDecimals = (address: string) =>
    Effect.gen(function*() {
      const client = (yield* SuiPublicClient).client
  
      const decimals = yield * Effect.tryPromise({
        try: async () => {
          const result = await client.getCoinMetadata({coinType: address});
          return result?.decimals
        },
        catch: err => new ReadCoinError({
          cause: extractErrorDetails(err as ReadCoinError),
        }),
      })
      return decimals;
    })

    export const readCoinSymbol = (address: string) =>
      Effect.gen(function*() {
        const client = (yield* SuiPublicClient).client

        const symbol = yield * Effect.tryPromise({
          try: async () => {
            const result = await client.getCoinMetadata({coinType: address});
            return result?.symbol
          },
          catch: err => new ReadCoinError({
            cause: extractErrorDetails(err as ReadCoinError),
          }),
        })
        return symbol;
      })

  export const readCoinMetadata = (address: string) =>
    Effect.gen(function*() {
      const client = (yield* SuiPublicClient).client

      const metadata = yield * Effect.tryPromise({
        try: async () => {
          const result = await client.getCoinMetadata({coinType: address});
          return result
        },
        catch: err => new ReadCoinError({
          cause: extractErrorDetails(err as ReadCoinError),
        }),
      })
      return metadata;
    })