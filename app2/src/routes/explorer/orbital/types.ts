import type { TransferListItem } from "@unionlabs/sdk/schema"

export type EnhancedTransferListItem = TransferListItem & {
  isTestnetTransfer?: boolean
  sourceDisplayName?: string
  destinationDisplayName?: string
  formattedTimestamp?: string
  routeKey?: string
  senderDisplay?: string
  receiverDisplay?: string
}

export interface NodeHealthData {
  totalNodes: number
  healthyNodes: number
  degradedNodes: number
  unhealthyNodes: number
  avgResponseTime: number
  nodesWithRpcs: Array<{
    chainId: string
    chainName: string
    rpcUrl: string
    rpcType: string
    status: string
    responseTimeMs: number
    lastCheckTime: number
    latestBlockHeight?: number
    errorMessage?: string
  }>
  chainHealthStats: Record<string, {
    chainName: string
    healthyNodes: number
    totalNodes: number
    avgResponseTime: number
  }>
}

export interface ChainFlowData {
  chains: ChainData[]
  chainFlowTimeScale: Record<string, ChainData[]>
  totalOutgoing: number
  totalIncoming: number
  serverUptimeSeconds: number
}

export interface LatencyData {
  sourceName: string
  destinationName: string
  packetRecv: { p5: number; median: number; p95: number }
  writeAck: { p5: number; median: number; p95: number }
  packetAck: { p5: number; median: number; p95: number }
}

export interface ChartData {
  popularRoutes: RouteData[]
  activeSenders: WalletStats[]
  activeReceivers: WalletStats[]
  currentRates: TransferRates
  popularRoutesTimeScale: Record<string, RouteData[]>
  activeSendersTimeScale: Record<string, WalletStats[]>
  activeReceiversTimeScale: Record<string, WalletStats[]>
  chainFlowData: ChainFlowData
  assetVolumeData: AssetVolumeData
  latencyData: LatencyData[]
  nodeHealthData: NodeHealthData
}

export interface NodeData {
  chainId: string
  chainName: string
  rpcUrl: string
  rpcType: string
  status: string // "healthy", "degraded", "unhealthy"
  responseTimeMs: number
  lastCheckTime: number
  latestBlockHeight?: number
  errorMessage?: string
}

export interface ChainHealthStat {
  chainName: string
  healthyNodes: number
  totalNodes: number
  avgResponseTime: number
}

export interface NodeHealthSummary {
  totalNodes: number
  healthyNodes: number
  degradedNodes: number
  unhealthyNodes: number
  avgResponseTime: number
  nodesWithRpcs: NodeData[]
  chainHealthStats: Record<string, ChainHealthStat>
}

export interface WalletStats {
  count: number
  address: string
  displayAddress: string
  lastActivity: string
}

export interface ActiveWalletRates {
  sendersLastMin: number
  sendersLastHour: number
  sendersLastDay: number
  sendersLast7d: number
  sendersLast14d: number
  sendersLast30d: number

  receiversLastMin: number
  receiversLastHour: number
  receiversLastDay: number
  receiversLast7d: number
  receiversLast14d: number
  receiversLast30d: number

  totalLastMin: number
  totalLastHour: number
  totalLastDay: number
  totalLast7d: number
  totalLast14d: number
  totalLast30d: number

  uniqueSendersTotal: number
  uniqueReceiversTotal: number
  uniqueTotalWallets: number
}

export interface TransferRates {
  txPerMinute: number
  txPerHour: number
  txPerDay: number
  txPer7Days: number
  txPer14Days: number
  txPer30Days: number
  txPerMinuteChange?: number
  txPerHourChange?: number
  txPerDayChange?: number
  txPer7DaysChange?: number
  txPer14DaysChange?: number
  txPer30DaysChange?: number
  totalTracked: number
  serverUptimeSeconds: number
}

export interface ActiveWalletRates {
  sendersLastMin: number
  sendersLastHour: number
  sendersLastDay: number
  sendersLast7d: number
  sendersLast14d: number
  sendersLast30d: number
  sendersLastMinChange?: number
  sendersLastHourChange?: number
  sendersLastDayChange?: number
  sendersLast7dChange?: number
  sendersLast14dChange?: number
  sendersLast30dChange?: number
  receiversLastMin: number
  receiversLastHour: number
  receiversLastDay: number
  receiversLast7d: number
  receiversLast14d: number
  receiversLast30d: number
  receiversLastMinChange?: number
  receiversLastHourChange?: number
  receiversLastDayChange?: number
  receiversLast7dChange?: number
  receiversLast14dChange?: number
  receiversLast30dChange?: number
  totalLastMin: number
  totalLastHour: number
  totalLastDay: number
  totalLast7d: number
  totalLast14d: number
  totalLast30d: number
  totalLastMinChange?: number
  totalLastHourChange?: number
  totalLastDayChange?: number
  totalLast7dChange?: number
  totalLast14dChange?: number
  totalLast30dChange?: number
  uniqueSendersTotal: number
  uniqueReceiversTotal: number
  uniqueTotalWallets: number
  serverUptimeSeconds: number
}

export interface LogEntry {
  id: number
  timestamp: string
  type: string
  message: string
  sourceChain: string
  destChain: string
  hash: string
  sender?: string | undefined
  receiver?: string | undefined
  sourceChainId?: string | undefined
  destChainId?: string | undefined
}

export interface RouteData {
  route: string
  count: number
  fromChain: string
  toChain: string
  fromName: string
  toName: string
  countChange?: number
}

export interface ChainAsset {
  assetSymbol: string
  assetName?: string
  outgoingCount: number
  incomingCount: number
  netFlow: number
  totalVolume: number
  averageAmount: number
  percentage: number
  lastActivity: string
}

export interface ChainData {
  universal_chain_id: string
  chainName: string
  outgoingCount: number
  incomingCount: number
  netFlow: number
  outgoingChange?: number
  incomingChange?: number
  netFlowChange?: number
  lastActivity: string
  topAssets?: ChainAsset[]
}

export interface AssetRoute {
  fromChain: string
  toChain: string
  fromName: string
  toName: string
  route: string
  count: number
  volume: number
  percentage: number
  lastActivity: string
}

export interface Asset {
  assetSymbol: string
  assetName: string
  transferCount: number
  totalVolume: number
  averageAmount: number
  largestTransfer: number
  volumeChange?: number
  countChange?: number
  lastActivity: string
  topRoutes: AssetRoute[]
}

export interface AssetVolumeData {
  assets: Asset[]
  assetVolumeTimeScale: Record<string, Asset[]>
  totalAssets: number
  totalVolume: number
  totalTransfers: number
  serverUptimeSeconds: number
}
