import { user } from "$lib/stores/user.svelte.ts"
import {
  getContribution,
  getContributor,
  getQueuePositionAndLength,
  getSubmittedContribution
} from "$lib/supabase/queries.ts"

export const checkQueue = async (): Promise<{ position: number; total: number } | null> => {
  const userId = user.session?.user.id
  if (!userId) {
    throw new Error("User is not logged in")
  }

  const { data, error, count } = await getQueuePositionAndLength()

  if (error) {
    console.error("Error fetching queue:", error)
    return null
  }

  if (!(data && Array.isArray(data))) {
    console.error("Unexpected data format from getQueuePosition")
    return null
  }
  const position = data.findIndex(row => row.id === userId)
  const userPosition = position !== -1 ? position + 1 : -1

  return {
    position: userPosition,
    total: count ?? 0
  }
}

export const checkContribution = async (): Promise<{
  status: string
  shouldContribute: boolean
}> => {
  const userId = user.session?.user.id
  if (!userId) {
    throw new Error("User is not logged in")
  }

  const { data: currentContributorData, error: currentContributorError } =
    await getContributor(userId)

  if (currentContributorError) {
    console.error("Error checking current contributor:", currentContributorError)
    return { status: "error", shouldContribute: false }
  }

  if (currentContributorData) {
    const { data: submittedContribution, error: submittedError } =
      await getSubmittedContribution(userId)

    if (submittedError) {
      console.error("Error checking submitted contribution:", submittedError)
      return { status: "error", shouldContribute: false }
    }

    if (!submittedContribution) {
      return { status: "contribute", shouldContribute: true }
    }

    const { data: contributionData, error: contributionError } = await getContribution(userId)

    if (contributionError) {
      console.error("Error checking contribution:", contributionError)
      return { status: "error", shouldContribute: false }
    }

    if (!contributionData) {
      return { status: "verifying", shouldContribute: false }
    }
  }

  const { data: contribution, error: finalContributionError } = await getContribution(userId)

  if (finalContributionError) {
    console.error("Error in final contribution check:", finalContributionError)
    return { status: "error", shouldContribute: false }
  }

  return {
    status: contribution ? "contributed" : "noContribution",
    shouldContribute: !contribution
  }
}
