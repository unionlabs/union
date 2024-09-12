import { supabase } from "$lib/supabase/client.ts"

export const getContributor = async (userId: string) => {
  const { data, error } = await supabase
    .from("current_contributor_id")
    .select("id")
    .eq("id", userId)
    .single()

  if (error) console.error("Error in getContributor:", error)
  return { data, error }
}

export const getSubmittedContribution = async (userId: string) => {
  const { data, error } = await supabase
    .from("contribution_submitted")
    .select("id")
    .eq("id", userId)
    .maybeSingle()

  if (error) console.error("Error in getSubmittedContribution:", error)
  return { data, error }
}

export const getContribution = async (userId: string) => {
  const { data, error } = await supabase
    .from("contribution")
    .select("id")
    .eq("id", userId)
    .maybeSingle()

  if (error) console.error("Error in getContribution:", error)
  return { data, error }
}

export const getUserQueuePosition = async (userId: string) => {
  const { data, error } = await supabase.from("current_queue").select("*").eq("id", userId).single()

  if (error) {
    if (error.code === "PGRST116") {
      return { data: undefined, error: undefined }
    }
    console.error("Error getting user data:", error)
    return { data: undefined, error }
  }

  return { data, error: undefined }
}

export const getQueueCount = async () => {
  const { count, error } = await supabase
    .from("current_queue")
    .select("*", { count: "exact", head: true })

  if (error) {
    console.error("Error getting total count:", error)
    return { count: undefined, error }
  }

  return { count, error: undefined }
}

export const getQueuePayloadId = async (userId: string) => {
  const { data, error } = await supabase
    .from("queue")
    .select("payload_id")
    .eq("id", userId)
    .single()

  if (error) console.error("Error in getQueuePayloadId:", error)
  return { data, error }
}
