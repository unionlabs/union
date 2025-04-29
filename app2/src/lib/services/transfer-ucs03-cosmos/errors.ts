import { Data } from "effect"

export class SwitchChainError extends Data.TaggedError("SwitchChainError")<{
  cause: string
}> {}

export class OfflineSignerError extends Data.TaggedError("OfflineSignerError")<{
  cause: unknown
}> {}

export class CosmWasmError extends Data.TaggedError("CosmWasmError")<{
  cause: string
}> {}

export class CosmosWalletNotConnectedError extends Data.TaggedError(
  "CosmosWalletNotConnectedError"
)<{
  cause: string
}> {}

export class CosmosWalletNotOnWindowError extends Data.TaggedError("CosmosWalletNotOnWindowError")<{
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
