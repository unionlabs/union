<script lang="ts">
import Modal from "$lib/components/ui/Modal.svelte"

interface Props {
  isOpen: boolean
  onClose: () => void
}

const { isOpen, onClose }: Props = $props()
</script>

<Modal
  {isOpen}
  {onClose}
  class="w-full max-w-md"
>
  <div class="p-6 space-y-6">
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
        As part of our commitment to the community, <span class="text-orange-400 font-semibold"
        >team members donate their entire mindshare percentage</span> back to the community pool.
      </p>

      <p>
        This means:
      </p>

      <ul class="list-disc list-inside space-y-2 ml-4 text-sm">
        <li>
          Team members will <span class="text-orange-400">not receive any rewards</span> from their
          mindshare
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
          community first. Team members compete for fun and recognition, while ensuring all rewards
          go to our amazing community members.
        </p>
      </div>
    </div>
  </div>
</Modal>
