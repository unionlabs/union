<script lang="ts">
import { supabase } from "$lib/supabase/client.ts"
import { createState } from "$lib/state/index.svelte.ts"
import { watch } from "runed"
import { checkAuth } from "$lib/state/session.svelte.ts"
import Terminal from "$lib/components/Terminal/index.svelte"
import { start } from "$lib/client"
import Timer from "$lib/components/Terminal/Timer.svelte"

import "../styles/tailwind.css"
import { onMount } from "svelte"

let { children } = $props()

let { user, contributor } = createState()

$effect(() => {
  const {
    data: { subscription }
  } = supabase.auth.onAuthStateChange(() => {
    checkAuth()
  })
  return () => {
    subscription.unsubscribe()
  }
})

$effect(() => {
  if (
    contributor.contributionState === "contribute" &&
    contributor.state !== "contributing" &&
    contributor.downloadedSecret
  ) {
    start()
  }
})

watch(
  () => user.session?.user.id,
  () => {
    contributor.setUserId(user.session?.user.id)
  }
)

$effect(() => {
  if (!showBootSequence) {
    // @ts-ignore
    document.getElementById("glitch-video").play()
  }
})
let showBootSequence = $state(localStorage?.getItem("ceremony:show-boot-sequence") !== "false")
let bootSequenceVideoElement = $state<HTMLVideoElement | null>(null)

onMount(() => bootSequenceVideoElement?.play())

const hideBootSequenceVideo = () => {
  showBootSequence = false
  localStorage?.setItem("ceremony:show-boot-sequence", "false")
}
</script>

{#if showBootSequence}
  <video
    muted
    autoplay
    playsinline
    data-video="bootsequence"
    onended={hideBootSequenceVideo}
    bind:this={bootSequenceVideoElement}
    oncanplay={function() {
      this.autoplay = true
    }}
    onloadeddata={function() {
      this.autoplay = true
    }}
    onloadedmetadata={function() {
      this.muted = true
    }}
  >
    <source src="https://pub-32dd1494f0fa423cb1013941269ecce9.r2.dev/glitchboot.webm" type="video/webm" />
  </video>
{:else}
  <video
    id="glitch-video"
    loop
    muted
    autoplay
    playsinline
    data-video="glitch"
    oncanplay={function() {
      this.autoplay = true
    }}
    onloadeddata={function() {
      this.autoplay = true
    }}
    onloadedmetadata={function() {
      this.muted = true
    }}
  >
    <source src="https://pub-32dd1494f0fa423cb1013941269ecce9.r2.dev/glitch.webm" type="video/webm" />
  </video>
  <main class="flex w-full h-full overflow-hidden content flex-col items-center justify-center gap-4">
    <Terminal>
      {@render children()}
    </Terminal>
    <Timer />
  </main>
{/if}

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
