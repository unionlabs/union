import type { Database } from "$lib/dashboard/database.types.ts"
import { createClient } from "@supabase/supabase-js"
import { Effect, Option } from "effect"

// Extend the existing database types with our new tables
export interface AllocationDatabase extends Database {
  public: Database["public"] & {
    Tables: Database["public"]["Tables"] & {
      user_allocations: {
        Row: {
          id: string
          user_id: string
          total_allocation: number
          available_allocation: number
          pre_staked_amount: number
          shared_out_amount: number
          received_amount: number
          claimed_at: string | null
          created_at: string
          updated_at: string
        }
        Insert: {
          id?: string
          user_id: string
          total_allocation: number
          available_allocation: number
          pre_staked_amount?: number
          shared_out_amount?: number
          received_amount?: number
          claimed_at?: string | null
          created_at?: string
          updated_at?: string
        }
        Update: {
          id?: string
          user_id?: string
          total_allocation?: number
          available_allocation?: number
          pre_staked_amount?: number
          shared_out_amount?: number
          received_amount?: number
          claimed_at?: string | null
          created_at?: string
          updated_at?: string
        }
      }
      allocation_shares: {
        Row: {
          id: string
          from_user_id: string
          to_user_id: string | null
          amount: number
          share_type: string
          referral_code: string | null
          transaction_hash: string | null
          status: string
          created_at: string
          completed_at: string | null
        }
        Insert: {
          id?: string
          from_user_id: string
          to_user_id?: string | null
          amount: number
          share_type?: string
          referral_code?: string | null
          transaction_hash?: string | null
          status?: string
          created_at?: string
          completed_at?: string | null
        }
        Update: {
          id?: string
          from_user_id?: string
          to_user_id?: string | null
          amount?: number
          share_type?: string
          referral_code?: string | null
          transaction_hash?: string | null
          status?: string
          created_at?: string
          completed_at?: string | null
        }
      }
      pre_stakes: {
        Row: {
          id: string
          user_id: string
          validator_address: string
          validator_name: string | null
          amount: number
          estimated_apy: number | null
          bonus_rate: number | null
          transaction_hash: string | null
          status: string
          created_at: string
          activated_at: string | null
        }
        Insert: {
          id?: string
          user_id: string
          validator_address: string
          validator_name?: string | null
          amount: number
          estimated_apy?: number | null
          bonus_rate?: number | null
          transaction_hash?: string | null
          status?: string
          created_at?: string
          activated_at?: string | null
        }
        Update: {
          id?: string
          user_id?: string
          validator_address?: string
          validator_name?: string | null
          amount?: number
          estimated_apy?: number | null
          bonus_rate?: number | null
          transaction_hash?: string | null
          status?: string
          created_at?: string
          activated_at?: string | null
        }
      }
      referral_codes: {
        Row: {
          id: string
          user_id: string
          code: string
          uses_count: number
          total_rewards_earned: number
          is_active: boolean
          created_at: string
        }
        Insert: {
          id?: string
          user_id: string
          code: string
          uses_count?: number
          total_rewards_earned?: number
          is_active?: boolean
          created_at?: string
        }
        Update: {
          id?: string
          user_id?: string
          code?: string
          uses_count?: number
          total_rewards_earned?: number
          is_active?: boolean
          created_at?: string
        }
      }
      user_social_scores: {
        Row: {
          id: string
          user_id: string
          total_score: number
          shares_given_count: number
          shares_received_count: number
          referrals_count: number
          pre_stake_bonus_score: number
          rank: number | null
          last_calculated_at: string
        }
        Insert: {
          id?: string
          user_id: string
          total_score?: number
          shares_given_count?: number
          shares_received_count?: number
          referrals_count?: number
          pre_stake_bonus_score?: number
          rank?: number | null
          last_calculated_at?: string
        }
        Update: {
          id?: string
          user_id?: string
          total_score?: number
          shares_given_count?: number
          shares_received_count?: number
          referrals_count?: number
          pre_stake_bonus_score?: number
          rank?: number | null
          last_calculated_at?: string
        }
      }
      marketplace_activities: {
        Row: {
          id: string
          user_id: string | null
          activity_type: string
          amount: number | null
          metadata: any | null
          created_at: string
        }
        Insert: {
          id?: string
          user_id?: string | null
          activity_type: string
          amount?: number | null
          metadata?: any | null
          created_at?: string
        }
        Update: {
          id?: string
          user_id?: string | null
          activity_type?: string
          amount?: number | null
          metadata?: any | null
          created_at?: string
        }
      }
    }
  }
}

