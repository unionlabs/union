<script lang="ts">
import Card from "$lib/components/ui/Card.svelte"
import Sections from "$lib/components/ui/Sections.svelte"
import {
  getYapsSeason0Public,
  getYapsSeason1Public,
  type YapsSeason,
} from "$lib/dashboard/queries/public"
import { runPromise } from "$lib/runtime"
import { Effect, Option, pipe } from "effect"
import { onDestroy, onMount } from "svelte"
import YappersHero from "./YappersHero.svelte"
import YappersPodium from "./YappersPodium.svelte"
import YappersTable from "./YappersTable.svelte"
import YappersTeamModal from "./YappersTeamModal.svelte"

// Reset all state on navigation
let activeTab = $state<"season1" | "season0">("season1")
let sceneLoaded = $state(false)
let season0Data = $state<YapsSeason[] | null>(null)
let season0Loading = $state(true)
let season1Data = $state<YapsSeason[] | null>(null)
let season1Loading = $state(true)
let contentReady = $state(false)
let currentPage = $state(1)
const itemsPerPage = 50
let isTeamModalOpen = $state(false)
let showLimitTooltip = $state(false)

// Reset function to ensure clean state on navigation
function resetPageState() {
  sceneLoaded = false
  season0Data = null
  season0Loading = true
  season1Data = null
  season1Loading = true
  contentReady = false
  currentPage = 1
  isTeamModalOpen = false
  showLimitTooltip = false
}

const currentData = $derived(activeTab === "season1" ? season1Data : season0Data)

function openTeamModal() {
  isTeamModalOpen = true
}

function closeTeamModal() {
  isTeamModalOpen = false
}

function toggleLimitTooltip() {
  showLimitTooltip = !showLimitTooltip
}

function closeLimitTooltip() {
  showLimitTooltip = false
}

