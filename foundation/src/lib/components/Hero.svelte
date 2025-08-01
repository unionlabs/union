<script lang="ts">
import FlickeringGrid from "./FlickeringGrid.svelte"
import Lights from "./Lights.svelte"
let useFlickeringGrid = true // Set to true to test FlickeringGrid

// Responsive grid settings based on screen size
let innerWidth = 0
let gridSettings = $derived.by(() => {
  if (innerWidth < 768) {
    // Mobile
    return { squareSize: 7, gridGap: 8, flickerChance: 0.08 }
  } else if (innerWidth < 1024) {
    // Tablet
    return { squareSize: 8, gridGap: 10, flickerChance: 0.09 }
  } else {
    // Desktop
    return { squareSize: 9, gridGap: 12, flickerChance: 0.1 }
  }
})
</script>

<svelte:window bind:innerWidth />

<div class="bg-black w-full h-svh block overflow-hidden relative">
  <FlickeringGrid
    class="z-0 absolute inset-0 size-full"
    squareSize={gridSettings.squareSize}
    gridGap={gridSettings.gridGap}
    color="#9CA3AF"
    maxOpacity={0.6}
    flickerChance={gridSettings.flickerChance}
    fadeTop={true}
  />

  <div class="absolute bottom-0 left-0 w-full h-full z-10">
    <Lights />
  </div>

  <div class="w-full h-full relative z-20 py-8 sm:py-16">
    <div class="container relative h-full w-full max-w-7xl">
      <div class="flex h-full w-full flex-col justify-end items-start">
        <!-- Header Section -->
        <div class="flex flex-col gap-4 max-w-4xl">
          <div class="flex flex-col gap-4">
            <p class="text-muted-foreground text-xs md:text-sm uppercase leading-none tracking-widest font-medium">
              THE UNION FOUNDATION
            </p>
            <h1 class="text-white text-4xl md:text-6xl lg:text-7xl xl:text-8xl font-bold leading-tight tracking-tight">
              Laying the new<br />foundation
            </h1>
          </div>

          <p class="text-white/80 text-xl md:text-2xl lg:text-3xl font-light leading-relaxed max-w-3xl">
            Dedicated to a more secure, decentralized,<br class="hidden md:block" /> and
            interoperable web3.
          </p>
        </div>
      </div>
    </div>
  </div>
</div>

<style>
/* Custom styles for better typography and spacing */
.container {
	margin-left: auto;
	margin-right: auto;
	padding-left: 1rem;
	padding-right: 1rem;
}

@keyframes appear {
	0% {
		opacity: 0;
	}
	100% {
		opacity: 1;
	}
}

.animate-appear {
	animation: appear 1.5s ease-out 0.5s forwards;
}

/* Grid background - needs to be defined in tailwind.config.ts */
.bg-grid-white\/\[0\.03\] {
	background-image: radial-gradient(circle, rgba(255, 255, 255, 0.03) 1px, transparent 1px);
	background-size: 20px 20px;
}

/* Responsive font sizing */
h1 {
	line-height: 1.1;
}

/* Ensure proper text colors for dark theme */
.text-foreground {
	color: white;
}

.text-muted-foreground {
	color: rgba(255, 255, 255, 0.7);
}

.border-muted-foreground {
	border-color: rgba(255, 255, 255, 0.7);
}

.border-muted-foreground\/40 {
	border-color: rgba(255, 255, 255, 0.4);
}

/* Leading snug override */
.leading-snug\! {
	line-height: 1.375 !important;
}
</style>