// Types for our marketplace data
export interface UserAllocation {
  id: string
  user_id: string
  total_allocation: number
  available_allocation: number
  pre_staked_amount: number
  shared_out_amount: number
  received_amount: number
  claimed_at: string | null
  created_at: string
  updated_at: string
  // Joined user data
  username?: string
  avatar_url?: string
  social_score?: number
  social_rank?: number
}

export interface AllocationShare {
  id: string
  from_user_id: string
  to_user_id: string | null
  amount: number
  share_type: string
  referral_code: string | null
  transaction_hash: string | null
  status: string
  created_at: string
  completed_at: string | null
  // Joined user data
  from_username?: string
  from_avatar?: string
  to_username?: string
  to_avatar?: string
}

export interface MarketplaceStats {
  total_shared: number
  active_users: number
  avg_allocation: number
  recent_activity_count: number
}

export interface PreStake {
  id: string
  user_id: string
  validator_address: string
  validator_name: string | null
  amount: number
  estimated_apy: number | null
  bonus_rate: number | null
  transaction_hash: string | null
  status: string
  created_at: string
  activated_at: string | null
}

export interface ReferralCode {
  id: string
  user_id: string
  code: string
  uses_count: number
  total_rewards_earned: number
  is_active: boolean
  created_at: string
}

// Allocation Marketplace Service
export class AllocationMarketplaceService {
  constructor(private supabase: ReturnType<typeof createClient<AllocationDatabase>>) {}

  // Get user's allocation data with social info
  getUserAllocation(userId: string) {
    return Effect.tryPromise({
      try: async () => {
        const { data, error } = await this.supabase
          .from("user_allocations")
          .select(`
            *,
            users!inner(display_name, pfp),
            user_social_scores(total_score, rank)
          `)
          .eq("user_id", userId)
          .single()

        if (error) {
          throw error
        }
        return data as UserAllocation
      },
      catch: (error) => new Error(`Failed to get user allocation: ${error}`),
    })
  }

  // Create or update user allocation
  upsertUserAllocation(userId: string, totalAllocation: number) {
    return Effect.tryPromise({
      try: async () => {
        const { data, error } = await this.supabase
          .from("user_allocations")
          .upsert({
            user_id: userId,
            total_allocation: totalAllocation,
            available_allocation: totalAllocation,
          })
          .select()
          .single()

        if (error) {
          throw error
        }
        return data
      },
      catch: (error) => new Error(`Failed to upsert user allocation: ${error}`),
    })
  }

  // Create allocation share
  createAllocationShare(
    fromUserId: string,
    toUserId: string | null,
    amount: number,
    shareType: string = "direct",
    referralCode?: string,
  ) {
    return Effect.tryPromise({
      try: async () => {
        const { data, error } = await this.supabase
          .from("allocation_shares")
          .insert({
            from_user_id: fromUserId,
            to_user_id: toUserId,
            amount,
            share_type: shareType,
            referral_code: referralCode,
          })
          .select()
          .single()

        if (error) {
          throw error
        }
        return data
      },
      catch: (error) => new Error(`Failed to create allocation share: ${error}`),
    })
  }

  // Complete allocation share
  completeAllocationShare(shareId: string, transactionHash?: string) {
    return Effect.tryPromise({
      try: async () => {
        const { data, error } = await this.supabase
          .from("allocation_shares")
          .update({
            status: "completed",
            completed_at: new Date().toISOString(),
            transaction_hash: transactionHash,
          })
          .eq("id", shareId)
          .eq("status", "pending")
          .select()
          .single()

        if (error) {
          throw error
        }
        return data
      },
      catch: (error) => new Error(`Failed to complete allocation share: ${error}`),
    })
  }