onMount(() => {
  resetPageState()

  const checkAndSetContentReady = () => {
    if (sceneLoaded && !season0Loading && !season1Loading) {
      setTimeout(() => contentReady = true, 200)
    }
  }

  // Load public season data
  runPromise(
    pipe(
      Effect.all([
        pipe(
          getYapsSeason0Public(),
          Effect.catchAll(() => Effect.succeed(Option.none())),
        ),
        pipe(
          getYapsSeason1Public(),
          Effect.catchAll(() => Effect.succeed(Option.none())),
        ),
      ]),
      Effect.tap(([season0Result, season1Result]) => {
        if (Option.isSome(season0Result)) {
          season0Data = season0Result.value
        }
        season0Loading = false

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

  const initializeUnicornStudio = () => {
    const windowWithUnicorn = window as any
    if (windowWithUnicorn.UnicornStudio && windowWithUnicorn.UnicornStudio.isInitialized) {
      windowWithUnicorn.UnicornStudio.isInitialized = false

      setTimeout(() => {
        try {
          windowWithUnicorn.UnicornStudio.init().then((scenes: any) => {
            windowWithUnicorn.UnicornStudio.isInitialized = true
            sceneLoaded = true
            checkAndSetContentReady()
          }).catch((err: any) => {
            sceneLoaded = true
            checkAndSetContentReady()
          })
        } catch (err) {
          sceneLoaded = true
          checkAndSetContentReady()
        }
      }, 100)
      return
    }

    // Check if script is already loaded but not initialized
    if (windowWithUnicorn.UnicornStudio && !windowWithUnicorn.UnicornStudio.isInitialized) {
      windowWithUnicorn.UnicornStudio.init().then((scenes: any) => {
        windowWithUnicorn.UnicornStudio.isInitialized = true
        sceneLoaded = true
        checkAndSetContentReady()
      }).catch((err: any) => {
        sceneLoaded = true
        checkAndSetContentReady()
      })
      return
    }

    // Load script if not present
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
                  window.dispatchEvent(new CustomEvent('unicornStudioReady'));
                });
              }
            };
            i.onerror = function(err) {
              window.dispatchEvent(new CustomEvent('unicornStudioReady'));
            };
            (document.head || document.body).appendChild(i);
          }
        }();
      `
    document.head.appendChild(script)
  }

  initializeUnicornStudio()

  const handleScenesReady = () => {
    if (!sceneLoaded) {
      sceneLoaded = true
      checkAndSetContentReady()
    }
  }

  window.addEventListener("unicornStudioReady", handleScenesReady)

  const fallbackTimeout = setTimeout(() => {
    if (!sceneLoaded) {
      sceneLoaded = true
      checkAndSetContentReady()
    }
  }, 5000)

  const contentFallbackTimeout = setTimeout(() => {
    contentReady = true
  }, 8000)

  return () => {
    window.removeEventListener("unicornStudioReady", handleScenesReady)
    clearTimeout(fallbackTimeout)
    clearTimeout(contentFallbackTimeout)
  }
})

onDestroy(() => {
  const windowWithUnicorn = window as any
  if (windowWithUnicorn.UnicornStudio && windowWithUnicorn.UnicornStudio.destroy) {
    windowWithUnicorn.UnicornStudio.destroy()
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
      <div class="flex flex-col items-center relative mt-4">
        <img
          src="/yaps/unionxkaito.png"
          alt="Mad Logo"
          class="w-72 h-auto mb-6"
        />
        <img
          src="/yaps/mad.png"
          alt="Mad Logo"
          class="w-full h-auto object-contain relative drop-shadow-[0_0_30px_rgba(234,88,12,0.8)]"
        />
      </div>

      <YappersHero />

      <div>
        <Card
          class="relative flex flex-col gap-6 p-6 bg-gradient-to-br from-zinc-900/90 via-zinc-950/90 to-orange-950/30 border border-orange-900/50 backdrop-blur-sm"
          onclick={(e) => {
            const target = e.target as HTMLElement
            if (target && !target.closest(".relative.group")) {
              closeLimitTooltip()
            }
          }}
        >
          <!-- Tab Header -->
          <div class="flex flex-col gap-4">
            <!-- Tabs and Badge Container -->
            <div class="flex flex-col sm:flex-row sm:justify-between sm:items-center gap-3">
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
                    currentPage = 1
                  }}
                >
                  Season 1
                </button>
              </div>

              <!-- Season Date Info -->
              <div class="block">
                {#if activeTab === "season0"}
                  <div class="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-full bg-zinc-800/60 border border-zinc-700/50 text-xs font-medium text-zinc-300">
                    <div class="w-2 h-2 rounded-full bg-zinc-500"></div>
                    <span>Concluded Oct 23 - Jul 25</span>
                  </div>
                {:else}
                  <div class="relative inline-flex items-center gap-1.5 px-3 py-1.5 rounded-full bg-gradient-to-r from-orange-500/10 to-yellow-500/10 border border-orange-500/30 backdrop-blur-sm">
                    <div class="w-2 h-2 rounded-full bg-gradient-to-r from-orange-400 to-yellow-400 animate-pulse">
                    </div>
                    <span
                      class="text-xs font-bold text-transparent bg-clip-text bg-gradient-to-r from-orange-400 to-yellow-400"
                    >Live since Jul 23</span>
                  </div>
                {/if}
              </div>
            </div>
          </div>

          {@const entries = currentData || []}

          <!-- Podium (only show for Season 0) -->
          {#if activeTab === "season0"}
            <YappersPodium {entries} />
          {/if}

          <!-- Display Limit Notice -->
          <div class="flex items-center justify-center gap-1.5 text-zinc-400 text-sm mb-4">
            <span>Display Limited to 100 Yappers</span>
            <div class="relative group">
              <button
                class="w-3.5 h-3.5 text-orange-500 hover:text-orange-400 transition-colors cursor-pointer flex items-center justify-center -mt-px"
                onclick={toggleLimitTooltip}
                aria-label="More information about display limit"
              >
                <svg
                  class="w-full h-full"
                  fill="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path d="M11,9H13V7H11M12,20C7.59,20 4,16.41 4,12C4,7.59 7.59,4 12,4C16.41,4 20,7.59 20,12C20,16.41,16.41,20 12,20M12,2A10,10 0 0,0 2,12A10,10 0 0,0 12,22A10,10 0 0,0 22,12A10,10 0 0,0 12,2M11,17H13V11H11V17Z" />
                </svg>
              </button>

              <!-- Custom tooltip -->
              <div
                class="
                  absolute bottom-full left-1/2 transform -translate-x-1/2 mb-2 w-80 sm:w-96 max-w-[90vw] p-3 bg-zinc-900 border border-orange-500/30 rounded-lg shadow-xl text-xs text-zinc-200 leading-relaxed z-50 transition-all duration-200
                  {showLimitTooltip ? 'opacity-100 visible' : 'opacity-0 invisible'}
                  group-hover:opacity-100 group-hover:visible
                "
              >
                Mindshare data is currently limited to displaying the top 100 yappers due to API
                constraints. However, there is no cap on the total number of eligible yappers—those
                ranked beyond 100 will still receive rewards proportional to their mindshare.
                <div class="absolute top-full left-1/2 transform -translate-x-1/2 w-0 h-0 border-l-4 border-r-4 border-t-4 border-transparent border-t-zinc-900">
                </div>
              </div>
            </div>
          </div>

          <!-- Table -->
          <YappersTable
            {entries}
            searchQuery=""
            bind:currentPage
            {itemsPerPage}
            {openTeamModal}
          />
        </Card>
      </div>
    </Sections>
  </div>
{:else}
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

<YappersTeamModal
  isOpen={isTeamModalOpen}
  onClose={closeTeamModal}
/>

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
