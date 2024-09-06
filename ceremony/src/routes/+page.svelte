<script lang="ts">
import H1 from "$lib/components/typography/H1.svelte"
import { onMount } from "svelte"
//
// let imageVisible = false
// onMount(() => {
//   imageVisible = true
// })
</script>

<section class="relative w-full h-full flex items-center justify-center overflow-hidden">
  <div class="max-w-7xl sm:px-6 lg:px-8">
    <div class="max-w-7xl sm:px-6 lg:px-8 fade-in-text">
      <H1 class="text-center">
        Welcome to<br>Union Ceremony
      </H1>
    </div>
  </div>
<!--  <img-->
<!--          src="/images/whale.png"-->
<!--          alt=""-->
<!--          class="absolute -z-10 w-full h-full object-cover deep-sea-rise"-->
<!--          class:visible={imageVisible}/>-->
</section>

<style>
    .deep-sea-rise {
        opacity: 0;
        transform: scale(0.5) translateY(140px);
        filter: brightness(0.1);
        animation: riseFromDepth 2.5s ease-out forwards;
    }

    .deep-sea-rise.visible {
        opacity: 1;
    }

    @keyframes riseFromDepth {
        0% {
            opacity: 0;
            transform: scale(0.5) translateY(140px);
            filter: brightness(0.1);
        }

        100% {
            opacity: 1;
            transform: scale(1) translateY(0);
            filter: brightness(1);
        }
    }

    .fade-in-text {
        opacity: 0;
        animation: fadeInText 1.5s ease-out forwards;
        animation-delay: 2s;
    }

    @keyframes fadeInText {
        0% {
            opacity: 0;
        }
        100% {
            opacity: 1;
        }
    }
</style>
