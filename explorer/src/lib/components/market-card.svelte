<script lang="ts">
import CornerMarks from "$lib/components/corner-marks.svelte"
import { Skeleton } from "$lib/components/ui/skeleton/index.js"
import { chainStore } from "$lib/stores/chain.svelte"
import TrendingDownIcon from "@lucide/svelte/icons/trending-down"
import TrendingUpIcon from "@lucide/svelte/icons/trending-up"
import { onMount } from "svelte"

interface PriceData {
  current_price: number
  price_change_percentage_24h: number
  market_cap: number
  total_volume: number
  prices: [number, number][] // [timestamp, price]
  volumes: [number, number][] // [timestamp, volume]
}

interface CachedData {
  data: PriceData
  timestamp: number
  range: string
  coinId: string
}

let priceData = $state<PriceData | null>(null)
let loading = $state(true)
let error = $state(false)
let selectedRange = $state<"1" | "7" | "30" | "90" | "365">("7")
let hoverIndex = $state<number | null>(null)
let chartEl = $state<HTMLDivElement | null>(null)

// Get coingecko_id from current chain's primary asset
const coinId = $derived(chainStore.config.assets[0]?.coingecko_id)
const tokenSymbol = $derived(chainStore.config.assets[0]?.symbol ?? "TOKEN")

const CACHE_KEY = "market-data-cache"
const CACHE_TTL = 10 * 60 * 1000 // 10 minutes
const REFRESH_INTERVAL = 10 * 60 * 1000 // 10 minutes
const MIN_FETCH_INTERVAL = 30 * 1000 // 30 seconds between fetches per coin

// Track last fetch time per coin to prevent rapid re-fetches
const lastFetchTimes = new Map<string, number>()

const RANGES = [
  { value: "1", label: "24H" },
  { value: "7", label: "7D" },
  { value: "30", label: "30D" },
  { value: "90", label: "90D" },
  { value: "365", label: "1Y" },
] as const

function getCachedData(id: string, range: string, ignoreExpiry = false): PriceData | null {
  try {
    const cached = localStorage.getItem(`${CACHE_KEY}-${id}-${range}`)
    if (!cached) {
      return null
    }
    const parsed: CachedData = JSON.parse(cached)
    if (!ignoreExpiry && Date.now() - parsed.timestamp > CACHE_TTL) {
      return null
    }
    if (parsed.coinId !== id) {
      return null
    }
    return parsed.data
  } catch {
    return null
  }
}

function canFetch(id: string): boolean {
  const lastFetch = lastFetchTimes.get(id)
  if (!lastFetch) {
    return true
  }
  return Date.now() - lastFetch >= MIN_FETCH_INTERVAL
}

function setCachedData(id: string, range: string, data: PriceData) {
  try {
    const cached: CachedData = { data, timestamp: Date.now(), range, coinId: id }
    localStorage.setItem(`${CACHE_KEY}-${id}-${range}`, JSON.stringify(cached))
  } catch {
    // localStorage might be full or unavailable
  }
}

