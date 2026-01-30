<script lang="ts" module>
import { untrack } from "svelte"
import type { Snippet } from "svelte"
import type { CacheKey, CacheValue } from "$lib/cache/schema"

type CacheEntry<T> = {
  value: T
  timestamp: number
}

type CacheData = {
  [K in CacheKey]?: CacheEntry<CacheValue<K>>
}

const STORAGE_PREFIX = "cache:"
const caches = $state<CacheData>({})

function getStorageKey(cacheKey: string): string {
  return `${STORAGE_PREFIX}${cacheKey}`
}

function loadFromStorage<T>(cacheKey: string): CacheEntry<T> | null {
  if (typeof localStorage === "undefined") return null

  try {
    const stored = localStorage.getItem(getStorageKey(cacheKey))
    if (!stored) return null
    return JSON.parse(stored)
  } catch {
    return null
  }
}

function saveToStorage<T>(cacheKey: string, entry: CacheEntry<T>) {
  if (typeof localStorage === "undefined") return

  try {
    localStorage.setItem(getStorageKey(cacheKey), JSON.stringify(entry))
  } catch {
    // storage error
  }
}

function removeFromStorage(cacheKey: string) {
  if (typeof localStorage === "undefined") return
  localStorage.removeItem(getStorageKey(cacheKey))
}

function getCachedValue<K extends CacheKey>(cacheKey: K, ttl?: number): CacheValue<K> | null {
  // Try memory cache first
  let entry = (caches as any)[cacheKey] as CacheEntry<CacheValue<K>> | undefined

  // If not in memory, try localStorage
  if (!entry) {
    const loaded = loadFromStorage<CacheValue<K>>(cacheKey)
    if (loaded) {
      // Restore to memory cache (untracked to avoid mutation in reactive context)
      untrack(() => {
        ;(caches as any)[cacheKey] = loaded
      })
      entry = loaded
    }
  }

  if (!entry) return null

  // Check if TTL expired (still return stale data - stale-while-revalidate)
  return entry.value
}

function getValue<K extends CacheKey>(cacheKey: K): CacheValue<K> | undefined {
  const entry = (caches as any)[cacheKey] as CacheEntry<CacheValue<K>> | undefined
  return entry?.value
}

async function query<K extends CacheKey>(
  cacheKey: K,
  effect: () => Promise<CacheValue<K>>,
): Promise<CacheValue<K>> {
  const result = await effect()
  setCachedValue(cacheKey, result)
  return result
}

function setCachedValue<K extends CacheKey>(cacheKey: K, value: CacheValue<K>) {
  untrack(() => {
    const entry = {
      value,
      timestamp: Date.now(),
    }
    ;(caches as any)[cacheKey] = entry
    saveToStorage(cacheKey, entry)
  })
}

function invalidateCache(cacheKey: CacheKey) {
  delete (caches as any)[cacheKey]
  removeFromStorage(cacheKey)
}

function clearAllCache() {
  Object.keys(caches).forEach((key) => delete (caches as any)[key])
  if (typeof localStorage === "undefined") return

  const keysToRemove: string[] = []
  for (let i = 0; i < localStorage.length; i++) {
    const key = localStorage.key(i)
    if (key?.startsWith(STORAGE_PREFIX)) {
      keysToRemove.push(key)
    }
  }
  keysToRemove.forEach((key) => localStorage.removeItem(key))
}

// Cache API
export const cache: {
  get: typeof getCachedValue
  getValue: typeof getValue
  set: typeof setCachedValue
  query: typeof query
  invalidate: typeof invalidateCache
  clear: typeof clearAllCache
  data: typeof caches
} = {
  get: getCachedValue,
  getValue,
  set: setCachedValue,
  query,
  invalidate: invalidateCache,
  clear: clearAllCache,
  data: caches,
}

export { matchPromiseWithCache, matchPromise }
</script>

{#snippet matchPromiseWithCache<T, E, K extends CacheKey = CacheKey>(
  promise: Promise<T>,
  options: {
    cacheKey: K
    ttl?: number
    onLoading: Snippet<[]>
    onSuccess: Snippet<[T]>
    onError: Snippet<[E]>
  }
)}
  {@const cached = getCachedValue(options.cacheKey, options.ttl) as T | null}
  {#await promise}
    {#if cached}
      {@render options.onSuccess(cached)}
    {:else}
      {@render options.onLoading()}
    {/if}
  {:then value}
    {(setCachedValue(options.cacheKey, value as any), "")}
    {@render options.onSuccess(value)}
  {:catch error}
    {#if cached}
      {@render options.onSuccess(cached)}
    {:else}
      {@render options.onError(error)}
    {/if}
  {/await}
{/snippet}

{#snippet matchPromise<T, E>(
  promise: Promise<T>,
  options: {
    onLoading: Snippet<[]>
    onSuccess: Snippet<[T]>
    onError: Snippet<[E]>
  }
)}
  {#await promise}
    {@render options.onLoading()}
  {:then value}
    {@render options.onSuccess(value)}
  {:catch error}
    {@render options.onError(error)}
  {/await}
{/snippet}
