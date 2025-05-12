<script lang="ts">
import { Effect, Option, Either } from "effect"
import Card from "$lib/components/ui/Card.svelte"
import Sections from "$lib/components/ui/Sections.svelte"
import { cn } from "$lib/utils"
import Skeleton from "$lib/components/ui/Skeleton.svelte"
import Tooltip from "$lib/components/ui/Tooltip.svelte"
import type { Chain } from "@unionlabs/sdk/schema"
import { chains } from "$lib/stores/chains.svelte"
import type { RpcProtocolType } from "@unionlabs/sdk/schema"
    import { runPromise } from "$lib/runtime";

type RpcType = "cosmos" | "evm"

interface RpcStatusResult {
  url: string
  type: RpcType
  responseTimeMs: number
  status: CosmosStatus | EvmStatus
}

interface CosmosStatus {
  kind: "cosmos"
  latestBlockHeight: number
  catchingUp: boolean
  moniker: string
  network: string
}

interface EvmStatus {
  kind: "evm"
  latestBlockHex: string
  latestBlockNumber: number
}

class RpcStatusError {
  readonly _tag = "RpcStatusError"
  readonly type: RpcType
  readonly url: string
  readonly message: string
  readonly cause?: unknown

  constructor(type: RpcType, url: string, message: string, cause?: unknown) {
    this.type = type
    this.url = url
    this.message = message
    this.cause = cause
  }
}

const withTiming = async (fn: () => Promise<any>) => {
  const start = performance.now()
  const result = await fn()
  const end = performance.now()
  return { result, duration: end - start }
}

const checkRpcStatus = (
  type: RpcType,
  url: string
): Effect.Effect<RpcStatusResult, RpcStatusError> =>
  Effect.tryPromise({
    try: async signal => {
      if (type === "cosmos") {
        const { result: res, duration } = await withTiming(async () =>
          fetch(url, {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({
              jsonrpc: "2.0",
              id: 1,
              method: "status",
              params: []
            })
          })
        )
        if (!res.ok)
          throw new RpcStatusError("cosmos", url, `HTTP ${res.status} - ${res.statusText}`)
        const data = await res.json()
        if (!(data?.result?.sync_info && data?.result?.node_info)) {
          throw new RpcStatusError("cosmos", url, "Malformed response", data)
        }
        return {
          url,
          type: "cosmos",
          responseTimeMs: duration,
          status: {
            kind: "cosmos",
            latestBlockHeight: Number(data.result.sync_info.latest_block_height),
            catchingUp: data.result.sync_info.catching_up,
            moniker: data.result.node_info.moniker,
            network: data.result.node_info.network
          }
        }
      }
      if (type === "evm") {
        const { result: res, duration } = await withTiming(async () =>
          fetch(url, {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({
              jsonrpc: "2.0",
              id: 1,
              method: "eth_blockNumber",
              params: []
            })
          })
        )
        if (!res.ok) throw new RpcStatusError("evm", url, `HTTP ${res.status} - ${res.statusText}`)
        const data = await res.json()
        if (!data?.result) {
          throw new RpcStatusError("evm", url, "Missing result field", data)
        }
        const hex = data.result as string
        return {
          url,
          type: "evm",
          responseTimeMs: duration,
          status: {
            kind: "evm",
            latestBlockHex: hex,
            latestBlockNumber: Number.parseInt(hex, 16)
          }
        }
      }
      throw new RpcStatusError(type, url, `Unsupported type: ${type}`)
    },
    catch: err =>
      err instanceof RpcStatusError ? err : new RpcStatusError(type, url, "Unknown error", err)
  })

type NodeStatus = {
  chain: Chain
  rpcUrl: string
  status: "CHECKING" | "OK" | "ERROR"
  responseTime?: number
  error?: string
}

let nodeData: Map<string, NodeStatus> = $state(new Map())
let hasInitialized = $state(false)

