export interface ClientStatus {
  status: "idle" | "initializing" | "contributionStarted" | "contributionEnded" | "successful" | "offline"
  downloadStarted?: string
  downloading?: {
    file: string
    progress: number
  }
  downloadEnded?: string
  uploadStarted?: string
  uploadEnded?: string
  failed?: string
}

export interface ContributeBody {
  supabaseProject: string
  bucket: string
  jwt: string
  apiKey: string
  contributorId: string
  payloadId: string
  userEmail: string
}