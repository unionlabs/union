<script lang="ts">
  import Glitch from "$lib/components/Glitch.svelte";
  import {fade} from "svelte/transition";

  let isPlaying = $state(false);
  let isLoading = $state(false);
  let video: HTMLVideoElement;

  function startVideo() {
    isLoading = true;
    isPlaying = true;

    setTimeout(() => {
      if (video) {
        video.muted = false;
        video.play()
          .then(() => {
            isLoading = false;
          })
          .catch((error) => {
            console.error('Error playing video:', error);
            isLoading = false;
            isPlaying = false;
          });
      }
    }, 0);
  }
</script>

{#if !isPlaying}
  <div class="h-svh w-full flex items-center justify-center">
    {#if isLoading}
      <svg class="animate-spin size-24 text-union-accent-500" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
        <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
        <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
      </svg>
    {:else}
      <button class="text-union-accent-500" onclick={startVideo} aria-label="Play video">
        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="size-24">
          <path stroke-linecap="round" stroke-linejoin="round" d="M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0Z" />
          <path stroke-linecap="round" stroke-linejoin="round" d="M15.91 11.672a.375.375 0 0 1 0 .656l-5.603 3.113a.375.375 0 0 1-.557-.328V8.887c0-.286.307-.466.557-.327l5.603 3.112Z" />
        </svg>
      </button>
    {/if}
  </div>
{:else}
  <video
          bind:this={video}
          id="glitch-video"
          loop
          playsinline
          data-video="glitch"
  >
    <track kind="captions" src="dsa">
    <source src="https://pub-32dd1494f0fa423cb1013941269ecce9.r2.dev/zkgm-v1.mp4" type="video/webm"/>
  </video>
  <main class="h-svh w-full flex justify-center items-center" in:fade>
    <Glitch text="ZKGM"/>
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