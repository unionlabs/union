import { encodeAbiParameters, type Hex } from "viem"
import { zkgmPacketAbi } from "../evm/abi/index.js"
import { encodeAbi, type Instruction } from "./instruction.js"

export type ZkgmPacket = {
  salt: Hex
  path: bigint
  instruction: Instruction
}

export const encodeZkgmPacketAbi = (packet: ZkgmPacket) =>
  encodeAbiParameters(zkgmPacketAbi, [
    packet.salt,
    packet.path,
    {
      opcode: packet.instruction.opcode,
      version: packet.instruction.version,
      operand: encodeAbi(packet.instruction),
    },
  ])