async function fetchPriceData(id: string | undefined, days = selectedRange, forceRefresh = false) {
  if (!id) {
    // No coingecko_id for this chain
    priceData = null
    loading = false
    error = false
    return
  }

  // Always check cache first (valid cache)
  const cached = getCachedData(id, days)
  if (cached) {
    priceData = cached
    loading = false
    // If not forcing refresh, we're done
    if (!forceRefresh) {
      return
    }
  }

  // Check rate limiting - don't fetch too frequently
  if (!canFetch(id)) {
    // Use stale cache if available, otherwise keep current state
    const staleCache = getCachedData(id, days, true)
    if (staleCache && !priceData) {
      priceData = staleCache
    }
    loading = false
    return
  }

  // Only show loading if we have no data at all
  if (!priceData) {
    loading = true
  }
  error = false

  try {
    // Record fetch time before making request
    lastFetchTimes.set(id, Date.now())

    const [priceRes, chartRes] = await Promise.all([
      fetch(
        `https://api.coingecko.com/api/v3/simple/price?ids=${id}&vs_currencies=usd&include_24hr_change=true&include_market_cap=true&include_24hr_vol=true`,
      ),
      fetch(
        `https://api.coingecko.com/api/v3/coins/${id}/market_chart?vs_currency=usd&days=${days}`,
      ),
    ])

    // Handle rate limiting
    if (priceRes.status === 429 || chartRes.status === 429) {
      console.warn("CoinGecko rate limited, using cached data")
      const staleCache = getCachedData(id, days, true)
      if (staleCache) {
        priceData = staleCache
        loading = false
        return
      }
      throw new Error("Rate limited and no cache available")
    }

    const priceJson = await priceRes.json()
    const chartJson = await chartRes.json()
    const coinData = priceJson[id]

    if (!coinData) {
      throw new Error("No data returned for coin")
    }

    const newData: PriceData = {
      current_price: coinData.usd,
      price_change_percentage_24h: coinData.usd_24h_change,
      market_cap: coinData.usd_market_cap,
      total_volume: coinData.usd_24h_vol,
      prices: chartJson.prices,
      volumes: chartJson.total_volumes,
    }

    priceData = newData
    setCachedData(id, days, newData)
    loading = false
  } catch (e) {
    console.error("Failed to fetch price data:", e)
    // Try to use stale cache on any error
    const staleCache = getCachedData(id, days, true)
    if (staleCache) {
      priceData = staleCache
      loading = false
    } else if (!priceData) {
      error = true
      loading = false
    } else {
      // Keep existing data, just stop loading
      loading = false
    }
  }
}

// Track current coinId for the interval
let currentCoinId = $state<string | undefined>(undefined)
let mounted = $state(false)
let refreshInterval: ReturnType<typeof setInterval> | null = null

// Refetch when coinId changes (only after mount to avoid duplicate with onMount)
$effect(() => {
  const id = coinId
  if (mounted && id !== currentCoinId) {
    currentCoinId = id
    // Try to load from cache first before showing loading state
    const cached = id ? getCachedData(id, selectedRange) : null
    if (cached) {
      priceData = cached
      loading = false
    } else {
      priceData = null
      loading = true
    }
    error = false
    fetchPriceData(id, selectedRange)
  }
})

onMount(() => {
  mounted = true
  currentCoinId = coinId
  fetchPriceData(coinId, selectedRange)

  refreshInterval = setInterval(() => {
    if (coinId) {
      fetchPriceData(coinId, selectedRange, true)
    }
  }, REFRESH_INTERVAL)

  return () => {
    if (refreshInterval) {
      clearInterval(refreshInterval)
    }
  }
})

function formatPrice(price: number): string {
  if (price >= 1) {
    return `$${price.toFixed(2)}`
  }
  if (price >= 0.01) {
    return `$${price.toFixed(4)}`
  }
  return `$${price.toFixed(6)}`
}

function formatLargeNumber(num: number): string {
  if (num >= 1e9) {
    return `$${(num / 1e9).toFixed(2)}B`
  }
  if (num >= 1e6) {
    return `$${(num / 1e6).toFixed(2)}M`
  }
  if (num >= 1e3) {
    return `$${(num / 1e3).toFixed(2)}K`
  }
  return `$${num.toFixed(2)}`
}

interface ChartData {
  line: string
  area: string
  volumeBars: { x: number; y: number; w: number; h: number }[]
  points: { x: number; y: number; price: number; volume: number; timestamp: number }[]
}

