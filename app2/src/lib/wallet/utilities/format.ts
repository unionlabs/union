export const truncateUnionAddress = (address: string, length = 6) =>
  length > 0 ? `${address.slice(0, length)}...${address.slice(-length)}` : address

export const truncateEvmAddress = (address: string, length = 6) =>
  length > 0 ? `${address.slice(0, length)}...${address.slice(-length)}` : address

export const convertByteArrayToHex = (byteArray: Uint8Array): string =>
  byteArray.reduce((hex, byte) => hex + byte.toString(16).padStart(2, "0"), "").toUpperCase()
