import type { Chain } from "@unionlabs/sdk/schema/chain"
import { Data } from "effect"
import type { Effect } from "effect"
import type { getCosmosChainInfo } from "../cosmos/chain-info"

export class CosmosSwitchChainError extends Data.TaggedError(
  "CosmosSwitchChainError",
)<{
  cause: unknown
  chainId: string
  phase: "enable" | "suggest"
  chainInfo: Effect.Effect.Success<ReturnType<typeof getCosmosChainInfo>>
}> {}

export class NoCosmosChainInfoError extends Data.TaggedError(
  "NoCosmosChainInfoError",
)<{
  chain: Chain
}> {}

export class OfflineSignerError extends Data.TaggedError("OfflineSignerError")<{
  chain_id: string
  cause: unknown
}> {}

export class CosmWasmError extends Data.TaggedError("CosmWasmError")<{
  cause: string
}> {}

export class CosmosWalletNotConnectedError extends Data.TaggedError(
  "CosmosWalletNotConnectedError",
)<{
  cause: string
}> {}

export class CosmosWalletNotOnWindowError extends Data.TaggedError(
  "CosmosWalletNotOnWindowError",
)<{
  kind: string
}> {}

export class GetChainInfoError extends Data.TaggedError("GetChainInfoError")<{
  cause: string
  chainId?: string
}> {}

export class GasPriceError extends Data.TaggedError("GasPriceError")<{
  cause: string
  chainId?: string
}> {}
