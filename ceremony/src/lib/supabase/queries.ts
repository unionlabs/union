import { supabase } from "$lib/supabase/client.ts"

export const getContributor = async (userId: string) => {
  const { data, error } = await supabase
    .from("current_contributor_id")
    .select("id")
    .eq("id", userId)
    .maybeSingle()

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

export const getQueuePosition = async () => {
  const { data, error } = await supabase
    .from("queue")
    .select("id, joined")
    .order("joined", { ascending: true })

  if (error) console.error("Error in getQueuePosition:", error)
  return { data, error }
}
