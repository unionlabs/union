import { raise } from "#utilities/index.js"
import type { Ucs1Configuration } from "./query/off-chain.ts"

export const createPfmMemo = ({
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
  })

export function findPfmPath({
  sourceChainId,
  destinationChainId,
  data
}: {
  sourceChainId: string
  destinationChainId: string
  data: Array<Ucs1Configuration>
}): [source: Ucs1Configuration, destination: Ucs1Configuration] {
  raise("Not implemented")
}
