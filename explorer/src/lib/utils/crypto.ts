// Utility functions for validator address conversion

/**
 * Convert base64 string to Uint8Array
 */
export function fromBase64(base64: string): Uint8Array {
  const binary = atob(base64)
  const bytes = new Uint8Array(binary.length)
  for (let i = 0; i < binary.length; i++) {
    bytes[i] = binary.charCodeAt(i)
  }
  return bytes
}

/**
 * Convert Uint8Array to base64 string
 */
export function toBase64(bytes: Uint8Array): string {
  let binary = ""
  for (let i = 0; i < bytes.length; i++) {
    binary += String.fromCharCode(bytes[i])
  }
  return btoa(binary)
}

/**
 * Convert Uint8Array to hex string
 */
export function toHex(bytes: Uint8Array): string {
  return Array.from(bytes)
    .map((b) => b.toString(16).padStart(2, "0"))
    .join("")
}

/**
 * Convert hex string to Uint8Array
 */
export function fromHex(hex: string): Uint8Array {
  const cleanHex = hex.startsWith("0x") ? hex.slice(2) : hex
  const bytes = new Uint8Array(cleanHex.length / 2)
  for (let i = 0; i < bytes.length; i++) {
    bytes[i] = parseInt(cleanHex.slice(i * 2, i * 2 + 2), 16)
  }
  return bytes
}

/**
 * SHA256 hash using Web Crypto API
 */
export async function sha256(data: Uint8Array): Promise<Uint8Array> {
  const hashBuffer = await crypto.subtle.digest("SHA-256", data)
  return new Uint8Array(hashBuffer)
}

/**
 * Convert consensus pubkey to hex address (for matching block signatures)
 * Supports ed25519 and bn254 pubkeys
 */
export async function consensusPubkeyToHexAddress(
  consensusPubkey?: { "@type": string; "key": string },
): Promise<string> {
  if (!consensusPubkey?.key) {
    return ""
  }

  const pubkeyBytes = fromBase64(consensusPubkey.key)
  const hash = await sha256(pubkeyBytes)

  // Take first 20 bytes (40 hex chars) and uppercase
  return toHex(hash).slice(0, 40).toUpperCase()
}

/**
 * Check if a validator signed a block by comparing addresses
 * Block signatures use base64-encoded addresses
 */
export function didValidatorSign(
  validatorHexAddress: string,
  signatures: Array<{ validator_address: string; signature: string | null }>,
): boolean {
  if (!validatorHexAddress || !signatures) {
    return false
  }

  return signatures.some((sig) => {
    if (!sig.validator_address || !sig.signature) {
      return false
    }
    // Convert base64 validator_address to hex and compare
    try {
      const sigHex = toHex(fromBase64(sig.validator_address)).toUpperCase()
      return sigHex === validatorHexAddress
    } catch {
      return false
    }
  })
}
