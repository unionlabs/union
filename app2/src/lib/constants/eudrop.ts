// UDrop contract configuration

export const EUDROP_CONTRACT_ADDRESS = "0xB055d055bfc0b1956a5D09a8358F8B5b924348E7" as const

export const EUDROP_ABI = [
  {
    name: "active",
    type: "function",
    stateMutability: "view",
    inputs: [],
    outputs: [{ name: "", type: "bool" }],
  },
  {
    name: "claim",
    type: "function",
    stateMutability: "nonpayable",
    inputs: [
      { name: "beneficiary", type: "address" },
      { name: "amount", type: "uint256" },
      { name: "proof", type: "bytes32[]" },
    ],
    outputs: [],
  },
  {
    name: "claimed",
    type: "function",
    stateMutability: "view",
    inputs: [
      { name: "", type: "address" },
    ],
    outputs: [
      { name: "", type: "bool" },
    ],
  },
] as const
