import { Data } from "effect";

export class SwitchChainError extends Data.TaggedError("SwitchChainError")<{
  cause: string
}> {}

export class OfflineSignerError extends Data.TaggedError("OfflineSignerError")<{
  cause: string
}> {}

export class CosmWasmError extends Data.TaggedError("CosmWasmError")<{
  cause: string
}> {}

export class GetChainInfoError extends Data.TaggedError("GetChainInfoError")<{
  cause: string
  chainId?: string
}> {}

export class GasPriceError extends Data.TaggedError("GasPriceError")<{
  cause: string
  chainId?: string
}> {}