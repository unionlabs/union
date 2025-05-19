<script lang="ts">
import Button from "$lib/components/ui/Button.svelte"
import type {
  AchievementError,
  AuthenticationError,
  CategoryError,
  ChainError,
  DashboardUnknownException,
  EmailLinkError,
  LeaderboardError,
  MissionError,
  ProviderLinkError,
  RewardError,
  SupabaseClientError,
  SupabaseError,
  WalletError,
} from "$lib/dashboard/errors"
import { Match, pipe } from "effect"
import { slide } from "svelte/transition"
import BaselineCloseIcon from "../icons/BaselineCloseIcon.svelte"
import SharpErrorOutlineIcon from "../icons/SharpErrorOutlineIcon.svelte"
import Modal from "../ui/Modal.svelte"

interface Props {
  error:
    | AuthenticationError
    | SupabaseClientError
    | SupabaseError
    | DashboardUnknownException
    | AchievementError
    | LeaderboardError
    | MissionError
    | RewardError
    | WalletError
    | ProviderLinkError
    | EmailLinkError
    | ChainError
    | CategoryError
  onClose?: () => void
}

let { error, onClose }: Props = $props()
let showDetails = $state(false)

const getUserFriendlyMessage = pipe(
  Match.type<Props["error"]>(),
  Match.tags({
    AuthenticationError: (x) => x.message || "Authentication failed. Please try signing in again.",
    ProviderLinkError: (x) =>
      `The ${x.provider} account is already linked to another user. Please use a different account.`,
    EmailLinkError: (x) => `The email ${x.email} is already linked to another account.`,
    SupabaseClientError: () =>
      "Unable to connect to the server. Please check your internet connection.",
    SupabaseError: (x) =>
      x.error?.message ?? "An error occurred while communicating with the database.",
    DashboardUnknownException: (x) => x.message,
    AchievementError: (x) => `Failed to ${x.operation} achievements. Please try again.`,
    LeaderboardError: (x) =>
      x.operation === "loadLevels"
        ? "Unable to load level data. Please try again."
        : "Unable to load leaderboard data. Please try again.",
    MissionError: (x) => `Failed to ${x.operation} missions. Please try again.`,
    RewardError: (x) => `Failed to ${x.operation} rewards. Please try again.`,
    WalletError: (x) => `Failed to ${x.operation} wallet. Please try again.`,
    ChainError: () => "Unable to load chain data. Please try again.",
    CategoryError: () => "Unable to load category data. Please try again.",
  }),
  Match.orElse(() => "An unexpected error occurred."),
)
</script>

<div class="rounded bg-red-500/10 border border-red-500/50 overflow-hidden">
  <div class="flex justify-between items-center gap-2 p-3">
    <div class="flex items-center gap-2">
      <SharpErrorOutlineIcon class="text-red-500 size-4 min-w-4" />
      <p class="text-sm text-red-200">{getUserFriendlyMessage(error)}</p>
    </div>

    <div class="flex gap-2 shrink-0">
      <Button
        variant="secondary"
        onclick={() => showDetails = true}
        class="text-sm"
      >
        Details
      </Button>
      {#if onClose}
        <Button
          variant="outline"
          onclick={onClose}
          class="p-1"
        >
          <BaselineCloseIcon class="size-4" />
        </Button>
      {/if}
    </div>
  </div>
</div>

<Modal
  isOpen={showDetails}
  onClose={() => showDetails = false}
  class="w-full max-w-2xl"
>
  <div
    class="overflow-auto mt-6"
    in:slide
    out:slide|local={{ delay: 0 }}
  >
    <section class="mt-4">
      <h3 class="text-lg font-bold">Error Type</h3>
      <pre>{error._tag}</pre>
    </section>

    {#if error.cause}
      <section class="mt-4">
        <h3 class="text-lg font-bold">Cause</h3>
        <pre class="text-sm whitespace-pre-wrap">{JSON.stringify(error.cause, null, 2)}</pre>
      </section>
    {/if}

    {#if error._tag === "SupabaseError" && error.error}
      <section class="mt-4">
        <h3 class="text-lg font-bold">Database Error Details</h3>
        <pre class="text-sm whitespace-pre-wrap">{JSON.stringify(error.error, null, 2)}</pre>
      </section>
    {/if}

    {#if error._tag === "DashboardUnknownException"}
      <section class="mt-4">
        <h3 class="text-lg font-bold">Error Message</h3>
        <p>{error.message}</p>
        {#if error.cause}
          <pre class="text-sm mt-2 whitespace-pre-wrap">{JSON.stringify(error.cause, null, 2)}</pre>
        {/if}
      </section>
    {/if}
  </div>
</Modal>
