<script lang="ts">
import { supabase } from "$lib/supabase/client.ts"
import { createState } from "$lib/state/index.svelte.ts"
import { watch } from "runed"
import { checkAuth } from "$lib/state/session.svelte.ts"
import Terminal from "$lib/components/Terminal/index.svelte"
import { start } from "$lib/client"

import "../styles/tailwind.css"
import Print from "$lib/components/Terminal/Print.svelte";

let { children } = $props()

let { user, contributor, terminal } = createState()

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

<main class="flex flex-col w-full h-full items-center justify-center">
  <Terminal>
    {@render children()}
  </Terminal>
  <Print>1:00:00</Print>
</main>
