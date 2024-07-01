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
