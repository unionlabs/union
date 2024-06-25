export type Step = {
  status: StepStatus
  title: string
  description: string
  // kind of a hack to include it in this type
  traceDetails?: {
    tx: string,
    block: string,
    time: string,
    tx_url?: string,
    block_url?: string,
  }
}

export type StepStatus = "PENDING" | "IN_PROGRESS" | "COMPLETED" | "ERROR"
