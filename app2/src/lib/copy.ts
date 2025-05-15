import { interpolate } from "$lib/utils/interpolate.js"
import { NotACosmosChainError } from "@unionlabs/sdk/schema"

export const OfflineSignerCopy = interpolate(
  "Something went wrong signing the {{step}}",
)
export const NoCosmosChainInfoCopy = interpolate(
  "Could not find chain {{sourceChain}}. Please check wallet configuration.",
)
export const CosmosWalletNotOnWindowCopy = interpolate(
  "Cosmos wallet not available on `window.{{wallet}}`. Please check wallet connection.",
)
export const SwitchChainCopy = interpolate(
  "Could not switch {{wallet}} wallet from chain {{from}} to chain {{to}}. Please switch manually.",
)
export const ContractErrorCopy = interpolate(
  "Error executing contract",
)
