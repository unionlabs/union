// UDrop contract configuration

export const UDROP_CONTRACT_ADDRESS = "0xC0DEB405dd405Ee54F2Fc24E8E3DB5D417001631" as const

export const UDROP_ABI = [
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
