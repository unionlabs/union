import { user } from "$lib/stores/user.svelte.ts"
import {
  getContribution,
  getContributor,
  getQueueCount,
  getSubmittedContribution,
  getUserQueuePosition,
  queryAllowance,
  queryContributions,
  queryUserContribution,
  queryUserPublicHash,
  queryUserWallet
} from "$lib/supabase/queries.ts"
import { supabase } from "$lib/supabase/client.ts"
import type { AllowanceState, ContributionState } from "$lib/stores/state.svelte.ts"

export const callJoinQueue = async (code: string | null): Promise<boolean> => {
  const userId = user.session?.user.id
  if (!userId) {
    throw new Error("User is not logged in")
  }

  try {
    const { error } = await supabase.rpc("join_queue", { code_id: code })

    if (error) {
      console.error("Error joining queue:", error)
      return false
    }

    return true
  } catch (err) {
    console.error("Unexpected error:", err)
    return false
  }
}

export const checkIfOpen = async (): Promise<boolean> => {
  const { data, error } = await supabase.rpc("open_to_public")
  console.log("isOpen:", data)
  return data
}

export const getUserQueueInfo = async () => {
  const userId = user.session?.user.id
  if (!userId) {
    throw new Error("User is not logged in")
  }

  const { data, error } = await getUserQueuePosition(userId)
  const { count, error: countError } = await getQueueCount()

  if (error) {
    console.error("Error getting user queue position:", error)
    return { error }
  }

  if (!data) {
    return {
      inQueue: false,
      message: "User not found in the queue"
    }
  }

  return {
    inQueue: true,
    count: count,
    ...data
  }
}

export const getContributionState = async (): Promise<ContributionState> => {
  const userId = user.session?.user.id
  if (!userId) {
    throw new Error("User ID is required")
  }

  try {
    const [contributor, submittedContribution, verifiedContribution] = await Promise.all([
      getContributor(userId),
      getSubmittedContribution(userId),
      getContribution(userId)
    ])

    const isContributor = !!contributor?.data
    const hasSubmitted = !!submittedContribution?.data
    const hasVerified = !!verifiedContribution?.data

    let status: ContributionState

    if (isContributor && !hasSubmitted && !hasVerified) {
      status = "contribute"
    } else if (isContributor && hasSubmitted && !hasVerified) {
      status = "verifying"
    } else if (hasVerified) {
      status = "contributed"
    } else {
      status = "notContributed"
    }

    return status
  } catch (error) {
    console.log("Error checking contribution status:", error)
    throw new Error("Failed to check contribution status")
  }
}

export const getAllowanceState = async (userId: string | undefined): Promise<AllowanceState> => {
  if (!userId) {
    console.log("Need to be logged in to get allowance state")
    return undefined
  }

  const { data, error } = await queryAllowance()
  if (error || !data) return undefined

  if (data.has_redeemed) return "hasRedeemed"
  if (data.in_queue) return "inQueue"
  if (data.in_waitlist) return "inWaitlist"

  return "join"
}

export const getContributions = async () => {
  const { data, error } = await queryContributions()
  if (error || !data) return undefined

  return data
}

export const getUserContribution = async (hash: string) => {
  console.log(hash)

  const { data, error } = await queryUserContribution(hash)
  if (error || !data) return undefined

  return data
}

interface WalletData {
  id: string
  wallet: string
}

export const insertWalletData = async (data: WalletData) => {
  const { data: insertedData, error } = await supabase
    .from("wallet_address")
    .insert([
      {
        id: data.id,
        wallet: data.wallet
      }
    ])
    .select()

  if (error) {
    console.error("Error inserting data:", error)
    return null
  }

  return insertedData
}

export const getPublicHash = async () => {
  const userId = user.session?.user.id
  if (!userId) {
    throw new Error("User ID is required")
  }

  const { data, error } = await queryUserPublicHash(userId)
  if (error || !data) return undefined

  return data.public_key_hash
}

export const getUserWallet = async (userId: string | undefined) => {
  if (!userId) {
    console.log("Need to be logged in to get allowance state")
    return undefined
  }

  const { data, error } = await queryUserWallet(userId)
  if (error || !data) return undefined

  return data.wallet
}
