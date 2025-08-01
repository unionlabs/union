<script lang="ts">
import type { YapsSeason } from "$lib/dashboard/queries/public"

interface Props {
  entries: YapsSeason[]
}

let { entries }: Props = $props()

function getAvatarUrl(username: string | null, pfp: string | null) {
  const cleanUsername = (username || "unknown").toLowerCase().replace(" ", "")
  if (pfp?.startsWith("pbs.twimg.com")) {
    return `https://${pfp}`
  }
  if (pfp?.startsWith("http")) {
    return pfp
  }
  return `https://unavatar.io/x/${cleanUsername}`
}

function createAvatarErrorHandler(username: string) {
  return (event: Event) => {
    const img = event.target as HTMLImageElement
    if (img && !img.src.includes("unavatar.io")) {
      img.src = `https://unavatar.io/x/${username.toLowerCase().replace(" ", "")}`
    }
  }
}

function formatMindshare(mindshare: string | null): string {
  if (!mindshare) {
    return "0.00%"
  }
  if (mindshare.includes("%")) {
    return mindshare
  }
  const value = parseFloat(mindshare)
  if (isNaN(value)) {
    return "0.00%"
  }
  return (value * 100).toFixed(2) + "%"
}

// Filter out team members and get top 3 non-team yappers
const podiumEntries = $derived(
  entries.filter(entry => !entry.team).slice(0, 3),
)
</script>