const checkNode = (chain: Chain, rpcUrl: string) =>
  Effect.gen(function* (_) {
    const key = `${chain.universal_chain_id}-${rpcUrl}`

    nodeData = new Map(
      nodeData.set(key, {
        chain,
        rpcUrl,
        status: "CHECKING"
      })
    )

    const result = yield* _(
      checkRpcStatus(chain.rpc_type === "cosmos" ? "cosmos" : "evm", rpcUrl).pipe(
        Effect.map(res =>
          Either.right({
            status: "OK" as const,
            responseTime: Math.round(res.responseTimeMs)
          })
        ),
        Effect.catchAll((err: RpcStatusError) =>
          Effect.succeed(
            Either.left({
              status: "ERROR" as const,
              error: err.message
            })
          )
        )
      )
    )

    const status = Either.match(result, {
      onLeft: error => error,
      onRight: success => success
    })

    nodeData = new Map(
      nodeData.set(key, {
        chain,
        rpcUrl,
        ...status
      })
    )

    return status
  })

async function checkAllNodes() {
  const chainsData = Option.getOrElse(chains.data, () => [])
  const rpcNodes = chainsData.flatMap(chain =>
    chain.rpcs.filter(rpc => rpc.type === ("rpc" as RpcProtocolType)).map(rpc => ({ chain, rpc }))
  )

  await Promise.all(
    rpcNodes.map(({ chain, rpc }) => checkNode(chain, rpc.url).pipe(runPromise))
  )
}

$effect(() => {
  if (Option.isSome(chains.data) && !hasInitialized) {
    hasInitialized = true
    checkAllNodes()
  }
})

setInterval(() => {
  checkAllNodes()
}, 30000)
</script>

<Sections>
  <Card class="overflow-auto" divided>
    <div class="p-3 text-sm font-medium text-zinc-400">Node Status</div>
    <div class="space-y-1">
      {#if Option.isNone(chains.data)}
        {#each Array(3) as _}
          <div class="flex justify-between gap-8 px-4 py-2 h-12 items-center">
            <div class="space-y-2">
              <Skeleton class="h-4 w-24" />
              <Skeleton class="h-3 w-32" />
            </div>
            <div class="flex items-center gap-2">
              <Skeleton class="h-5 w-16" />
              <Skeleton class="h-5 w-16" />
            </div>
          </div>
        {/each}
      {:else}
        {#each Option.getOrElse(chains.data, () => []) as chain}
          {#each chain.rpcs.filter((rpc) => rpc.type === ("rpc" as RpcProtocolType)) as rpc}
            {@const key = `${chain.universal_chain_id}-${rpc.url}`}
            {@const status = nodeData.get(key)}
            <a
              href={rpc.url}
              target="_blank"
              rel="noopener noreferrer"
              class={cn(
                "flex justify-between gap-8 px-4 py-2 h-12 items-center",
                "hover:bg-zinc-50 dark:hover:bg-zinc-900 transition-colors duration-75",
                "cursor-pointer"
              )}
            >
              <div>
                <h2 class="text-sm font-medium">{chain.display_name}</h2>
                <p class="text-xs text-zinc-400">{rpc.url}</p>
              </div>
              <div class="flex items-center gap-2">
                {#if status?.responseTime}
                  <span
                    class="px-2 py-0.5 text-xs font-medium bg-zinc-500/20 text-zinc-500 rounded-sm"
                  >
                    {status.responseTime}ms
                  </span>
                {/if}
                {#if status?.error}
                  <Tooltip>
                    {#snippet trigger()}
                      <span
                        class="px-2 py-0.5 text-xs font-medium bg-red-500/20 text-red-500 rounded-sm"
                      >
                        ERROR
                      </span>
                    {/snippet}
                    {#snippet content()}
                      <span class="text-red-500">{status.error}</span>
                    {/snippet}
                  </Tooltip>
                {:else}
                  <span
                    class={cn(
                      "px-2 py-0.5 text-xs font-medium rounded-sm",
                      !status
                        ? "bg-zinc-500/20 text-zinc-500"
                        : status.status === "CHECKING"
                          ? "bg-accent/20 text-accent"
                          : status.status === "OK"
                            ? "bg-emerald-500/20 text-emerald-500"
                            : "bg-zinc-500/20 text-zinc-500"
                    )}
                  >
                    {!status ? "PENDING" : status.status}
                  </span>
                {/if}
              </div>
            </a>
          {/each}
        {/each}
      {/if}
    </div>
  </Card>
</Sections>
