<script lang="ts">
import AchievementStats from "$lib/dashboard/components/AchievementStats.svelte"
import Leaderboard from "$lib/dashboard/components/Leaderboard.svelte"
import MissionStats from "$lib/dashboard/components/MissionStats.svelte"
import ProfileCard from "$lib/dashboard/components/ProfileCard.svelte"
import RewardStats from "$lib/dashboard/components/RewardStats.svelte"
import SocialConnections from "$lib/dashboard/components/SocialConnections.svelte"
import WalletStats from "$lib/dashboard/components/WalletStats.svelte"
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
