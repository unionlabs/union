import { user } from "$lib/stores/user.svelte.ts"
import {
  getContribution,
  getContributor,
  getQueueCount,
  getSubmittedContribution,
  getUserQueuePosition,
  queryAllowance
} from "$lib/supabase/queries.ts"
import { supabase } from "$lib/supabase/client.ts"
import type { AllowanceState, ContributionState } from "$lib/stores/state.svelte.ts"

export const callJoinQueue = async (codeId: string): Promise<boolean> => {
  const userId = user.session?.user.id
  if (!userId) {
    throw new Error("User is not logged in")
  }

  try {
    const { data, error } = await supabase.rpc("join_queue", { code_id: codeId })

    if (error) {
      console.error("Error calling join_queue:", error)
      return false
    }

    if (!data) {
      console.error("No data returned from join_queue")
      return false
    }

    console.log("Successfully joined queue:", data)
    return true
  } catch (err) {
    console.error("Unexpected error:", err)
    return false // Ensure false is returned on error
  }
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
    return
  }

  const { data, error } = await queryAllowance()
  if (error || !data) return undefined

  if (data.in_waitlist) return "waitingList"
  if (data.has_redeemed) return "invited"

  return undefined
}
