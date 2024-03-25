import type { PageLoad } from "./$types"

export const load = (({ params }) => {
  return {
    transfers: [
      {
        hash: "0x1a2b3c4d5e6f7g8h",
        type: "transfer",
        height: 123456,
        sender: "union1q2w3e4r5t6y7u8i9o0p1a2s3d4f5g6h7j8k9l0m",
        receiver: "union1z2x3c4v5b6n7m8k9j8h7g6f5d4s3a2p0o9i8u7y",
        amount: "1000",
        timestamp: "2023-04-01T12:00:00Z",
        gasUsed: 21000
      },
      {
        hash: "0x9a8b7c6d5e4f3g2h",
        type: "stake",
        height: 123457,
        sender: "union1a2s3d4f5g6h7j8k9l0m1q2w3e4r5t6y7u8i9o0p",
        receiver: "union1y2u3i4o5p6a7s8d9f0g1h2j3k4l5z6x7c8v9b0n",
        amount: "500",
        timestamp: "2024-04-02T13:00:00Z",
        gasUsed: 22000
      },
      {
        hash: "0x5f4e3d2c1b0a9z8y",
        type: "unstake",
        height: 123458,
        sender: "union1z2x3c4v5b6n7m8k9j8h7g6f5d4s3a2p0o9i8u7y",
        receiver: "union1q2w3e4r5t6y7u8i9o0p1a2s3d4f5g6h7j8k9l0m",
        amount: "200",
        timestamp: "2024-04-05T18:39:30Z",
        gasUsed: 23000
      },
      {
        hash: "0x1a2b3c4d5e6f7g8h",
        type: "transfer",
        height: 123456,
        sender: "union1q2w3e4r5t6y7u8i9o0p1a2s3d4f5g6h7j8k9l0m",
        receiver: "union1z2x3c4v5b6n7m8k9j8h7g6f5d4s3a2p0o9i8u7y",
        amount: "1000",
        timestamp: "2023-04-01T12:00:00Z",
        gasUsed: 21000
      },
      {
        hash: "0x9a8b7c6d5e4f3g2h",
        type: "stake",
        height: 123457,
        sender: "union1a2s3d4f5g6h7j8k9l0m1q2w3e4r5t6y7u8i9o0p",
        receiver: "union1y2u3i4o5p6a7s8d9f0g1h2j3k4l5z6x7c8v9b0n",
        amount: "500",
        timestamp: "2024-04-02T13:00:00Z",
        gasUsed: 22000
      }
    ]
  }
}) satisfies PageLoad
