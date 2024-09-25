<script lang="ts">
import { supabase } from "$lib/supabase/client.ts"
import { Toaster } from "svelte-sonner"
import { createState } from "$lib/state/index.svelte.ts"

import "../styles/tailwind.css"

let { children } = $props()

let { user } = createState()

$effect(() => {
  const {
    data: { subscription }
  } = supabase.auth.onAuthStateChange((event, session) => {
    user.session = session
  })
  return () => {
    subscription.unsubscribe()
  }
})
</script>



<Toaster position="bottom-right" toastOptions={{ class: 'rounded-none border border-black',}}/>

<main class="w-full h-full overflow-y-scroll">
  {@render children()}
</main>