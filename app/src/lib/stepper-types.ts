export type Step = {
  status: StepStatus
  title: string
  description: string
  // kind of a hack to include it in this type
  traceDetails?: {
    chain_display_name: string
    tx: string
    block: string
    timestamp: string
    tx_url?: string
    block_url?: string
  }
}

export type StepStatus = "PENDING" | "IN_PROGRESS" | "COMPLETED" | "WARNING" | "ERROR"

export type RawTrace = {
  type: string
  height: number | null
  block_hash: string | null
  timestamp: string | null
  transaction_hash: string | null
  chain: {
    chain_id: string
  }
}

export type Trace = {
  status: StepStatus
  block_url: string | null
  transaction_url: string | null
} & RawTrace
