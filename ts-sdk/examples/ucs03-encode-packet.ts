import { toHex } from "viem"
import { Instruction } from "@unionlabs/sdk/ucs03"
import { encodeZkgmPacketAbi, type ZkgmPacket } from "../src/ucs03/zkgm-packet.js"

const examplePacket: ZkgmPacket = {
  salt: "0x1234567890123456789012345678901234567890123456789012345678901234",
  path: 123456789n,
  instruction: Instruction.FungibleAssetOrder.make({
    operand: [
      toHex("stars1qcvavxpxw3t8d9j7mwaeq9wgytkf5vwputv5x4"),
      toHex("union1d95n4r6dnrfrps59szhl8mk7yqewsuzyw0zh5q"),
      toHex("stars1qrde534d4jwk44dn7w7gu9e2rayutr7kqx8lfjhsk3rd7z9rzxhq2gh3lr"),
      1000000n,
      "WETH",
      "WETH",
      0,
      0n,
      "0x756e696f6e3170707865737461307064643038716537366779366b7438667563717978727a773076786a79366439767430676368357879707371656138377278",
      1000000n
    ]
  })
}

console.log("ZkgmPacket:", examplePacket)
console.log("Encoded:", encodeZkgmPacketAbi(examplePacket))
