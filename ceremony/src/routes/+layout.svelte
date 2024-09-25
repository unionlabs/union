<script lang="ts">
import { supabase } from "$lib/supabase/client.ts"
import { createState } from "$lib/state/index.svelte.ts"
import TerminalWindow from "$lib/components/TerminalApp/TerminalWindow.svelte";

import "../styles/tailwind.css"
import {watch} from "runed";
import {checkAuth} from "$lib/state/session.svelte.ts";

let { children } = $props()

let { user, contributor} = createState()

$effect(() => {
  const {
    data: { subscription }
  } = supabase.auth.onAuthStateChange((event, session) => {
    checkAuth()
  })
  return () => {
    subscription.unsubscribe()
  }
})

watch(() => user.session?.user.id, () => {
  contributor.setUserId(user.session?.user.id)
})

</script>

<main class="w-full h-full overflow-y-scroll">
  <TerminalWindow>
    {@render children()}
  </TerminalWindow>
</main>