<!-- Podium with Design Consistency -->
<div class="flex flex-col lg:flex-row items-end justify-center gap-3 lg:gap-4 md:mt-10">
  <!-- 2nd Place -->
  <div class="w-full lg:flex-1 lg:max-w-xs order-2 lg:order-1">
    <!-- Mobile Layout -->
    <div class="flex flex-row lg:hidden h-20 backdrop-blur-sm border border-orange-400 rounded-xl p-3 relative">
      <!-- Full card gradient -->
      <div class="absolute inset-0 bg-gradient-to-r from-transparent to-orange-500/15 rounded-xl">
      </div>

      <!-- Content (left side) -->
      <div class="relative z-10 flex items-center gap-3 flex-1">
        <div class="w-10 h-10 rounded-full bg-zinc-800/80 backdrop-blur-sm border-2 border-orange-400 flex items-center justify-center overflow-hidden flex-shrink-0">
          <img
            src={getAvatarUrl(podiumEntries[1]?.username, podiumEntries[1]?.pfp)}
            alt={podiumEntries[1]?.username}
            class="w-full h-full object-cover rounded-full"
            onerror={createAvatarErrorHandler(podiumEntries[1]?.username || "")}
          />
        </div>
        <div class="flex flex-col">
          <div class="text-white text-xs">
            @{podiumEntries[1]?.username?.toLowerCase().replace(" ", "")}
          </div>
          <div class="text-orange-400 text-sm font-bold">
            {formatMindshare(podiumEntries[1]?.mindshare)}
          </div>
        </div>
      </div>

      <!-- Number (right side) -->
      <div class="relative z-10 flex items-center justify-center min-w-[4rem]">
        <img
          src="/yaps/2.png"
          alt="2nd Place"
          class="h-8"
        />
      </div>
    </div>

    <!-- Desktop Layout -->
    <div class="hidden lg:block relative backdrop-blur-sm border border-orange-400 rounded-xl">
      <!-- Gradient Background -->
      <div class="absolute inset-0 bg-gradient-to-b from-transparent to-orange-500/10 rounded-xl">
      </div>

      <!-- Avatar (popping out) -->
      <div class="absolute -top-8 left-1/2 -translate-x-1/2 w-16 h-16 rounded-full bg-zinc-800/80 backdrop-blur-sm border-4 border-orange-400 flex items-center justify-center overflow-hidden z-10">
        <img
          src={getAvatarUrl(podiumEntries[1]?.username, podiumEntries[1]?.pfp)}
          alt={podiumEntries[1]?.username}
          class="w-full h-full object-cover rounded-full"
          onerror={createAvatarErrorHandler(podiumEntries[1]?.username || "")}
        />
      </div>

      <!-- Content (normal flex behavior) -->
      <div class="relative z-10 flex flex-col items-center text-center pt-16 pb-6 px-4 gap-8">
        <!-- Text Content -->
        <div>
          <div class="text-white text-sm">
            @{podiumEntries[1]?.username?.toLowerCase().replace(" ", "")}
          </div>
          <div class="text-orange-400 text-base font-bold mt-1">
            {formatMindshare(podiumEntries[1]?.mindshare)}
          </div>
        </div>

        <!-- Number -->
        <img
          src="/yaps/2.png"
          alt="2nd Place"
          class="h-24 drop-shadow-lg"
        />
      </div>
    </div>
  </div>

  <!-- 1st Place (Winner) -->
  <div class="w-full lg:flex-1 lg:max-w-xs order-1 lg:order-2">
    <!-- Mobile Layout -->
    <div class="flex flex-row lg:hidden h-20 backdrop-blur-sm border border-red-400 rounded-xl p-3 relative">
      <!-- Full card gradient -->
      <div class="absolute inset-0 bg-gradient-to-r from-transparent to-red-500/20 rounded-xl">
      </div>

      <!-- Content (left side) -->
      <div class="relative z-10 flex items-center gap-3 flex-1">
        <div class="w-12 h-12 rounded-full bg-zinc-800/80 backdrop-blur-sm border-3 border-red-400 flex items-center justify-center overflow-hidden flex-shrink-0">
          <img
            src={getAvatarUrl(podiumEntries[0]?.username, podiumEntries[0]?.pfp)}
            alt={podiumEntries[0]?.username}
            class="w-full h-full object-cover rounded-full"
            onerror={createAvatarErrorHandler(podiumEntries[0]?.username || "")}
          />
        </div>
        <div class="flex flex-col">
          <div class="text-white text-sm">
            @{podiumEntries[0]?.username?.toLowerCase().replace(" ", "")}
          </div>
          <div class="text-red-400 text-lg font-bold">
            {formatMindshare(podiumEntries[0]?.mindshare)}
          </div>
        </div>
      </div>

      <!-- Number (right side) -->
      <div class="relative z-10 flex items-center justify-center min-w-[4rem]">
        <img
          src="/yaps/1.png"
          alt="1st Place"
          class="h-10"
        />
      </div>
    </div>

    <!-- Desktop Layout -->
    <div class="hidden lg:block relative backdrop-blur-sm border border-red-400 rounded-xl">
      <!-- Gradient Background -->
      <div class="absolute inset-0 bg-gradient-to-b from-transparent to-red-500/10 rounded-xl">
      </div>

      <!-- Avatar (popping out - bigger for winner) -->
      <div class="absolute -top-10 left-1/2 -translate-x-1/2 w-20 h-20 rounded-full bg-zinc-800/80 backdrop-blur-sm border-4 border-red-400 flex items-center justify-center overflow-hidden z-10">
        <img
          src={getAvatarUrl(podiumEntries[0]?.username, podiumEntries[0]?.pfp)}
          alt={podiumEntries[0]?.username}
          class="w-full h-full object-cover rounded-full"
          onerror={createAvatarErrorHandler(podiumEntries[0]?.username || "")}
        />
      </div>

      <!-- Content (normal flex behavior) -->
      <div class="relative z-10 flex flex-col items-center text-center pt-16 pb-8 px-4 gap-10">
        <!-- Text Content -->
        <div>
          <div class="text-white text-base">
            @{podiumEntries[0]?.username?.toLowerCase().replace(" ", "")}
          </div>
          <div class="text-red-400 text-xl font-bold mt-2">
            {formatMindshare(podiumEntries[0]?.mindshare)}
          </div>
        </div>

        <!-- Number -->
        <img
          src="/yaps/1.png"
          alt="1st Place"
          class="h-28 drop-shadow-lg"
        />
      </div>
    </div>
  </div>

  <!-- 3rd Place -->
  <div class="w-full lg:flex-1 lg:max-w-xs order-3 lg:order-3">
    <!-- Mobile Layout -->
    <div class="flex flex-row lg:hidden h-20 backdrop-blur-sm border border-yellow-400 rounded-xl p-3 relative">
      <!-- Full card gradient -->
      <div class="absolute inset-0 bg-gradient-to-r from-transparent to-yellow-500/15 rounded-xl">
      </div>

      <!-- Content (left side) -->
      <div class="relative z-10 flex items-center gap-3 flex-1">
        <div class="w-10 h-10 rounded-full bg-zinc-800/80 backdrop-blur-sm border-2 border-yellow-400 flex items-center justify-center overflow-hidden flex-shrink-0">
          <img
            src={getAvatarUrl(podiumEntries[2]?.username, podiumEntries[2]?.pfp)}
            alt={podiumEntries[2]?.username}
            class="w-full h-full object-cover rounded-full"
            onerror={createAvatarErrorHandler(podiumEntries[2]?.username || "")}
          />
        </div>
        <div class="flex flex-col">
          <div class="text-white text-xs">
            @{podiumEntries[2]?.username?.toLowerCase().replace(" ", "")}
          </div>
          <div class="text-yellow-400 text-sm font-bold">
            {formatMindshare(podiumEntries[2]?.mindshare)}
          </div>
        </div>
      </div>

      <!-- Number (right side) -->
      <div class="relative z-10 flex items-center justify-center min-w-[4rem]">
        <img
          src="/yaps/3.png"
          alt="3rd Place"
          class="h-8"
        />
      </div>
    </div>

    <!-- Desktop Layout -->
    <div class="hidden lg:block relative backdrop-blur-sm border border-yellow-400 rounded-xl">
      <!-- Gradient Background -->
      <div class="absolute inset-0 bg-gradient-to-b from-transparent to-yellow-500/10 rounded-xl">
      </div>

      <!-- Avatar (popping out) -->
      <div class="absolute -top-7 left-1/2 -translate-x-1/2 w-16 h-16 rounded-full bg-zinc-800/80 backdrop-blur-sm border-4 border-yellow-400 flex items-center justify-center overflow-hidden z-10">
        <img
          src={getAvatarUrl(podiumEntries[2]?.username, podiumEntries[2]?.pfp)}
          alt={podiumEntries[2]?.username}
          class="w-full h-full object-cover rounded-full"
          onerror={createAvatarErrorHandler(podiumEntries[2]?.username || "")}
        />
      </div>

      <!-- Content (normal flex behavior) -->
      <div class="relative z-10 flex flex-col items-center text-center pt-16 pb-5 px-4 gap-4">
        <!-- Text Content -->
        <div>
          <div class="text-white text-sm">
            @{podiumEntries[2]?.username?.toLowerCase().replace(" ", "")}
          </div>
          <div class="text-yellow-400 text-base font-bold mt-1">
            {formatMindshare(podiumEntries[2]?.mindshare)}
          </div>
        </div>

        <!-- Number -->
        <img
          src="/yaps/3.png"
          alt="3rd Place"
          class="h-24 drop-shadow-lg"
        />
      </div>
    </div>
  </div>
</div>
