import { ucs03abi } from "./ucs03.js"
import { Option } from "effect"

export * from "./ucs03.js"

const packetComponentAbis = Option.fromNullable(
  ucs03abi.find(a => "name" in a && a.name === "ensureExported")
).pipe(Option.map(a => a.inputs))

export const fungibleAssetOrderAbi = packetComponentAbis.pipe(
  Option.flatMap(a =>
    Option.fromNullable(a.find(as => as.internalType === "struct FungibleAssetOrder"))
  ),
  Option.map(a => a.components),
  Option.getOrThrow
)

export const instructionAbi = packetComponentAbis.pipe(
  Option.flatMap(a => Option.fromNullable(a.find(as => as.internalType === "struct Instruction"))),
  Option.map(a => a.components),
  Option.getOrThrow
)

export const zkgmPacketAbi = packetComponentAbis.pipe(
  Option.flatMap(a => Option.fromNullable(a.find(as => as.internalType === "struct ZkgmPacket"))),
  Option.map(a => a.components),
  Option.getOrThrow
)
