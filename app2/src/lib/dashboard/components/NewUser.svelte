<script lang="ts">
import Card from "$lib/components/ui/Card.svelte";
import SpinnerIcon from "$lib/components/icons/SpinnerIcon.svelte";
import { dashboard } from "$lib/dashboard/stores/user.svelte";
import { Option } from "effect";
import { onMount, onDestroy } from "svelte";
  import Sections from "$lib/components/ui/Sections.svelte";

let isNewUser = $state(false);
let intervalId: ReturnType<typeof setInterval>;

function checkIfNewUser() {
  Option.match(dashboard.user, {
    onNone: () => {
      isNewUser = false;
    },
    onSome: (user) => {
      const createdAt = new Date(user.created_at).getTime();
      const oneHourAgo = Date.now() - (60 * 5 * 1000);
      isNewUser = createdAt > oneHourAgo;
    }
  });
}

onMount(() => {
  checkIfNewUser();
  // Check every minute
  intervalId = setInterval(checkIfNewUser, 60 * 1000);
});

onDestroy(() => {
  if (intervalId) {
    clearInterval(intervalId);
  }
});
</script>

{#if isNewUser}
    <div class="absolute inset-0 flex flex-col gap-4 transition-all duration-300 bg-zinc-925/10 backdrop-blur-lg z-50">
        <Sections>
            <div class="flex flex-col items-center mt-36 text-center gap-4 h-svh">
                <div class="relative w-fit">
                    <div class="px-2 py-0.5 rounded-sm bg-zinc-800/80 border scale-110 border-accent/50 transition-all duration-300 flex items-center gap-2">
                        <SpinnerIcon class="size-3 text-accent animate-spin"/>
                        <span class="text-sm font-medium text-accent">Processing</span>
                    </div>
                    <div class="absolute inset-0 rounded-sm bg-accent/20 blur-sm animate-pulse"></div>
                </div>
                
                <h3 class="text-lg font-bold">Welcome to Union Dashboard</h3>
                
                <div class="text-sm text-neutral-300 space-y-2 max-w-md">
                    <p>Setting up your account might take a while. In the meantime, you can:</p>
                    <ul class="list-disc list-inside space-y-1 text-neutral-400">
                        <li>Add your social connections</li>
                        <li>Connect your wallets</li>
                        <li>Follow us on social media</li>
                        <li>Join our Discord server</li>
                    </ul>
                </div>
            </div>
        </Sections>
    </div>
{/if} 