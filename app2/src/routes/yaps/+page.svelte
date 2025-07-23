<script lang="ts">
  import Button from "$lib/components/ui/Button.svelte"
  import Card from "$lib/components/ui/Card.svelte"
  import Sections from "$lib/components/ui/Sections.svelte"
  import { getYapsSeason0, getYapsSeason1, type YapsSeason } from "$lib/dashboard/queries/public"
  import { runPromise } from "$lib/runtime"
  import { Effect, Option, pipe } from "effect"
  import { onDestroy, onMount } from "svelte"
  import YappersTable from "./YappersTable.svelte"
  import YappersPodium from "./YappersPodium.svelte"
  import YappersHero from "./YappersHero.svelte"


  let activeTab = $state<"season1" | "season0">("season1")
  let sceneLoaded = $state(false)
  let season0Data = $state<YapsSeason[] | null>(null)
  let season0Loading = $state(true)
  let season1Data = $state<YapsSeason[] | null>(null)
  let season1Loading = $state(true)
  let contentReady = $state(false)
  let searchQuery = $state("")
  let currentPage = $state(1)
  const itemsPerPage = 50

  // Computed values for current data
  const currentData = $derived(activeTab === "season1" ? season1Data : season0Data)
  const currentLoading = $derived(activeTab === "season1" ? season1Loading : season0Loading)


onMount(() => {
  const checkAndSetContentReady = () => {
    if (sceneLoaded && !season0Loading && !season1Loading) {
      setTimeout(() => contentReady = true, 200)
    }
  }

  runPromise(
    pipe(
      Effect.all([
        pipe(
          getYapsSeason0(),
          Effect.catchAll(() => Effect.succeed(Option.none()))
        ),
        pipe(
          getYapsSeason1(),
          Effect.catchAll(() => Effect.succeed(Option.none()))
        )
      ]),
      Effect.tap(([season0Result, season1Result]) => {
        // Handle Season 0 result
        if (Option.isSome(season0Result)) {
          season0Data = season0Result.value
        }
        season0Loading = false

        // Handle Season 1 result
        if (Option.isSome(season1Result)) {
          season1Data = season1Result.value
        }
        season1Loading = false

        checkAndSetContentReady()
        return Effect.void
      }),
      Effect.catchAll((error) => {
        season0Loading = false
        season1Loading = false
        checkAndSetContentReady()
        return Effect.succeed(undefined)
      }),
    ),
  )

  // Load the Unicorn Studio script
  const script = document.createElement("script")
  script.type = "text/javascript"
  script.textContent = `
      !function(){
        if(!window.UnicornStudio){
          window.UnicornStudio={isInitialized:!1};
          var i=document.createElement("script");
          i.src="https://cdn.jsdelivr.net/gh/hiunicornstudio/unicornstudio.js@v1.4.28/dist/unicornStudio.umd.js";
          i.onload=function(){
            if(!window.UnicornStudio.isInitialized){
              UnicornStudio.init().then(scenes => {
                window.UnicornStudio.isInitialized = true;

                window.dispatchEvent(new CustomEvent('unicornStudioReady'));
              }).catch((err) => {
                console.error('Unicorn Studio init error:', err);
              });
            }
          };
          (document.head || document.body).appendChild(i);
        }
      }();
    `

  document.head.appendChild(script)

  const handleScenesReady = () => {
    sceneLoaded = true
    checkAndSetContentReady()
  }

  window.addEventListener("unicornStudioReady", handleScenesReady)

  return () => {
    window.removeEventListener("unicornStudioReady", handleScenesReady)
  }
})

onDestroy(() => {
  if ((window as any).UnicornStudio && (window as any).UnicornStudio.destroy) {
    ;(window as any).UnicornStudio.destroy()
  }
})
</script>

<div class="fixed sm:left-64 inset-0 bg-zinc-950">
  <div
    data-us-project="XhVxd2zKyAwt8YAl5a2m"
    class="absolute inset-0 w-full h-full"
  >
  </div>
</div>

