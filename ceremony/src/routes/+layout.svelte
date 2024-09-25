<script lang="ts">
import { supabase } from "$lib/supabase/client.ts"
import { createState } from "$lib/state/index.svelte.ts"
import TerminalWindow from "$lib/components/TerminalApp/TerminalWindow.svelte";

import "../styles/tailwind.css"

let { children } = $props()

let { user, contributor} = createState()

$effect(() => {
  const {
    data: { subscription }
  } = supabase.auth.onAuthStateChange((event, session) => {
    user.session = session
    user.loading = false
    contributor.setUserId(user.session?.user.id)
  })
  return () => {
    subscription.unsubscribe()
  }
})

</script>

<main class="w-full h-full overflow-y-scroll">
  <TerminalWindow>
    {@render children()}
  </TerminalWindow>
</main>