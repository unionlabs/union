<script lang="ts">
import { supabase } from "$lib/supabase/client.ts"
import { createState } from "$lib/state/index.svelte.ts"

import "../styles/tailwind.css"
import { watch } from "runed"
import { checkAuth } from "$lib/state/session.svelte.ts"
import Terminal from "$lib/components/Terminal/index.svelte"
import { start } from "$lib/client"

let { children } = $props()

let { user, contributor } = createState()

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

$effect(() => {
  if (contributor.contributionState === "contribute" && contributor.state !== "contributing") {
    start()
  }
})

watch(
  () => user.session?.user.id,
  () => {
    contributor.setUserId(user.session?.user.id)
  }
)
</script>

<main class="flex w-full h-full overflow-hidden">
  <Terminal>
    {@render children()}
  </Terminal>
</main>
