import { Result } from "neverthrow"

export const createPfmMemo = Result.fromThrowable(
  ({
    port,
    channel,
    receiver
  }: {
    port: string
    channel: string
    receiver: string
  }) =>
    JSON.stringify({
      forward: {
        port,
        channel,
        receiver: receiver.startsWith("0x") ? receiver.slice(2) : receiver
      }
    }),
  error => new Error("Failed to create PFM memo", { cause: error })
)
