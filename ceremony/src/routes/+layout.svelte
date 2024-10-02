<script lang="ts">
import { supabase } from "$lib/supabase/client.ts"
import { createState } from "$lib/state/index.svelte.ts"
import { watch } from "runed"
import { checkAuth } from "$lib/state/session.svelte.ts"
import Terminal from "$lib/components/Terminal/index.svelte"
import { start } from "$lib/client"
import Timer from "$lib/components/Terminal/Timer.svelte"
import { onMount } from "svelte"
import { axiom } from "$lib/utils/axiom.ts"
import {
  generateUserErrorMessage,
  sendWindowErrorLog,
  sendWindowRejectionLog
} from "$lib/utils/error.ts"

import "../styles/tailwind.css"

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
    axiom.ingest("monitor", [{ user: user.session?.user.id, type: "start_contribution" }])
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

onMount(() => {
  bootSequenceVideoElement?.play()
})

const hideBootSequenceVideo = () => {
  showBootSequence = false
  localStorage?.setItem("ceremony:show-boot-sequence", "false")
}

const handleRejection = async (e: PromiseRejectionEvent) => {
  const errorId = await sendWindowRejectionLog(e)
  console.error(generateUserErrorMessage(errorId))
}

const handleError = async (e: Event) => {
  const errorId = await sendWindowErrorLog(e)
  console.error(generateUserErrorMessage(errorId))
}

const imagePath = "https://ceremony.union.build/images/ceremony-og.png"
</script>

<svelte:window on:error={handleError} on:unhandledrejection={handleRejection} />

<svelte:head>
  <title>Union Ceremony</title>
  <meta name="description" content="Ceremony to generate trustworthy cryptographic keys for securing the Union zero-knowledge system."/>

  <meta property="og:title" content="Union Ceremony"/>
  <meta property="og:description" content="Ceremony to generate trustworthy cryptographic keys for securing the Union zero-knowledge system."/>
  <meta property="og:type" content="website"/>
  <meta property="og:url" content="https://ceremony.union.build"/>
  <meta property="og:site_name" content="Union Ceremony"/>
  <meta property="og:locale" content="en_US"/>
  <meta property="og:image" content="https://ceremony.union.build/images/ceremony-og.png"/>
  <meta property="og:image:secure_url" content="https://ceremony.union.build/images/ceremony-og.png"/>
  <meta property="og:image:type" content="image/png"/>
  <meta property="og:image:width" content="1200"/>
  <meta property="og:image:height" content="630"/>
  <meta property="og:image:alt" content="Union Ceremony event banner"/>

  <meta name="twitter:title" content="Union Ceremony"/>
  <meta name="twitter:description" content="Ceremony to generate trustworthy cryptographic keys for securing the Union zero-knowledge system."/>
  <meta name="twitter:card" content="summary_large_image"/>
  <meta name="twitter:site" content="@union_build"/>
  <meta name="twitter:creator" content="@union_build"/>
  <meta name="twitter:image" content="https://ceremony.union.build/images/ceremony-og.png"/>
  <meta name="twitter:image:alt" content="Union Ceremony event banner"/>
  <meta name="twitter:image:width" content="1200"/>
  <meta name="twitter:image:height" content="630"/>

  <link rel="canonical" href="https://ceremony.union.build"/>
  <link rel="icon" href="/favicon.ico" type="image/x-icon"/>
</svelte:head>

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
