import { bech32 } from "@scure/base"
import {Schema} from "effect";
import {AddressAptosDisplay, AddressCosmosDisplay, AddressEvmDisplay} from "$lib/schema/address";

/**
 * Convert a bech32 display address to canonical bytes
 */
export function cosmosDisplayToCanonical(displayAddress: string): Uint8Array {
  try {
    const decoded = bech32.decode(displayAddress as `${string}1${string}`)
    const canonicalAddress = bech32.fromWords(decoded.words)
    return new Uint8Array(canonicalAddress)
  } catch (error: any) {
    throw new Error(`Invalid Cosmos bech32 address: ${error.message}`)
  }
}

/**
 * Convert an EVM display address (hex) to canonical bytes
 */
export function evmDisplayToCanonical(displayAddress: string): Uint8Array {
  // Validate EVM address format (0x + 40 hex characters)
  if (!/^0x[0-9a-fA-F]{40}$/.test(displayAddress)) {
    throw new Error("EVM address must be 0x followed by 40 hex characters")
  }

  // Remove 0x prefix and convert to bytes
  const hexWithoutPrefix = displayAddress.slice(2)
  const bytes = new Uint8Array(20)

  for (let i = 0; i < 40; i += 2) {
    bytes[i / 2] = Number.parseInt(hexWithoutPrefix.substring(i, i + 2), 16)
  }

  return bytes
}

/**
 * Convert an Aptos display address (hex) to canonical bytes
 */
export function aptosDisplayToCanonical(displayAddress: string): Uint8Array {
  // Validate Aptos address format (0x + 64 hex characters)
  if (!/^0x[0-9a-fA-F]{64}$/.test(displayAddress)) {
    throw new Error("Aptos address must be 0x followed by 64 hex characters")
  }

  // Remove 0x prefix and convert to bytes
  const hexWithoutPrefix = displayAddress.slice(2)
  const bytes = new Uint8Array(32)

  for (let i = 0; i < 64; i += 2) {
    bytes[i / 2] = Number.parseInt(hexWithoutPrefix.substring(i, i + 2), 16)
  }

  return bytes
}

/**
 * Converts a Uint8Array to a hex string
 */
export function bytesToHex(bytes: Uint8Array): string {
  let hexString = ""
  for (const byte of bytes) {
    hexString += byte.toString(16).padStart(2, "0")
  }
  return `0x${hexString}`
}

export const isValidCanonicalForChain = (displayAddress: string, destinationRpcType: string): boolean => {
  if (!displayAddress || displayAddress.length === 0) {
    return false;
  }

  // Function to validate display format using schema
  const isValidDisplay = (schema: Schema.Schema<any, any>): boolean => {
    try {
      Schema.decodeSync(schema)(displayAddress, { errors: "all" });
      return true;
    } catch (e) {
      return false;
    }
  };

  // First validate the display format using appropriate schema
  let isValidDisplayFormat = false;
  switch (destinationRpcType) {
    case "evm":
      isValidDisplayFormat = isValidDisplay(AddressEvmDisplay);
      break;
    case "cosmos":
      isValidDisplayFormat = isValidDisplay(AddressCosmosDisplay);
      break;
    case "aptos":
      isValidDisplayFormat = isValidDisplay(AddressAptosDisplay);
      break;
    default:
      return false;
  }

  // If display format is invalid, canonical format cannot be valid
  if (!isValidDisplayFormat) {
    return false;
  }

  // Then convert from display to canonical and validate
  try {
    let canonicalBytes: Uint8Array;

    switch (destinationRpcType) {
      case "evm":
        // Convert EVM display address (checksum hex) to canonical bytes (20 bytes)
        canonicalBytes = evmDisplayToCanonical(displayAddress);
        return canonicalBytes.length === 20;

      case "cosmos":
        // Convert Cosmos display address (bech32) to canonical bytes
        canonicalBytes = cosmosDisplayToCanonical(displayAddress);
        return canonicalBytes.length === 20 || canonicalBytes.length === 32;

      case "aptos":
        // Convert Aptos display address (hex) to canonical bytes
        canonicalBytes = aptosDisplayToCanonical(displayAddress);
        return canonicalBytes.length === 32;

      default:
        return false;
    }
  } catch (error) {
    return false;
  }
};