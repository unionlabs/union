import { browser } from "$app/environment"
import { Effect, Layer, Logger, LogLevel } from "effect"
import { CosmosClient, CosmosClientLive, type CosmosClientConfig } from "$lib/services/cosmos-client"
import { getChain, DEFAULT_CHAIN, type ChainConfig } from "$lib/chains/config"
import { chainStore } from "$lib/stores/chain.svelte"

// Proxy endpoints for browser requests to avoid CORS
const REST_PROXY = "/api/cosmos"
const RPC_PROXY = "/api/rpc"

// Cache of runtimes by universal chain ID
const runtimeCache = new Map<string, ReturnType<typeof createChainRuntime>>()

// Create a runtime for a specific chain (using universal_chain_id)
export const createChainRuntime = (universalChainId: string) => {
  const chain = getChain(universalChainId) ?? getChain(DEFAULT_CHAIN)!

  // Use proxy for browser, direct API for server
  // Browser: single proxy endpoint (proxy handles racing internally)
  // Server: race all endpoints directly
  const restEndpoint = browser ? REST_PROXY : chain.api[0]
  const rpcEndpoint = browser ? RPC_PROXY : chain.rpc[0]

  const config: CosmosClientConfig = {
    restEndpoint,
    // Additional REST endpoints for racing (server-side only)
    ...(browser ? {} : { restEndpoints: chain.api.slice(1) }),
    rpcEndpoint,
    // Additional RPC endpoints for racing (server-side only)
    ...(browser ? {} : { rpcEndpoints: chain.rpc.slice(1) }),
    chainName: universalChainId,  // Pass universal chain ID for proxy header
  }

  const MainLayer = CosmosClientLive(config).pipe(
    Layer.provide(Logger.minimumLogLevel(LogLevel.Warning)),
  )

  return {
    chain,
    config,
    runPromise: <A, E>(effect: Effect.Effect<A, E, CosmosClient>) =>
      Effect.runPromise(effect.pipe(Effect.provide(MainLayer))),
  }
}

// Get or create a runtime for a chain
function getRuntime(universalChainId: string) {
  let runtime = runtimeCache.get(universalChainId)
  if (!runtime) {
    runtime = createChainRuntime(universalChainId)
    runtimeCache.set(universalChainId, runtime)
  }
  return runtime
}

// Default runtime for Union mainnet (used during SSR)
export const defaultRuntime = createChainRuntime(DEFAULT_CHAIN)

// Get the current runtime based on selected chain
export function getCurrentRuntime() {
  return getRuntime(chainStore.id)
}

// Get the current chain config
export function getCurrentChain(): ChainConfig {
  return chainStore.config
}

// Run an effect with the current chain's runtime
export const runPromise = <A, E>(effect: Effect.Effect<A, E, CosmosClient>) =>
  getCurrentRuntime().runPromise(effect)
