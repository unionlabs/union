import { Data } from "effect";

export class SwitchChainError extends Data.TaggedError("SwitchChainError")<{
  cause: string
}> {}