  // Get recent allocation shares for marketplace feed
  getRecentShares(limit: number = 20) {
    return Effect.tryPromise({
      try: async () => {
        const { data, error } = await this.supabase
          .from("allocation_shares")
          .select(`
            *,
            from_user:users!from_user_id(display_name, pfp),
            to_user:users!to_user_id(display_name, pfp)
          `)
          .eq("status", "completed")
          .order("completed_at", { ascending: false })
          .limit(limit)

        if (error) {
          throw error
        }
        return data as AllocationShare[]
      },
      catch: (error) => new Error(`Failed to get recent shares: ${error}`),
    })
  }

  // Get marketplace statistics
  getMarketplaceStats() {
    return Effect.tryPromise({
      try: async () => {
        // Get total shared in last 24 hours
        const { data: sharesData, error: sharesError } = await this.supabase
          .from("allocation_shares")
          .select("amount, from_user_id")
          .eq("status", "completed")
          .gte("completed_at", new Date(Date.now() - 24 * 60 * 60 * 1000).toISOString())

        if (sharesError) {
          throw sharesError
        }

        const totalShared = sharesData?.reduce((sum, share) => sum + share.amount, 0) || 0
        const activeUsers = new Set(sharesData?.map(share => share.from_user_id)).size

        // Get average allocation
        const { data: allocData, error: allocError } = await this.supabase
          .from("user_allocations")
          .select("total_allocation")

        if (allocError) {
          throw allocError
        }

        const avgAllocation = allocData?.length
          ? allocData.reduce((sum, alloc) => sum + alloc.total_allocation, 0) / allocData.length
          : 0

        return {
          total_shared: totalShared,
          active_users: activeUsers,
          avg_allocation: Math.floor(avgAllocation),
          recent_activity_count: sharesData?.length || 0,
        } as MarketplaceStats
      },
      catch: (error) => new Error(`Failed to get marketplace stats: ${error}`),
    })
  }

  // Create pre-stake
  createPreStake(
    userId: string,
    validatorAddress: string,
    validatorName: string,
    amount: number,
    estimatedApy: number,
    bonusRate: number,
  ) {
    return Effect.tryPromise({
      try: async () => {
        const { data, error } = await this.supabase
          .from("pre_stakes")
          .insert({
            user_id: userId,
            validator_address: validatorAddress,
            validator_name: validatorName,
            amount,
            estimated_apy: estimatedApy,
            bonus_rate: bonusRate,
          })
          .select()
          .single()

        if (error) {
          throw error
        }
        return data
      },
      catch: (error) => new Error(`Failed to create pre-stake: ${error}`),
    })
  }

  // Get user's referral code
  getUserReferralCode(userId: string) {
    return Effect.tryPromise({
      try: async () => {
        const { data, error } = await this.supabase
          .from("referral_codes")
          .select("*")
          .eq("user_id", userId)
          .eq("is_active", true)
          .order("created_at", { ascending: false })
          .limit(1)
          .maybeSingle()

        if (error) {
          throw error
        }
        return Option.fromNullable(data)
      },
      catch: (error) => new Error(`Failed to get user referral code: ${error}`),
    })
  }

  // Create referral code
  createReferralCode(userId: string, code: string) {
    return Effect.tryPromise({
      try: async () => {
        const { data, error } = await this.supabase
          .from("referral_codes")
          .insert({
            user_id: userId,
            code,
          })
          .select()
          .single()

        if (error) {
          throw error
        }
        return data
      },
      catch: (error) => new Error(`Failed to create referral code: ${error}`),
    })
  }
}

// Helper function to generate referral codes
export function generateReferralCode(): string {
  const chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"
  let result = "UNION-"
  for (let i = 0; i < 6; i++) {
    result += chars.charAt(Math.floor(Math.random() * chars.length))
  }
  return result
}

// Helper function to format token amounts
export function formatTokenAmount(amount: number): string {
  return new Intl.NumberFormat().format(amount)
}

// Helper function to calculate rewards
export function calculateRewards(amount: number, apy: number, bonusRate: number = 0): {
  baseRewards: number
  bonusRewards: number
  totalRewards: number
} {
  const baseRewards = Math.floor(amount * (apy / 100))
  const bonusRewards = Math.floor(amount * (bonusRate / 100))
  return {
    baseRewards,
    bonusRewards,
    totalRewards: baseRewards + bonusRewards,
  }
}
