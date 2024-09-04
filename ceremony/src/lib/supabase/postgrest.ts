import { PostgrestClient } from '@supabase/postgrest-js'

const SUPABASE_URL = import.meta.env.VITE_SUPABASE_URL;
const SUPABASE_ANON_KEY = import.meta.env.VITE_SUPABASE_ANON_KEY;

const postgrest = new PostgrestClient(SUPABASE_URL, {
  headers: {
    apikey: SUPABASE_ANON_KEY,
  },
})