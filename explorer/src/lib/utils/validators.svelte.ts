import { addressFormat } from "$lib/stores/address-format.svelte"
import type { Validator } from "$lib/types/cosmos"
import {
  consensusPubkeyToHexAddress,
  fromBase64,
  fromHex,
  toBase64,
  toHex,
} from "$lib/utils/crypto"

// Avatar cache - reactive for Svelte 5
let avatarCacheVersion = $state(0)
const avatarCache = new Map<string, string>()
const pendingAvatarRequests = new Map<string, Promise<string | null>>()

// Load avatars from localStorage on init
if (typeof window !== "undefined") {
  try {
    const stored = localStorage.getItem("validator-avatars")
    if (stored) {
      const parsed = JSON.parse(stored) as Record<string, string>
      for (const [key, value] of Object.entries(parsed)) {
        avatarCache.set(key, value)
      }
    }
  } catch {
    // Ignore localStorage errors
  }
}

function saveAvatarsToStorage() {
  if (typeof window === "undefined") {
    return
  }
  try {
    const obj: Record<string, string> = {}
    avatarCache.forEach((v, k) => {
      obj[k] = v
    })
    localStorage.setItem("validator-avatars", JSON.stringify(obj))
  } catch {
    // Ignore localStorage errors
  }
}

/**
 * Get the current avatar cache version (for reactivity)
 */
export function getAvatarCacheVersion(): number {
  return avatarCacheVersion
}

/**
 * Fetch Keybase avatar for a validator identity
 * Results are cached to avoid duplicate requests
 */
export async function fetchKeybaseAvatar(identity: string): Promise<string | null> {
  if (!identity) {
    return null
  }

  // Return cached result
  if (avatarCache.has(identity)) {
    return avatarCache.get(identity) || null
  }

  // Return pending request if one exists
  if (pendingAvatarRequests.has(identity)) {
    return pendingAvatarRequests.get(identity)!
  }

  // Create new request
  const request = (async () => {
    try {
      const res = await fetch(
        `https://keybase.io/_/api/1.0/user/lookup.json?key_suffix=${identity}&fields=pictures`,
      )
      const data = await res.json()
      const url = data.them?.[0]?.pictures?.primary?.url ?? null
      if (url) {
        avatarCache.set(identity, url)
        avatarCacheVersion++ // Trigger reactivity
        saveAvatarsToStorage()
      }
      return url
    } catch {
      return null
    } finally {
      pendingAvatarRequests.delete(identity)
    }
  })()

  pendingAvatarRequests.set(identity, request)
  return request
}

/**
 * Get cached avatar URL for an identity (reactive - tracks cache version)
 */
export function getCachedAvatar(identity?: string): string | null {
  // Reading avatarCacheVersion makes this reactive
  const _ = avatarCacheVersion
  if (!identity) {
    return null
  }
  return avatarCache.get(identity) || null
}

/**
 * Build a map from consensus hex address to validator
 */
export async function buildValidatorMap(validators: Validator[]): Promise<Map<string, Validator>> {
  const map = new Map<string, Validator>()

  for (const validator of validators) {
    if (validator.consensus_pubkey) {
      const hexAddr = await consensusPubkeyToHexAddress(validator.consensus_pubkey)
      if (hexAddr) {
        map.set(hexAddr, validator)
      }
    }
  }

  return map
}

/**
 * Prefetch avatars for a list of validators
 */
export async function prefetchValidatorAvatars(validators: Validator[]): Promise<void> {
  const identities = validators
    .map(v => v.description.identity)
    .filter((id): id is string => !!id)

  await Promise.allSettled(identities.map(fetchKeybaseAvatar))
}

/**
 * Convert base64 address to hex (for proposer/signature addresses)
 */
export function base64ToHex(base64Address: string): string {
  try {
    return toHex(fromBase64(base64Address)).toUpperCase()
  } catch {
    return base64Address
  }
}

/**
 * Get validator info from a base64 encoded address
 */
export function getValidatorFromBase64Address(
  base64Address: string,
  validatorMap: Map<string, Validator>,
): { hex: string; validator: Validator | undefined; avatar: string | null } {
  const hex = base64ToHex(base64Address)
  const validator = validatorMap.get(hex)
  const avatar = getCachedAvatar(validator?.description.identity)
  return { hex, validator, avatar }
}

/**
 * Format an address based on the global address format setting
 * Input can be either hex or base64 - will detect and convert as needed
 */
export function formatAddress(address: string): string {
  if (!address) {
    return address
  }

  const format = addressFormat.value
  const isHexInput = /^[0-9a-fA-F]+$/.test(address)

  try {
    if (format === "hex") {
      // Want hex output
      if (isHexInput) {
        return address.toUpperCase()
      }
      // Convert from base64 to hex
      return toHex(fromBase64(address)).toUpperCase()
    } else {
      // Want base64 output
      if (!isHexInput) {
        return address // Already base64
      }
      // Convert from hex to base64
      return toBase64(fromHex(address))
    }
  } catch {
    return address
  }
}

/**
 * Get the current address format label
 */
export function getAddressFormatLabel(): string {
  return addressFormat.value === "hex" ? "HEX" : "Base64"
}
