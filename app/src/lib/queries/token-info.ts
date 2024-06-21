// We should not use this (yet) as we are hardcoding asset info in v0.assets
// because the decimals reported by ERC20 contracts are incorrect

// import { readContracts } from "@wagmi/core"
// import { erc20Abi, type Address } from "viem"
// import { config as evmConfig } from "$lib/wallet/evm/config.ts"

// const functionNames = ["symbol", "decimals"] as const

// export async function getEvmTokensInfo(contractAddresses: Array<string>) {
//   const promises = await Promise.allSettled(
//     contractAddresses.map(address =>
//       readContracts(evmConfig, {
//         allowFailure: false,
//         contracts: functionNames.map(functionName => ({
//           address: address as Address,
//           functionName,
//           abi: erc20Abi
//         }))
//       })
//     )
//   )
//   const promisesSettled = promises
//     .filter(promise => promise.status === "fulfilled")
//     .map((result, index) => {
//       // TODO: fix this type
//       const [symbol, decimals] = (result as any).value as [string, number]
//       return { address: contractAddresses[index], symbol: symbol, decimals: decimals }
//     })
//   return promisesSettled.filter(Boolean)
// }
