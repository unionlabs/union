<script lang="ts">
import Card from "$lib/components/ui/Card.svelte"
import Button from "$lib/components/ui/Button.svelte"
import Sections from "$lib/components/ui/Sections.svelte"
import { onMount, onDestroy } from "svelte";

let videoHovered = $state(false)
let showPlayButton = $state(true)
let videoElement: HTMLVideoElement
let activeTab = $state<'mad' | 'og'>('mad')
let sceneLoaded = $state(false)

function handlePlayClick() {
  if (videoElement) {
    videoElement.currentTime = 0
    videoElement.muted = false
    showPlayButton = false
    // Request fullscreen with proper typing
    if (videoElement.requestFullscreen) {
      videoElement.requestFullscreen()
    } else if ((videoElement as any).webkitRequestFullscreen) {
      // Safari
      (videoElement as any).webkitRequestFullscreen()
    } else if ((videoElement as any).msRequestFullscreen) {
      // IE/Edge
      (videoElement as any).msRequestFullscreen()
    }
  }
}

onMount(() => {
    // Load the Unicorn Studio script
    const script = document.createElement('script');
    script.type = 'text/javascript';
    script.textContent = `
      !function(){
        if(!window.UnicornStudio){
          window.UnicornStudio={isInitialized:!1};
          var i=document.createElement("script");
          i.src="https://cdn.jsdelivr.net/gh/hiunicornstudio/unicornstudio.js@v1.4.28/dist/unicornStudio.umd.js";
          i.onload=function(){
            if(!window.UnicornStudio.isInitialized){
              UnicornStudio.init().then(scenes => {
                console.log('Unicorn Studio scenes loaded:', scenes);
                window.UnicornStudio.isInitialized = true;
                // Dispatch custom event to notify that scenes are ready
                window.dispatchEvent(new CustomEvent('unicornStudioReady'));
              }).catch((err) => {
                console.error('Unicorn Studio init error:', err);
              });
            }
          };
          (document.head || document.body).appendChild(i);
        }
      }();
    `;
    
    document.head.appendChild(script);
    
    // Listen for when scenes are ready
    const handleScenesReady = () => {
      sceneLoaded = true;
    };
    
    window.addEventListener('unicornStudioReady', handleScenesReady);
    
    // Cleanup function
    return () => {
      window.removeEventListener('unicornStudioReady', handleScenesReady);
    };
});

onDestroy(() => {
  if ((window as any).UnicornStudio && (window as any).UnicornStudio.destroy) {
    (window as any).UnicornStudio.destroy();
  }
});
</script>


<div class="fixed md:left-64 inset-0">
  <!-- Unicorn Studio -->
  <div data-us-project="XhVxd2zKyAwt8YAl5a2m" class="absolute inset-0 w-full h-full"></div>

  <!-- Gritty vignette overlay -->
  <div class="absolute inset-0 z-20 pointer-events-none" style="background: radial-gradient(ellipse at center, rgba(0,0,0,0.0) 0%, rgba(0,0,0,0.1) 30%, rgba(139,69,19,0.3) 60%, rgba(0,0,0,0.5) 100%)"></div>
  
  <!-- Dust/noise texture overlay -->
  <div class="absolute inset-0 opacity-40 mix-blend-overlay z-20 pointer-events-none" style="background-image: url('data:image/svg+xml,%3Csvg viewBox=%220 0 200 200%22 xmlns=%22http://www.w3.org/2000/svg%22%3E%3Cfilter id=%22noiseFilter%22%3E%3CfeTurbulence type=%22fractalNoise%22 baseFrequency=%220.9%22 numOctaves=%224%22 /%3E%3C/filter%3E%3Crect width=%22100%25%22 height=%22100%25%22 filter=%22url(%23noiseFilter)%22 /%3E%3C/svg%3E')"></div>
  <div class="absolute inset-0 opacity-20 mix-blend-overlay z-20 pointer-events-none" style="background-image: url('/grain.gif')"></div>

</div>

