import { encodeAbiParameters, type Hex } from "viem"
import { encodeAbi, type Instruction } from "./instruction.js"
import { zkgmPacketAbi } from "../evm/abi/index.js"

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
      operand: encodeAbi(packet.instruction)
    }
  ])
