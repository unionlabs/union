<script lang="ts">
interface Props {
  isOpen: boolean
  onClose: () => void
}

let { isOpen, onClose }: Props = $props()

function handleBackdropClick(event: MouseEvent) {
  if (event.target === event.currentTarget) {
    onClose()
  }
}

function handleKeydown(event: KeyboardEvent) {
  if (event.key === "Escape") {
    onClose()
  }
}
</script>

{#if isOpen}
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <div
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/80 backdrop-blur-sm"
    onclick={handleBackdropClick}
    onkeydown={handleKeydown}
    role="dialog"
    aria-modal="true"
    tabindex="-1"
  >
    <div class="relative w-full max-w-md mx-4 bg-zinc-900 border border-orange-500/30 rounded-lg shadow-xl">
      <!-- Fixed close button -->
      <div class="absolute top-4 right-4 z-10">
        <button
          onclick={onClose}
          class="w-8 h-8 rounded-full bg-zinc-800/80 border border-zinc-700/60 flex items-center justify-center text-zinc-400 hover:text-white hover:bg-zinc-700/80 transition-all duration-200"
          aria-label="Close modal"
        >
          <svg
            class="w-4 h-4"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M6 18L18 6M6 6l12 12"
            />
          </svg>
        </button>
      </div>

      <!-- Scrollable content -->
      <div class="max-h-[80vh] overflow-y-auto">
        <div class="p-6 pr-16">
          <!-- Header -->
          <div class="flex items-center gap-3 mb-6">
            <div class="w-8 h-8 bg-orange-500 rounded-full flex items-center justify-center">
              <svg
                class="w-5 h-5 text-white"
                fill="currentColor"
                viewBox="0 0 24 24"
              >
                <path d="M11,9H13V7H11M12,20C7.59,20 4,16.41 4,12C4,7.59 7.59,4 12,4C16.41,4 20,7.59 20,12C20,16.41 16.41,20 12,20M12,2A10,10 0 0,0 2,12A10,10 0 0,0 12,22A10,10 0 0,0 22,12A10,10 0 0,0 12,2M11,17H13V11H11V17Z" />
              </svg>
            </div>
            <h2 class="text-xl font-bold text-white">Team Member</h2>
          </div>

          <!-- Content -->
          <div class="space-y-4 text-zinc-200 leading-relaxed">
            <p>
              This user is a <span class="text-orange-400 font-semibold">Union team member</span>
              participating in the Mad Yaps leaderboard.
            </p>

            <p>
              As part of our commitment to the community, <span
                class="text-orange-400 font-semibold"
              >team members donate their entire mindshare percentage</span> back to the community
              pool.
            </p>

            <p>
              This means:
            </p>

            <ul class="list-disc list-inside space-y-2 ml-4 text-sm">
              <li>
                Team members will <span class="text-orange-400">not receive any rewards</span> from
                their mindshare
              </li>
              <li>
                Their percentage gets <span class="text-orange-400"
                >redistributed to all community yappers</span>
              </li>
              <li>This increases the reward pool for everyone else</li>
            </ul>

            <div class="mt-6 p-4 bg-orange-950/30 border border-orange-500/20 rounded-lg">
              <p class="text-sm text-orange-200">
                <span class="font-semibold">Community First:</span> Union believes in putting the
                community first. Team members compete for fun and recognition, while ensuring all
                rewards go to our amazing community members.
              </p>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
{/if}
