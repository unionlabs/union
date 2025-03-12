import { Data } from "effect";

export class SwitchChainError extends Data.TaggedError("SwitchChainError")<{
  cause: string
}> {}

export class OfflineSignerError extends Data.TaggedError("OfflineSignerError")<{
  cause: string
}> {}

export class StargateClientError extends Data.TaggedError("StargateClientError")<{
  cause: string
}> {}

export class GetChainInfoError extends Data.TaggedError("GetChainInfoError")<{
  cause: string
  chainId?: string
  wallet?: string
}> {}

export class GasPriceError extends Data.TaggedError("GasPriceError")<{
  cause: string
  chainId?: string
}> {}