<script lang="ts">
import Button from "$lib/components/ui/Button.svelte"
import type { YapsSeason } from "$lib/dashboard/queries/public"

interface Props {
  entries: YapsSeason[]
  searchQuery: string
  currentPage: number
  itemsPerPage: number
  openTeamModal: () => void
}

let { entries, searchQuery, currentPage = $bindable(), itemsPerPage, openTeamModal }: Props =
  $props()

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

let filteredEntries = $derived(
  searchQuery
    ? entries.filter(entry => {
      const username = entry.username?.toLowerCase() || ""
      const search = searchQuery.toLowerCase().replace(/[^a-z0-9]/g, "")
      const clean = username.replace(/[^a-z0-9]/g, "")
      return username.includes(searchQuery.toLowerCase()) || clean.includes(search)
    })
    : entries,
)

// Show all users in table, even if they appear in podium (podium is just a highlight)
let podiumSkipCount = 0

let totalPages = $derived(
  Math.ceil(Math.max(0, filteredEntries.length - podiumSkipCount) / itemsPerPage),
)
let listStartIndex = $derived(podiumSkipCount + (currentPage - 1) * itemsPerPage)
let listEntries = $derived(filteredEntries.slice(listStartIndex, listStartIndex + itemsPerPage))
</script>

<div class="overflow-visible">
  {#if searchQuery && listEntries.length === 0}
    <div class="text-center py-8 text-zinc-400">
      No results found for "{searchQuery}"
    </div>
  {:else}
    <table
      class="w-full table-fixed border-separate"
      style="border-spacing: 0 4px;"
    >
      <tbody>
        {#each listEntries as entry, index}
          {@const rank = searchQuery
          ? entries.findIndex(e => e === entry) + 1
          : listStartIndex + index + 1}
          <tr class="bg-zinc-800/20 hover:bg-zinc-700/40 border border-zinc-700/30 hover:border-zinc-600/50 transition-all duration-200 rounded-lg">
            <!-- Rank badge -->
            <td class="py-2 px-3 w-16 rounded-l-lg">
              <div class="min-w-[2rem] h-6 px-2 rounded bg-zinc-700/80 flex items-center justify-center text-xs font-bold text-zinc-300">
                {rank}
              </div>
            </td>

            <!-- Avatar -->
            <td class="py-2 px-3 w-12">
              <div class="w-8 h-8 rounded-full bg-zinc-800/80 backdrop-blur-sm border border-zinc-600/50 flex items-center justify-center overflow-hidden">
                <img
                  src={getAvatarUrl(entry.username, entry.pfp)}
                  alt={entry.username}
                  class="w-full h-full object-cover rounded-full"
                  onerror={createAvatarErrorHandler(entry.username || "")}
                />
              </div>
            </td>

            <!-- User info -->
            <td class="py-2 px-3 max-w-0 w-full">
              <div class="text-sm text-zinc-100 font-medium truncate max-w-full">
                @{entry.username?.toLowerCase().replace(" ", "")}
              </div>
            </td>

            <!-- Team badge column -->
            <td class="py-2 px-0 w-4">
              <div class="flex items-center justify-end h-full">
                {#if entry.team}
                  <button
                    class="size-5 text-orange-500 hover:text-orange-400 transition-colors cursor-pointer flex items-center justify-center"
                    onclick={openTeamModal}
                    aria-label="Team member information"
                  >
                    <svg
                      class="w-full h-full"
                      fill="currentColor"
                      viewBox="0 0 24 24"
                    >
                      <path d="M11,9H13V7H11M12,20C7.59,20 4,16.41 4,12C4,7.59 7.59,4 12,4C16.41,4 20,7.59 20,12C20,16.41 16.41,20 12,20M12,2A10,10 0 0,0 2,12A10,10 0 0,0 12,22A10,10 0 0,0 22,12A10,10 0 0,0 12,2M11,17H13V11H11V17Z" />
                    </svg>
                  </button>
                {/if}
              </div>
            </td>

            <!-- Mindshare -->
            <td class="py-2 px-3 w-16 rounded-r-lg">
              <div class="flex items-center justify-end h-full">
                <div class="font-bold text-sm {entry.team ? 'text-zinc-500' : 'text-white'}">
                  {formatMindshare(entry.mindshare)}
                </div>
              </div>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}

  <!-- Pagination -->
  {#if totalPages > 1}
    <div class="flex justify-center items-center gap-2 mt-6">
      <Button
        variant="secondary"
        onclick={() => {
          if (currentPage > 1) {
            currentPage = currentPage - 1
          }
        }}
      >
        Previous
      </Button>

      <span class="text-sm text-zinc-400 px-3">
        Page {currentPage} of {totalPages}
      </span>

      <Button
        variant="secondary"
        onclick={() => {
          if (currentPage < totalPages) {
            currentPage = currentPage + 1
          }
        }}
      >
        Next
      </Button>
    </div>
  {/if}
</div>
