<script lang="ts">
import "../styles/tailwind.css"
import { supabase } from "$lib/supabase/client.ts"
import { createState } from "$lib/state/index.svelte.ts"
import { watch } from "runed"
import { checkAuth } from "$lib/state/session.svelte.ts"
import Terminal from "$lib/components/Terminal/index.svelte"
import { start } from "$lib/client"
import Timer from "$lib/components/Terminal/Timer.svelte"
import { onMount } from "svelte"
import { getAverageTimes } from "$lib/supabase"

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

let enabledVideo = $state<"INTRO" | "MAIN" | "OUTRO">("INTRO")

let introVideoElement = $state<HTMLVideoElement | null>(null)

onMount(() => introVideoElement?.play())

const hideIntroVideo = () => (enabledVideo = "MAIN")

let outroVideoElement = $state<HTMLVideoElement | null>(null)

let deleteOutroVideo = () => (outroVideoElement = null)
</script>

{#if enabledVideo === 'INTRO'}
  <video
    muted
    autoplay
    playsinline
    data-video="intro"
    onended={hideIntroVideo}
    bind:this={introVideoElement}
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
{:else if contributor.clientState !== 'successful'}
  <video
    loop
    muted
    autoplay
    playsinline
    data-video="main"
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
{:else}
  {enabledVideo = 'OUTRO'}
  <video
    muted
    autoplay
    playsinline
    data-video="outro"
    onended={deleteOutroVideo}
    bind:this={outroVideoElement}
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
