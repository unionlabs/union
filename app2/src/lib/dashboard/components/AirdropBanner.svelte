<script lang="ts">
import { goto } from "$app/navigation"
import Button from "$lib/components/ui/Button.svelte"
import Card from "$lib/components/ui/Card.svelte"

function navigateToAirdrop() {
  goto("/udrop/check")
}
</script>

<Card class="relative overflow-hidden">
  <div class="relative flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
    <div class="flex items-center gap-4">
      <div class="flex flex-col gap-1">
        <h3 class="text-lg font-semibold text-white group-hover:text-accent transition-colors">
          🎁 Airdrop checker
        </h3>
        <p class="text-sm text-zinc-400 group-hover:text-zinc-300 transition-colors">
          Check your eligibility and see if you qualify for the Union airdrop
        </p>
      </div>
    </div>

    <!-- CTA Button -->
    <div class="flex sm:justify-end">
      <Button
        variant="secondary"
        onclick={navigateToAirdrop}
      >
        Check Eligibility
      </Button>
    </div>
  </div>
</Card>