// Generate angular SVG path with straight lines
function generatePath(
  prices: [number, number][],
  volumes: [number, number][] | undefined,
  width: number,
  height: number,
): ChartData {
  if (!prices || prices.length === 0) {
    return { line: "", area: "", volumeBars: [], points: [] }
  }

  // Downsample for performance
  const maxPoints = 100
  const step = Math.max(1, Math.floor(prices.length / maxPoints))
  const sampledPrices = prices.filter((_, i) => i % step === 0)
  const sampledVolumes = volumes?.filter((_, i) => i % step === 0) ?? []

  const priceValues = sampledPrices.map(([_, p]) => p)
  const min = Math.min(...priceValues)
  const max = Math.max(...priceValues)
  const range = max - min || 1
  const pad = height * 0.08

  const points = sampledPrices.map(([timestamp, price], i) => {
    const volume = sampledVolumes[i]?.[1] ?? 0
    return {
      x: (i / (sampledPrices.length - 1)) * width,
      y: pad + (height - 2 * pad) * (1 - (price - min) / range),
      price,
      volume,
      timestamp,
    }
  })

  // Build angular path with straight lines
  const line = points.map((p, i) => `${i === 0 ? "M" : "L"} ${p.x},${p.y}`).join(" ")

  const lastPoint = points[points.length - 1]
  const area = `${line} L ${lastPoint.x},${height} L 0,${height} Z`

  // Generate volume bars (bottom 25% of chart) - centered on each point's x
  let volumeBars: { x: number; y: number; w: number; h: number }[] = []
  if (sampledVolumes.length > 0) {
    const volValues = sampledVolumes.map(([_, v]) => v)
    const volMax = Math.max(...volValues) || 1
    const volHeight = height * 0.25
    const barWidth = width / (sampledVolumes.length - 1 || 1)
    volumeBars = points.map((pt, i) => {
      const vol = sampledVolumes[i]?.[1] ?? 0
      const h = (vol / volMax) * volHeight
      const w = Math.max(barWidth * 0.8, 1)
      return {
        x: pt.x - w / 2,
        y: height - h,
        w,
        h,
      }
    })
  }

  return { line, area, volumeBars, points }
}

function handleMouseMove(e: MouseEvent, pointsCount: number) {
  if (!chartEl) {
    return
  }
  const rect = chartEl.getBoundingClientRect()
  const x = e.clientX - rect.left
  const pct = x / rect.width
  const idx = Math.round(pct * (pointsCount - 1))
  hoverIndex = Math.max(0, Math.min(pointsCount - 1, idx))
}

function handleMouseLeave() {
  hoverIndex = null
}

function formatTimestamp(ts: number): string {
  const date = new Date(ts)
  return date.toLocaleDateString(undefined, {
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  })
}
</script>