{#if sceneLoaded}
<div class="relative z-10">
<Sections class="max-w-6xl mx-auto">
  <!-- Hero Section with Logo -->
  <div class="relative mb-8 mt-4">

    
    <img 
      src="/mad.png" 
      alt="Mad Logo" 
      class="w-full h-auto object-contain relative  drop-shadow-[0_0_30px_rgba(234,88,12,0.8)] animate-flicker"
    />
  </div>

  <!-- Hero Content Card -->
  <div class="relative">
    <Card class="relative p-6 bg-gradient-to-br from-zinc-900/90 via-zinc-950/90 to-orange-950/30 border border-orange-900/50 backdrop-blur-sm">
      <div class="flex flex-col lg:flex-row 4 lg:gap-8">
        <!-- Video Section -->
        <div 
          class="relative group/video w-full lg:w-2/5 flex-shrink-0"
          onmouseenter={() => videoHovered = true}
          onmouseleave={() => videoHovered = false}
        >
          <div class="relative aspect-square bg-zinc-900 rounded-lg overflow-hidden ring-1 ring-zinc-800">
            <video
              bind:this={videoElement}
              class="w-full h-full object-cover"
              controls
              loop
              muted
              autoplay
              playsinline
            >
              <source
                src="https://videos.cdn.union.build/mad-yaps-v1.mp4"
                type="video/mp4"
              />
              Your browser does not support the video tag.
            </video>
            
            <!-- Play button overlay for unmuting -->
            {#if showPlayButton}
              <button
                onclick={handlePlayClick}
                class="absolute inset-0 flex items-center justify-center bg-black/20 hover:bg-black/30 transition-colors cursor-pointer group"
              >
                <div class="w-20 h-20 rounded-full bg-white/10 backdrop-blur-sm flex items-center justify-center group-hover:bg-white/20 transition-colors">
                  <svg class="w-8 h-8 text-white ml-1" fill="currentColor" viewBox="0 0 20 20">
                    <path d="M6.3 2.841A1.5 1.5 0 004 4.11V15.89a1.5 1.5 0 002.3 1.269l9.344-5.89a1.5 1.5 0 000-2.538L6.3 2.84z" />
                  </svg>
                </div>
                <span class="absolute bottom-4 text-white text-sm font-medium bg-black/50 px-3 py-1 rounded-full">
                  Click to unmute
                </span>
              </button>
            {/if}
          </div>
        </div>

        <!-- Text Content Section -->
        <div class="flex flex-col justify-between w-full">
          <div class="space-y-6">
            <!-- Badge -->
            <div class="relative inline-flex">
              <div class="absolute inset-0 bg-gradient-to-r from-orange-500/20 to-yellow-500/20 blur-lg"></div>
              <div class="relative inline-flex items-center gap-1.5 px-3 py-1 rounded-full bg-gradient-to-r from-orange-500/10 to-yellow-500/10 border border-orange-500/30 backdrop-blur-sm">
                <div class="w-2 h-2 rounded-full bg-gradient-to-r from-orange-400 to-yellow-400 animate-pulse"></div>
                <span class="text-xs font-bold text-transparent bg-clip-text bg-gradient-to-r from-orange-400 to-yellow-400 uppercase tracking-wider">New Season</span>
              </div>
            </div>

            <!-- Main Content -->
            <div class="space-y-4">
              <h1 class="text-3xl font-black leading-tight uppercase">
                <span>UNION YAPPERS</span><br/>
              </h1>
              
              <p class="text-orange-200/80 text-md leading-relaxed font-medium">
                In the wasteland of Web3, only the mad survive. Join the warboys of DeFi as we ride to Valhalla on the Fury Road of decentralized chaos. 
              </p>
              
              <div class="flex flex-wrap gap-3 text-sm pt-2">
                <div class="flex items-center gap-2 text-orange-300/80 font-bold uppercase text-xs">
                  <svg class="w-4 h-4 text-orange-500 drop-shadow-[0_0_10px_rgba(234,88,12,0.8)]" fill="currentColor" viewBox="0 0 24 24">
                    <path d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z" />
                  </svg>
                  <span>WAR READY</span>
                </div>
                <div class="flex items-center gap-2 text-orange-300/80 font-bold uppercase text-xs">
                  <svg class="w-4 h-4 text-red-500 drop-shadow-[0_0_10px_rgba(239,68,68,0.8)]" fill="currentColor" viewBox="0 0 24 24">
                    <path d="M17.66 11.2C17.43 10.9 17.15 10.64 16.89 10.38C16.22 9.78 15.46 9.35 14.82 8.72C13.33 7.26 13 4.85 13.95 3C13 3.23 12.17 3.75 11.46 4.32C8.87 6.4 7.85 10.07 9.07 13.22C9.11 13.32 9.15 13.42 9.15 13.55C9.15 13.77 9 13.97 8.8 14.05C8.57 14.15 8.33 14.09 8.14 13.93C8.08 13.88 8.04 13.83 8 13.76C6.87 12.33 6.69 10.28 7.45 8.64C5.78 10 4.87 12.3 5 14.47C5.06 14.97 5.12 15.47 5.29 15.97C5.43 16.57 5.7 17.17 6 17.7C7.08 19.43 8.95 20.67 10.96 20.92C13.1 21.19 15.39 20.8 17.03 19.32C18.86 17.66 19.5 15 18.56 12.72L18.43 12.46C18.22 12 17.66 11.2 17.66 11.2M14.5 17.5C14.22 17.74 13.76 18 13.4 18.1C12.28 18.5 11.16 17.94 10.5 17.28C11.69 17 12.4 16.12 12.61 15.23C12.78 14.43 12.46 13.77 12.33 13C12.21 12.26 12.23 11.63 12.5 10.94C12.69 11.32 12.89 11.7 13.13 12C13.9 13 15.11 13.44 15.37 14.8C15.41 14.94 15.43 15.08 15.43 15.23C15.46 16.05 15.1 16.95 14.5 17.5H14.5Z" />
                  </svg>
                  <span>FULL OCTANE</span>
                </div>
                <div class="flex items-center gap-2 text-orange-300/80 font-bold uppercase text-xs">
                  <svg class="w-4 h-4 text-yellow-500 drop-shadow-[0_0_10px_rgba(234,179,8,0.8)]" fill="currentColor" viewBox="0 0 24 24">
                    <path d="M7.5,2C5.71,3.15 4.5,5.18 4.5,7.5C4.5,9.82 5.71,11.85 7.53,13L4.5,22H10.5L14.5,10L18.5,22H24.5L21.47,13C23.29,11.85 24.5,9.82 24.5,7.5C24.5,5.18 23.29,3.15 21.5,2" />
                  </svg>
                  <span>CHROME BLESSED</span>
                </div>
              </div>
            </div>
          </div>

          <!-- Action Button -->
          <div class="mt-6">
            <Button 
              variant="primary" 
            >
              <span class="skew-x-[5deg]">JOIN THE YAPPERS</span>
              <svg class="w-5 h-5 skew-x-[5deg]" fill="currentColor" viewBox="0 0 24 24">
                <path d="M5,17.59L15.59,7H9V5H19V15H17V8.41L6.41,19L5,17.59Z" />
              </svg>
            </Button>
          </div>
        </div>
      </div>
    </Card>
  </div>

  <!-- Leaderboard-style Percentage Card -->
  <div>
    <Card class="relative flex flex-col gap-4 p-4 bg-gradient-to-br from-zinc-900/90 via-zinc-950/90 to-orange-950/30 border border-orange-900/50 backdrop-blur-sm">
      <!-- Tab Header -->
      <div class="flex items-center justify-between">
        <div class="flex gap-0.5 p-1.5 bg-black rounded">
          <button
            class="px-4 py-1 text-xs font-bold uppercase tracking-widest transition-all rounded
              {activeTab === 'mad' 
                ? 'bg-amber-500 text-black shadow-sm' 
                : 'text-zinc-500 hover:text-zinc-300'}"
            onclick={() => activeTab = 'mad'}
          >
            Mad
          </button>
          <button
            class="px-4 py-1 text-xs font-bold uppercase tracking-widest transition-all rounded
              {activeTab === 'og' 
                ? 'bg-yellow-500 text-black shadow-sm' 
                : 'text-zinc-500 hover:text-zinc-300'}"
            onclick={() => activeTab = 'og'}
          >
            OG
          </button>
        </div>
      </div>

      <ul class="list-none p-0 flex flex-col gap-3">
        {#each (activeTab === 'mad' ? [
          { name: "Chrome Warrior", percentage: 97.2 },
          { name: "Fury Road Runner", percentage: 93.8 },
          { name: "War Rig Driver", percentage: 88.5 },
          { name: "Guzzoline Master", percentage: 84.1 },
          { name: "Wasteland King", percentage: 79.7 },
          { name: "Immortan's Hand", percentage: 75.3 },
          { name: "Buzzard Raider", percentage: 71.9 },
          { name: "Desert Scavenger", percentage: 67.4 },
          { name: "Road Warrior", percentage: 63.2 },
          { name: "Rust Survivor", percentage: 58.8 }
        ] : [
          { name: "Genesis Builder", percentage: 96.5 },
          { name: "Protocol Pioneer", percentage: 91.2 },
          { name: "Early Adopter", percentage: 87.8 },
          { name: "Chain Veteran", percentage: 83.4 },
          { name: "DeFi Original", percentage: 78.9 },
          { name: "Block Explorer", percentage: 74.5 },
          { name: "Yield Architect", percentage: 70.1 },
          { name: "Gas Wizard", percentage: 65.7 },
          { name: "Stake Master", percentage: 61.3 },
          { name: "Liquidity Legend", percentage: 56.9 }
        ]) as entry, rank}
          <li
            class="flex gap-3 items-center py-2 px-2 rounded-lg
              {rank === 0 ? 'bg-orange-500/5' : 
               rank === 1 ? 'bg-yellow-500/5' : 
               rank === 2 ? 'bg-amber-700/5' : 
               ''}"
          >
            <!-- Rank badge -->
            <div class="font-bold text-[13px] w-6 h-6 flex justify-center items-center rounded
              {rank === 0 ? 'text-orange-500 bg-orange-500/10' : 
               rank === 1 ? 'text-yellow-500 bg-yellow-500/10' : 
               rank === 2 ? 'text-amber-700 bg-amber-700/10' : 
               'text-zinc-300 bg-zinc-800/50'}">
              {rank + 1}
            </div>

            <!-- Name -->
            <div class="flex-1">
              <div class="text-sm {rank === 0 ? 'text-orange-500' : 
                                   rank === 1 ? 'text-yellow-500' : 
                                   rank === 2 ? 'text-amber-700' : 
                                   'text-zinc-100'}">
                {entry.name}
              </div>
            </div>

            <!-- Percentage with progress bar -->
            <div class="flex items-center gap-3 min-w-[120px]">
              <div class="flex-1 h-1.5 bg-zinc-800 rounded-full overflow-hidden">
                <div 
                  class="h-full transition-all duration-700 rounded-full
                    {rank === 0 ? 'bg-gradient-to-r from-orange-500 to-yellow-500' : 
                     rank === 1 ? 'bg-yellow-500' : 
                     rank === 2 ? 'bg-amber-700' : 
                     'bg-zinc-600'}"
                  style="width: {entry.percentage}%"
                ></div>
              </div>
              <div class="text-sm font-medium min-w-[50px] text-right
                {rank === 0 ? 'text-orange-500' : 
                 rank === 1 ? 'text-yellow-500' : 
                 rank === 2 ? 'text-amber-700' : 
                 'text-zinc-200'}">
                {entry.percentage}%
              </div>
            </div>
          </li>
        {/each}
      </ul>
    </Card>
  </div>

</Sections>
</div>
{:else}
<!-- Loading state while Unicorn Studio loads -->
<div class="flex items-center justify-center min-h-screen">
  <div class="text-white text-lg">Loading...</div>
</div>
{/if}


<style>
  @keyframes flicker {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.8; }
    25% { opacity: 0.95; }
    75% { opacity: 0.85; }
  }
  
  :global(.animate-flicker) {
    animation: flicker 3s ease-in-out infinite;
  }
  
  @keyframes pulse-slow {
    0%, 100% { opacity: 0.8; }
    50% { opacity: 1; }
  }
  
  :global(.animate-pulse-slow) {
    animation: pulse-slow 4s ease-in-out infinite;
  }
</style>  