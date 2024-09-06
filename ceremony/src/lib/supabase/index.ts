import { user } from "$lib/stores/user.svelte.ts"
import {
  getContribution,
  getContributor,
  getSubmittedContribution,
  getUserQueuePosition
} from "$lib/supabase/queries.ts"
import type { ContributionStatus } from "$lib/supabase/types.ts"

export const getUserQueueInfo = async () => {
  const userId = user.session?.user.id
  if (!userId) {
    throw new Error("User is not logged in")
  }

  const { data, count, error } = await getUserQueuePosition("73073266-b790-4de1-b2e1-3176c20c3f76")

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

export const checkContributionStatus = async (): Promise<ContributionStatus> => {
  const userId = user.session?.user.id
  if (!userId) {
    throw new Error("User ID is required")
  }

  try {
    const [contributor, submittedContribution, verifiedContribution] = await Promise.all([
      getContributor("73073266-b790-4de1-b2e1-3176c20c3f76"),
      getSubmittedContribution("73073266-b790-4de1-b2e1-3176c20c3f76"),
      getContribution("73073266-b790-4de1-b2e1-3176c20c3f76")
    ])

    const isContributor = !!contributor?.data
    const hasSubmitted = !!submittedContribution?.data
    const hasVerified = !!verifiedContribution?.data

    return {
      canContribute: isContributor && !hasSubmitted && !hasVerified,
      shouldContribute: isContributor && !hasSubmitted && !hasVerified,
      isVerifying: hasSubmitted && !hasVerified
    }
  } catch (error) {
    console.error("Error checking contribution status:", error)
    throw new Error("Failed to check contribution status")
  }
}
