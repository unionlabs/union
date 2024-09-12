export type CheckQueueResult = {
  user: number
  current: number
  length: number | null
} | null

export interface ContributionStatus {
  canContribute: boolean
  shouldContribute: boolean
  isVerifying: boolean
}
