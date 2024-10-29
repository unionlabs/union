import { supabase } from "$lib/supabase/client.ts"

export const queryContributor = async (userId: string) => {
  const { data, error } = await supabase
    .from("current_contributor_id")
    .select("id")
    .eq("id", userId)
    .single()
  return { data, error }
}

export const querySubmittedContribution = async (userId: string) => {
  const { data, error } = await supabase
    .from("contribution_submitted")
    .select("id")
    .eq("id", userId)
    .single()
  return { data, error }
}

export const queryContribution = async (userId: string) => {
  const { data, error } = await supabase.from("contribution").select("id").eq("id", userId).single()
  return { data, error }
}

export const queryContributionWindow = async (userId: string) => {
  const { data, error } = await supabase
    .from("contribution_status")
    .select("started, expire")
    .eq("id", userId)
    .maybeSingle()
  return { data, error }
}

export const queryUserQueuePosition = async (userId: string) => {
  const { data, error } = await supabase.from("current_queue").select("*").eq("id", userId).single()

  return { data, error }
}

export const queryQueueCount = async () => {
  const { count, error } = await supabase
    .from("current_queue")
    .select("*", { count: "exact", head: true })

  return { count, error }
}

export const queryQueuePayloadId = async (userId: string) => {
  const { data, error } = await supabase
    .from("queue")
    .select("payload_id")
    .eq("id", userId)
    .single()
  return { data, error }
}

export const queryCurrentUserState = async () => {
  const { data, error } = await supabase
    .from("current_user_state")
    .select("has_redeemed, in_queue, waitlist_position")
    .single()

  return { data, error }
}

export const queryContributions = async () => {
  const { data, error } = await supabase
    .from("users_contribution")
    .select("public_key_hash, payload_id")
    .order("time_verified", { ascending: false })

  return { data, error }
}

export const queryUserContribution = async (hash: string) => {
  const { data, error } = await supabase
    .from("users_contribution")
    .select("*")
    .eq("public_key_hash", hash)
    .single()

  return { data, error }
}

export const queryUserPublicHash = async (id: string) => {
  const { data, error } = await supabase
    .from("contribution_signature")
    .select("public_key_hash")
    .eq("id", id)
    .single()

  return { data, error }
}

export const queryUserWallet = async (id: string) => {
  const { data, error } = await supabase
    .from("wallet_address")
    .select("wallet")
    .eq("id", id)
    .single()

  return { data, error }
}

export const queryVerificationTime = async () => {
  const { data, error } = await supabase.from("current_verification_average").select("*").single()

  return { data, error }
}

export const queryContributionTime = async () => {
  const { data, error } = await supabase.from("current_contribution_average").select("*").single()

  return { data, error }
}
