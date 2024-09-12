import type { Session } from "@supabase/supabase-js"

export const user: { session: Session | null } = $state({ session: null })
