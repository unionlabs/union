import type { Entity } from "../client"

// Wallet verification constants
export const WALLET_VERIFICATION_MESSAGE =
  "I confirm that I wish to receive any rewards allocated to me in this wallet"
export const STARGAZE_VERIFICATION_MESSAGE =
  "I confirm this is my Stargaze wallet for Union NFT verification"

export const SUPPORTED_CHAINS = {
  "1": "Ethereum Mainnet",
  "11155111": "Sepolia Testnet",
} as const

export type SupportedChainId = keyof typeof SUPPORTED_CHAINS

// Database entity types
export type UserAchievement = Entity<"user_achievements">
export type UserExperience = Entity<"leaderboard">
export type UserMission = Entity<"user_missions">
export type UserReward = Entity<"user_rewards_with_queue">
export type Wallet = Entity<"wallets">
export type Device = Entity<"devices">
export type ReferralCode = Entity<"referral_codes">
export type UserAllocation = Entity<"user_allocations">

// Input types for functions
export interface DeviceInsert {
  ipAddress: string
  userId: string
  deviceIdentifier: string
}

export interface SubmitWalletVerificationInput {
  id: string
  address: string
  chainId: string
  message: string
  signature: string
  selectedChains: Array<string | null> | null
}

// Response types for edge functions
export type UpdatePreStakeResponse = {
  success: true
  message: string
} | {
  success: false
  error: string
}

export type UpdateWalletResponse = {
  success: true
  message: string
} | {
  success: false
  error: string
}

export type GenerateReferralCodeResponse = {
  success: true
  message: string
} | {
  success: false
  error: string
}

export type ClaimReferralCodeResponse = {
  success: true
  message: string
} | {
  success: false
  error: string
}

export type RemoveReferralCodeResponse = {
  success: true
  allocation: UserAllocation
  message: string
  percentage_returned: number
} | {
  success: false
  error: string
}

export type VerifyWalletSignatureResponse = {
  success: true
  message: string
  data: {
    wallet_address: string
    chain_id: string
    chain_name: string
  }
} | {
  success: false
  error: string
}

export type VerifyStargazeWalletResponse = {
  success: true
  message: string
  data: {
    address: string
    allocation: UserAllocation
  }
} | {
  success: false
  error: string
}

export type CreateAirdropEntryResponse = {
  success: true
  message: string
  data: {
    allocation: UserAllocation
    already_exists: boolean
  }
} | {
  success: false
  error: string
}

export type UpdateTwitterResponse = {
  success: true
  message: string
  data: {
    allocation: UserAllocation
    twitter_id: string
    twitter_username?: string
  }
} | {
  success: false
  error: string
}

export type ScanAllocationResponse = {
  success: true
  message: string
  data: {
    allocation: UserAllocation
    already_scanned: boolean
    is_eligible?: boolean
    matched_by?: "dashboard_id" | "twitter" | "stargaze_address"
    master_data?: {
      dashboard_id: string
      display_name: string
      level: string
    }
  }
} | {
  success: false
  error: string
}

export type AcceptTermsResponse = {
  success: true
  data: {
    accepted: boolean
    acceptedAt: string | null
    allocation: UserAllocation
  }
} | {
  success: false
  error: string
}

export type VerifyHumanResponse = {
  success: true
  message: string
  data: {
    is_human: boolean
    wallet_address: string
    humanProbability: number
    credentialPoints: number
    verifiedAt: string
  }
} | {
  success: false
  error: string
  data?: {
    humanProbability: number
    credentialPoints: number
    threshold: number
  }
}