<div class="relative border border-border">
  <CornerMarks />

  <div class="flex items-center justify-between px-4 py-3 border-b border-border bg-muted/20">
    <span class="text-xs font-medium uppercase tracking-wider">{tokenSymbol} / USD</span>
    {#if coinId}
      <a
        href="https://www.coingecko.com/en/coins/{coinId}"
        target="_blank"
        rel="noopener noreferrer"
        class="text-[10px] font-mono text-muted-foreground hover:text-foreground"
      >
        CoinGecko
      </a>
    {/if}
  </div>

  {#if loading}
    <div class="p-4 space-y-4">
      <div class="flex items-end justify-between">
        <Skeleton class="h-10 w-32" />
        <Skeleton class="h-5 w-16" />
      </div>
      <Skeleton class="h-32 w-full" />
      <div class="grid grid-cols-2 gap-4">
        <Skeleton class="h-12" />
        <Skeleton class="h-12" />
      </div>
    </div>
  {:else if !coinId}
    <div class="p-4 text-center text-muted-foreground text-sm py-12">
      No market data available for this chain
    </div>
  {:else if error}
    <div class="p-4 text-center text-muted-foreground text-sm py-12">
      Failed to load market data
    </div>
  {:else if priceData}
    {@const isPositive = priceData.price_change_percentage_24h >= 0}
    {@const changeColor = isPositive ? "text-green-500" : "text-red-500"}
    {@const strokeColor = isPositive ? "#22c55e" : "#ef4444"}
    {@const paths = generatePath(priceData.prices, priceData.volumes, 400, 120)}

    <div class="p-4 space-y-3">
      <!-- Price and Change -->
      <div class="flex items-end justify-between">
        <div class="text-3xl font-mono font-bold">{formatPrice(priceData.current_price)}</div>
        <div class="flex items-center gap-1 {changeColor}">
          {#if isPositive}
            <TrendingUpIcon class="h-4 w-4" />
          {:else}
            <TrendingDownIcon class="h-4 w-4" />
          {/if}
          <span class="font-mono text-sm font-medium">
            {isPositive ? "+" : ""}{priceData.price_change_percentage_24h.toFixed(2)}%
          </span>
        </div>
      </div>

      <!-- Chart -->
      <div
        class="h-32 -mx-4 relative cursor-crosshair"
        bind:this={chartEl}
        onmousemove={(e) => handleMouseMove(e, paths.points.length)}
        onmouseleave={handleMouseLeave}
        role="img"
      >
        <svg
          width="100%"
          height="100%"
          viewBox="0 0 400 120"
          preserveAspectRatio="none"
        >
          <defs>
            <linearGradient
              id="chartGradient"
              x1="0"
              y1="0"
              x2="0"
              y2="1"
            >
              <stop
                offset="0%"
                stop-color={strokeColor}
                stop-opacity="0.2"
              />
              <stop
                offset="100%"
                stop-color={strokeColor}
                stop-opacity="0"
              />
            </linearGradient>
          </defs>
          <!-- Volume bars -->
          {#each paths.volumeBars as bar}
            <rect
              x={bar.x}
              y={bar.y}
              width={bar.w}
              height={bar.h}
              fill="currentColor"
              class="text-muted-foreground/15"
            />
          {/each}
          <path
            d={paths.area}
            fill="url(#chartGradient)"
          />
          <path
            d={paths.line}
            fill="none"
            stroke={strokeColor}
            stroke-width="1.5"
            stroke-linecap="square"
            stroke-linejoin="miter"
            vector-effect="non-scaling-stroke"
          />
          <!-- Crosshair line -->
          {#if hoverIndex !== null && paths.points[hoverIndex]}
            {@const pt = paths.points[hoverIndex]}
            <line
              x1={pt.x}
              y1="0"
              x2={pt.x}
              y2="120"
              stroke="currentColor"
              stroke-width="1"
              stroke-dasharray="2,2"
              class="text-muted-foreground/50"
              vector-effect="non-scaling-stroke"
            />
            <!-- Highlight volume bar -->
            {#if paths.volumeBars[hoverIndex]}
              {@const bar = paths.volumeBars[hoverIndex]}
              <rect
                x={bar.x}
                y={bar.y}
                width={bar.w}
                height={bar.h}
                fill="currentColor"
                class="text-muted-foreground/40"
              />
            {/if}
          {/if}
        </svg>
        <!-- Hover dot (outside SVG to stay round) -->
        {#if hoverIndex !== null && paths.points[hoverIndex]}
          {@const pt = paths.points[hoverIndex]}
          {@const xPct = pt.x / 400 * 100}
          {@const yPct = pt.y / 120 * 100}
          <div
            class="absolute w-2 h-2 rounded-full pointer-events-none -translate-x-1/2 -translate-y-1/2"
            style="left: {xPct}%; top: {yPct}%; background-color: {strokeColor};"
          >
          </div>
        {/if}
        <!-- Tooltip -->
        {#if hoverIndex !== null && paths.points[hoverIndex]}
          {@const pt = paths.points[hoverIndex]}
          {@const pct = pt.x / 400}
          <div
            class="absolute top-0 pointer-events-none px-2 py-1.5 bg-background/95 border border-border text-xs font-mono space-y-0.5"
            style="left: {pct < 0.5 ? `${pct * 100 + 2}%` : 'auto'}; right: {pct >= 0.5 ? `${(1 - pct) * 100 + 2}%` : 'auto'};"
          >
            <div class="text-muted-foreground">{formatTimestamp(pt.timestamp)}</div>
            <div>{formatPrice(pt.price)}</div>
            <div class="text-muted-foreground">Vol: {formatLargeNumber(pt.volume)}</div>
          </div>
        {/if}
      </div>

      <!-- Range selector -->
      <div class="flex items-center justify-center gap-1">
        {#each RANGES as range}
          <button
            onclick={() => {
              selectedRange = range.value
              fetchPriceData(coinId, range.value)
            }}
            class="px-2 py-1 text-[10px] font-mono transition-colors {selectedRange === range.value ? 'bg-foreground text-background' : 'text-muted-foreground hover:text-foreground'}"
          >
            {range.label}
          </button>
        {/each}
      </div>

      <!-- Stats -->
      <div class="grid grid-cols-2 gap-4 pt-3 border-t border-border">
        <div>
          <div class="text-[10px] font-mono uppercase text-muted-foreground mb-1">Market Cap</div>
          <div class="font-mono text-sm">{formatLargeNumber(priceData.market_cap)}</div>
        </div>
        <div>
          <div class="text-[10px] font-mono uppercase text-muted-foreground mb-1">24h Volume</div>
          <div class="font-mono text-sm">{formatLargeNumber(priceData.total_volume)}</div>
        </div>
      </div>
    </div>
  {/if}
</div>
