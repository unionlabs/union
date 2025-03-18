import { ucs03abi } from "./ucs03.js"
import { Option } from "effect"

export * from "./ucs03.js"

const packetComponentAbis = Option.fromNullable(
  ucs03abi.find(a => "name" in a && a.name === "ensureExported")
).pipe(Option.map(a => a.inputs))

const getStructComponents = (structName: string) =>
  packetComponentAbis.pipe(
    Option.flatMap(a =>
      Option.fromNullable(a.find(as => as.internalType === `struct ${structName}`))
    ),
    Option.map(a => a.components),
    Option.getOrThrow
  )

export const fungibleAssetOrderAbi = getStructComponents("FungibleAssetOrder")
export const instructionAbi = getStructComponents("Instruction")
export const zkgmPacketAbi = getStructComponents("ZkgmPacket")
export const forwardAbi = getStructComponents("Forward")
export const multiplexAbi = getStructComponents("Multiplex")
export const batchAbi = getStructComponents("Batch")
export const ackAbi = getStructComponents("Ack")
export const batchAckAbi = getStructComponents("BatchAck")
export const fungibleAssetOrderAckAbi = getStructComponents("FungibleAssetOrderAck")
