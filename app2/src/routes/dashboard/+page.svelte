<script lang="ts">
  import ProfileCard from "$lib/dashboard/components/ProfileCard.svelte";
  import SocialConnections from "$lib/dashboard/components/SocialConnections.svelte";
  import Sections from "$lib/components/ui/Sections.svelte";
  import MissionStats from "$lib/dashboard/components/MissionStats.svelte";
  import AchievementStats from "$lib/dashboard/components/AchievementStats.svelte";
  import RewardStats from "$lib/dashboard/components/RewardStats.svelte";
  import WalletStats from "$lib/dashboard/components/WalletStats.svelte";
  import Leaderboard from "$lib/dashboard/components/Leaderboard.svelte";
  import { dashboard } from "$lib/dashboard/stores/user.svelte";
  import { Effect, pipe } from "effect";
  import { extractErrorDetails } from "@unionlabs/sdk/utils";

  let isDeleting = false;
  let error: string | null = null;

  async function handleDelete() {
    if (!confirm("Are you sure you want to delete your account? This action cannot be undone.")) {
      return;
    }

    isDeleting = true;
    error = null;

    pipe(
      dashboard.deleteAccount(),
      Effect.catchAll((e) => {
        const errorDetails = extractErrorDetails(e);
        error = typeof errorDetails === 'string' ? errorDetails : JSON.stringify(errorDetails);
        console.error("Delete account error:", e);
        return Effect.void;
      }),
      Effect.ensuring(Effect.sync(() => {
        isDeleting = false;
      })),
      Effect.runPromise
    );
  }
</script>

    <div class="flex flex-col gap-4">
        <div class="flex flex-col lg:flex-row gap-4">
          <ProfileCard />
          <SocialConnections />
        </div>
        
        <div class="flex flex-col lg:flex-row gap-4">
            <MissionStats />
            <AchievementStats />
            <RewardStats />
        </div>

        <WalletStats />
        
        <Leaderboard show={10} />
    </div>
    <div class="mt-4">
      <button 
        on:click={handleDelete}
        disabled={isDeleting}
        class="bg-red-500 hover:bg-red-600 text-white px-4 py-2 rounded disabled:opacity-50"
      >
        {isDeleting ? "Deleting..." : "Delete Account"}
      </button>
      {#if error}
        <p class="text-red-500 mt-2">{error}</p>
      {/if}
    </div>

