export type ClientStatus =
  | "idle"
  | "initializing"
  | "downloadStarted"
  | "downloading"
  | "downloadEnded"
  | "contributionStarted"
  | "contributionEnded"
  | "uploadStarted"
  | "uploadEnded"
  | "failed"
  | "successful"
  | "offline"
  | undefined

export interface ContributeBody {
  supabaseProject: string
  bucket: string
  jwt: string
  apiKey: string
  contributorId: string
  payloadId: string
  userEmail: string
}
