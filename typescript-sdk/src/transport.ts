import {
  RpcRequestError,
  createTransport,
  UrlRequiredError,
  type ClientConfig,
  type TransportConfig,
  type HttpTransportConfig
} from "viem"
import { sleep } from "./utilities/index.ts"
import type { RpcRequest } from "./types.ts"
import { getHttpRpcClient } from "viem/utils"
import { createBatchScheduler } from "./utilities/promise/batch-scheduler.ts"

export type Transport<TType extends string = string, TRpcAttributes = Record<string, any>> = ({
  pollingInterval,
  retryCount,
  timeout
}: {
  pollingInterval?: ClientConfig["pollingInterval"] | undefined
  retryCount?: TransportConfig["retryCount"] | undefined
  timeout?: TransportConfig["timeout"] | undefined
}) => {
  config: TransportConfig<TType>
  value?: TRpcAttributes | undefined
}

export type CosmosHttpTransport = Transport<
  "http",
  {
    fetchOptions?: HttpTransportConfig["fetchOptions"] | undefined
    url?: string | undefined
  }
>

/**
 * @description Creates a HTTP transport that connects to a JSON-RPC API.
 */
export function cosmosHttp(
  /** URL of the JSON-RPC API. Defaults to the chain's public RPC URL. */
  url?: string | undefined,
  config: HttpTransportConfig = {}
): CosmosHttpTransport {
  const {
    batch,
    fetchOptions,
    key = "http",
    name = "HTTP JSON-RPC",
    onFetchRequest,
    onFetchResponse,
    retryDelay
  } = config
  return ({ retryCount: retryCount_, timeout: timeout_ }) => {
    const { batchSize = 1000, wait = 0 } = typeof batch === "object" ? batch : {}
    const retryCount = config.retryCount ?? retryCount_
    const timeout = timeout_ ?? config.timeout ?? 10_000
    const url_ = url // || chain?.rpcUrls.default.http[0]
    if (!url_) throw new UrlRequiredError()

    const rpcClient = getHttpRpcClient(url_, {
      fetchOptions,
      onRequest: onFetchRequest,
      onResponse: onFetchResponse,
      timeout
    })

    return createTransport(
      {
        key,
        name,
        async request({ method, params }) {
          const body = { method, params }

          const { schedule } = createBatchScheduler({
            id: url_,
            wait,
            shouldSplitBatch(requests) {
              return requests.length > batchSize
            },
            fn: (body: Array<RpcRequest>) =>
              rpcClient.request({
                body
              }),
            sort: (a, b) => a.id - b.id
          })

          const fn = async (body: RpcRequest) =>
            batch
              ? schedule(body)
              : [
                  await rpcClient.request({
                    body
                  })
                ]

          const [{ error, result }] = await fn(body)
          if (error)
            throw new RpcRequestError({
              body,
              error,
              url: url_
            })
          return result
        },
        retryCount,
        retryDelay,
        timeout,
        type: "http"
      },
      {
        fetchOptions,
        url: url_
      }
    )
  }
}
/**
 * Given an array of rpc URLs, check the latency of each and return them ranked by latency
 */
export function rankCosmosRpcProviders({
  interval = 1_000,
  sampleCount = 10,
  timeout = 1_000,
  transports,
  weights = {}
}: {
  interval: number
  sampleCount: number
  timeout: number
  transports?: Array<string>
  weights?: { latency?: number; stability?: number }
}) {
  const { latency = 1, stability = 1 } = weights
  return {
    rank: async (rpcUrls?: Array<string>) => {
      const _transports = rpcUrls || transports
      if (!_transports) throw new Error("No transports provided")
      const results = await Promise.all(
        _transports.map(async rpcUrl => {
          const latencies = await Promise.all(
            Array.from({ length: sampleCount }, async () => {
              const start = Date.now()
              try {
                const controller = new AbortController()
                const timeoutSignal = AbortSignal.timeout(timeout)
                await fetch(rpcUrl, {
                  method: "head",
                  signal: AbortSignal.any([controller.signal, timeoutSignal])
                })

                return Date.now() - start
              } catch (error) {
                return Number.POSITIVE_INFINITY
              } finally {
                await sleep(interval)
              }
            })
          )
          const validLatencies = latencies.filter(latency => latency !== Number.POSITIVE_INFINITY)
          const stability = validLatencies.length
          const averageLatency = validLatencies.reduce((a, b) => a + b, 0) / stability
          return { rpcUrl, latency: averageLatency, stability }
        })
      )
      return results
        .sort((a, b) => {
          const aScore = a.latency * latency + a.stability * stability
          const bScore = b.latency * latency + b.stability * stability
          return aScore - bScore
        })
        .map(({ rpcUrl, latency }) => ({ rpcUrl, latency }))
    }
  }
}
