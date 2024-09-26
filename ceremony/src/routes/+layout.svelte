<script lang="ts">
import { supabase } from "$lib/supabase/client.ts"
import { createState } from "$lib/state/index.svelte.ts"
import { watch } from "runed"
import { checkAuth } from "$lib/state/session.svelte.ts"
import Terminal from "$lib/components/Terminal/index.svelte"
import { start } from "$lib/client"
import Timer from "$lib/components/Terminal/Timer.svelte"

import "../styles/tailwind.css"
import {onMount} from "svelte";
import {getAverageTimes} from "$lib/supabase";

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

<video autoplay muted loop data-video="">
  <source src="https://pub-32dd1494f0fa423cb1013941269ecce9.r2.dev/glitch.mov" type="video/mp4" />
</video>

<main class="flex w-full h-full overflow-hidden content flex-col items-center justify-center gap-4">
  <Terminal>
    {@render children()}
  </Terminal>
  <Timer />
</main>


<style lang="postcss">
video[data-video] {
  right: 0;
  bottom: 0;
  z-index: -1;
  width: 100vw;
  height: 100vh;
  min-width: 100%;
  position: fixed;
  min-height: 100%;
  object-fit: cover;
}
</style>
