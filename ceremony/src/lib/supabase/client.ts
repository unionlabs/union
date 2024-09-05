import { createClient, type SupabaseClient } from "@supabase/supabase-js"

const SUPABASE_ANON_KEY = import.meta.env.VITE_SUPABASE_ANON_KEY
const SUPABASE_URL = import.meta.env.VITE_SUPABASE_URL

export const supabase: SupabaseClient = createClient(SUPABASE_URL, SUPABASE_ANON_KEY)
