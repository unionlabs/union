<script lang="ts">
import Glitch from "$lib/components/Glitch.svelte"
import { fade } from "svelte/transition"
import Agents from "$lib/components/Agents.svelte"
import Bar from "$lib/components/Bar.svelte"

let isPlaying: boolean = $state(false)
let video: HTMLVideoElement | null = $state(null)
let overlay: boolean = $state(true)

function handleTouch() {
  if (!isPlaying) {
    startVideo()
  }
}

async function startVideo() {
  if (video) {
    // Reset video state
    video.muted = true
    video.currentTime = 0
    video.playsInline = true
    video.load()

    try {
      await video.play()
      setTimeout(() => {
        if (video) {
          video.muted = false
          isPlaying = true
          overlay = false
        }
      }, 100)
    } catch (error) {
      console.error("Error playing video:", error)
      isPlaying = false
      overlay = true
    }
  }
}
</script>

<div
        role="presentation"
        ontouchstart={handleTouch}
        class="w-full h-full"
>
  <video
          bind:this={video}
          id="glitch-video"
          loop
          playsinline
          muted
          preload="auto"
          class="fixed inset-0 w-full h-full object-cover -z-10"
          data-video="glitch"
  >
    <source
            src="https://pub-32dd1494f0fa423cb1013941269ecce9.r2.dev/zkgm-v1.mp4"
            type="video/mp4"
    />
  </video>

  {#if overlay}
    <div class="fixed inset-0 bg-black flex items-center justify-center">
      <button
              class="text-union-accent-500"
              onclick={startVideo}
              aria-label="Play video"
      >
        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="size-24">
          <path stroke-linecap="round" stroke-linejoin="round" d="M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0Z" />
          <path stroke-linecap="round" stroke-linejoin="round" d="M15.91 11.672a.375.375 0 0 1 0 .656l-5.603 3.113a.375.375 0 0 1-.557-.328V8.887c0-.286.307-.466.557-.327l5.603 3.112Z" />
        </svg>
      </button>
    </div>
  {/if}

  {#if isPlaying}
    <div class="h-svh w-full flex flex-col justify-between items-center relative" in:fade>
      <Bar/>
      <div class="flex-grow flex items-center">
        <Glitch text="ZKGM"/>
      </div>
      <div class="h-24 w-full flex-shrink-0">
        <Agents/>
      </div>
    </div>
  {/if}
</div>