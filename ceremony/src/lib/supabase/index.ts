import { user } from "$lib/state/session.svelte.ts"
import {
  queryContribution,
  queryContributor,
  queryQueueCount,
  querySubmittedContribution,
  queryUserQueuePosition,
  queryContributions,
  queryContributionTime,
  queryCurrentUserState,
  queryUserContribution,
  queryUserPublicHash,
  queryUserWallet,
  queryVerificationTime,
  queryContributionWindow
} from "$lib/supabase/queries.ts"
import { supabase } from "$lib/supabase/client.ts"
import { msToTimeString, sleep, timeToMs } from "$lib/utils/utils.ts"
import type { AllowanceState, ContributionState } from "$lib/state/contributor.svelte.ts"

export interface TimeResult {
  verification: string | null
  contribution: string | null
  total: string
  verificationMs: number
  contributionMs: number
  totalMs: number
}

export const callJoinQueue = async (code: string | null): Promise<boolean> => {
  if (!user.session) {
    throw new Error("User is not logged in")
  }
  const userId = user.session.user.id
  if (!userId) {
    throw new Error("User is not logged in")
  }

  try {
    const { error } = await supabase.rpc("join_queue", { code_id: code })

    if (error) {
      console.log("Error joining queue:", error)
      return false
    }

    return true
  } catch (err) {
    console.log("Unexpected error:", err)
    return false
  }
}

export const checkIfOpen = async (): Promise<boolean> => {
  const { data, error } = await supabase.rpc("open_to_public")
  return data
}

export const getUserQueueInfo = async () => {
  if (!user.session) {
    throw new Error("User is not logged in")
  }
  const userId = user.session.user.id

  const { data, error } = await queryUserQueuePosition(userId)
  const { count, error: countError } = await queryQueueCount()

  if (error) {
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
  if (!user.session) {
    throw new Error("User is not logged in")
  }
  const userId = user.session.user.id
  if (!userId) {
    throw new Error("User ID is required")
  }

  try {
    const [contributor, submittedContribution, verifiedContribution, contributionWindow] =
      await Promise.all([
        queryContributor(userId),
        querySubmittedContribution(userId),
        queryContribution(userId),
        queryContributionWindow(userId)
      ])

    const isContributor = !!contributor?.data
    const hasSubmitted = !!submittedContribution?.data
    const hasVerified = !!verifiedContribution?.data
    const isExpired = contributionWindow?.data?.expire
      ? Date.now() > new Date(contributionWindow.data.expire).getTime()
      : false

    let status: ContributionState

    if (isContributor && !hasSubmitted && !hasVerified) {
      status = "contribute"
    } else if (isContributor && hasSubmitted && !hasVerified) {
      status = "verifying"
    } else if (hasVerified) {
      status = "contributed"
    } else if (!hasSubmitted && isExpired) {
      status = "missed"
    } else {
      status = "notContributed"
    }

    return status
  } catch (error) {
    console.log("Error checking contribution status:", error)
    throw new Error("Failed to check contribution status")
  }
}

export const getCurrentUserState = async (userId: string | undefined): Promise<AllowanceState> => {
  if (!userId) {
    console.log("Need to be logged in to get allowance state")
    return undefined
  }

  const { data, error } = await queryCurrentUserState()
  if (error || !data) return undefined

  if (data.in_queue) return "inQueue"
  if (data.has_redeemed) return "hasRedeemed"
  if (data.in_waitlist) return "inWaitlist"

  return "join"
}

export const getContributions = async () => {
  const { data, error } = await queryContributions()
  if (error || !data) return undefined

  return data
}

export const getUserContribution = async (hash: string) => {
  await sleep(500)
  const { data, error } = await queryUserContribution(hash)
  if (error || !data) return undefined

  return data
}

interface WalletData {
  id: string
  wallet: string
}

export const insertWalletData = async (data: WalletData) => {

  const { data: existingData, error: fetchError } = await supabase
    .from('wallet_address')
    .select()
    .eq('id', data.id);

  if (fetchError) {
    console.error('Error fetching data:', fetchError);
    return null;
  }

  if (existingData && existingData.length > 0) {
    const { data: updatedData, error: updateError } = await supabase
      .from('wallet_address')
      .update({
        wallet: data.wallet,
      })
      .eq('id', data.id)
      .select()


    if (updateError) {
      console.error('Error updating data:', updateError);
      return null;
    }

    return updatedData;
  } else {
    console.log('INSERTING USER ADDRESS')
    const { data: insertedData, error: insertError } = await supabase
      .from('wallet_address')
      .insert({
        id: data.id,
        wallet: data.wallet,
      })
      .select();

    if (insertError) {
      console.error('Error inserting data:', insertError);
      return null;
    }

    return insertedData;
  }
};



export const getPublicHash = async () => {
  if (!user.session) {
    throw new Error("User is not logged in")
  }
  const userId = user.session.user.id

  const { data, error } = await queryUserPublicHash(userId)
  if (error || !data) return undefined

  return data.public_key_hash
}

export const getUserWallet = async (userId: string): Promise<string | undefined> => {
  if (!userId) {
    console.log("Need to be logged in to get allowance state")
    return undefined
  }

  const { data, error } = await queryUserWallet(userId)
  if (error || !data) return undefined

  return data.wallet
}

export const getAverageTimes = async (): Promise<TimeResult> => {
  let contributionResult: { data: unknown; error: unknown | null }
  let verificationResult: { data: unknown; error: unknown | null }

  try {
    ;[contributionResult, verificationResult] = await Promise.all([
      queryContributionTime(),
      queryVerificationTime()
    ])
  } catch (error) {
    console.error("Error fetching times:", error)
    contributionResult = { data: null, error: null }
    verificationResult = { data: null, error: null }
  }

  // @ts-ignore
  const contribution = contributionResult.data?.contribution_average ?? null
  // @ts-ignore
  const verification = verificationResult.data?.verification_average ?? null

  const contributionMs = timeToMs(contribution)
  const verificationMs = timeToMs(verification)
  const totalMs = contributionMs + verificationMs

  return {
    verification,
    contribution,
    total: msToTimeString(totalMs),
    verificationMs,
    contributionMs,
    totalMs
  }
}

export const rejoin = async () => {
  if (!user.session) {
    throw new Error("User is not logged in")
  }

  try {
    const { error } = await supabase.rpc("rejoin_queue")

    if (error) {
      console.log("Error joining queue:", error)
      return false
    }

    return true
  } catch (err) {
    console.log("Unexpected error:", err)
    return false
  }
}