{#if contentReady}
  <div class="relative z-10 animate-fade-in">
    <Sections class="max-w-6xl mx-auto">
      <div class="relative mt-4">
        <img
          src="/mad.png"
          alt="Mad Logo"
          class="w-full h-auto object-contain relative drop-shadow-[0_0_30px_rgba(234,88,12,0.8)]"
        />
      </div>

      <YappersHero />

      <div>
        <Card
          class="relative flex flex-col gap-6 p-6 bg-gradient-to-br from-zinc-900/90 via-zinc-950/90 to-orange-950/30 border border-orange-900/50 backdrop-blur-sm"
        >
          <!-- Tab Header -->
          <div class="flex flex-col sm:flex-row sm:justify-between sm:items-center gap-4">
            <div class="flex gap-0.5 p-1.5 bg-black rounded flex-shrink-0 self-start">
              <button
                class="
                  px-4 py-1 text-xs font-bold uppercase tracking-widest transition-all rounded
                  {activeTab === 'season0'
                  ? 'bg-white text-black shadow-sm'
                  : 'text-zinc-500 hover:text-zinc-300'}
                "
                onclick={() => {
                  activeTab = "season0"
                  searchQuery = ""
                  currentPage = 1
                }}
              >
                Season 0
              </button>
              <button
                class="
                  px-4 py-1 text-xs font-bold uppercase tracking-widest transition-all rounded
                  {activeTab === 'season1'
                  ? 'bg-white text-black shadow-sm'
                  : 'text-zinc-500 hover:text-zinc-300'}
                "
                onclick={() => {
                  activeTab = "season1"
                  searchQuery = ""
                  currentPage = 1
                }}
              >
                Season 1
              </button>
            </div>

            <!-- Search - Desktop only -->
            <div class="hidden sm:block relative w-64">
              <input
                type="text"
                bind:value={searchQuery}
                oninput={() => currentPage = 1}
                placeholder="Search yappers..."
                class="w-full px-3 py-2 text-sm bg-zinc-800/50 border border-zinc-700/60 rounded-lg text-white placeholder-zinc-400 focus:outline-none focus:border-orange-500/50 focus:ring-2 focus:ring-orange-500/20 transition-all duration-200"
              />
              <svg
                class="absolute right-3 top-2.5 w-4 h-4 text-zinc-400"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="m21 21-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
                />
              </svg>
            </div>
          </div>

          {@const entries = currentData || []}

          <!-- Podium -->
          {#if !searchQuery}
            <YappersPodium {entries} />
          {/if}

          <!-- Search - Mobile only -->
          <div class="sm:hidden relative max-w-md">
            <input
              type="text"
              bind:value={searchQuery}
              oninput={() => currentPage = 1}
              placeholder="Search yappers..."
              class="w-full px-3 py-2 text-sm bg-zinc-800/50 border border-zinc-700/60 rounded-lg text-white placeholder-zinc-400 focus:outline-none focus:border-orange-500/50 focus:ring-2 focus:ring-orange-500/20 transition-all duration-200"
            />
            <svg
              class="absolute right-3 top-2.5 w-4 h-4 text-zinc-400"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="m21 21-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
              />
            </svg>
          </div>

          <!-- Table -->
          <YappersTable {entries} {searchQuery} bind:currentPage {itemsPerPage} />
        </Card>
      </div>
    </Sections>
  </div>
{:else}
  <!-- Loading state while content loads -->
  <div class="flex items-center justify-center min-h-screen">
    <div class="flex flex-col items-center gap-4">
      <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-orange-500"></div>
      <div class="text-white text-lg font-medium">Loading Yaps...</div>
      {#if season0Loading || season1Loading}
        <div class="text-zinc-400 text-sm">
          {#if season0Loading && season1Loading}
            Loading both seasons...
          {:else if season0Loading}
            Loading Season 0...
          {:else if season1Loading}
            Loading Season 1...
          {/if}
        </div>
      {/if}
    </div>
  </div>
{/if}

<style>
@keyframes fade-in {
  0% { 
    opacity: 0; 
    transform: translateY(20px);
  }
  100% { 
    opacity: 1; 
    transform: translateY(0);
  }
}


:global(.animate-fade-in) {
  animation: fade-in 0.6s ease-out forwards;
}
</style>
