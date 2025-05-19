<script lang="ts">
import BaselineCloseIcon from "$lib/components/icons/BaselineCloseIcon.svelte"
import SharpErrorOutlineIcon from "$lib/components/icons/SharpErrorOutlineIcon.svelte"
import Button from "$lib/components/ui/Button.svelte"
import Modal from "$lib/components/ui/Modal.svelte"
import type {
  AccountError,
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
    | AccountError
  onClose?: () => void
}

let { error, onClose }: Props = $props()
let showDetails = $state(false)

const getUserFriendlyMessage = pipe(
  Match.type<Props["error"]>(),
  Match.tags({
    AuthenticationError: (x) => x.message || "Authentication failed. Please try signing in again.",
    ProviderLinkError: (x) =>
      x.message
      || `The ${x.provider} account is already linked to another user. Please use a different account.`,
    EmailLinkError: (x) =>
      x.message || `The email ${x.email} is already linked to another account.`,
    SupabaseClientError: (x) =>
      x.message || "Unable to connect to the server. Please check your internet connection.",
    SupabaseError: (x) =>
      x.message || x.error?.message || "An error occurred while communicating with the database.",
    DashboardUnknownException: (x) => x.message || "An unknown error occurred.",
    AchievementError: (x) =>
      x.message || `Failed to ${x.operation} achievements. Please try again.`,
    LeaderboardError: (x) =>
      x.message || `Failed to ${x.operation} leaderboard data. Please try again.`,
    MissionError: (x) => x.message || `Failed to ${x.operation} missions. Please try again.`,
    RewardError: (x) => x.message || `Failed to ${x.operation} rewards. Please try again.`,
    WalletError: (x) => x.message || `Failed to ${x.operation} wallet. Please try again.`,
    ChainError: (x) => x.message || `Failed to ${x.operation} chain data. Please try again.`,
    CategoryError: (x) => x.message || `Failed to ${x.operation} category data. Please try again.`,
    AccountError: (x) => x.message || `Failed to ${x.operation} account. Please try again.`,
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
        variant="danger"
        onclick={() => showDetails = true}
        class="text-sm"
      >Details</Button>
      {#if onClose}
        <Button
          variant="outline"
          onclick={onClose}
          class="p-1"
        ><BaselineCloseIcon class="size-4" /></Button>
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
      <h3 class="text-lg font-bold">Error Details</h3>
      <div class="mt-2 space-y-2">
        <p><span class="font-semibold">Type:</span> {error._tag}</p>
        <p><span class="font-semibold">Operation:</span> {error.operation}</p>
        {#if error.message}
          <p><span class="font-semibold">Message:</span> {error.message}</p>
        {/if}
        {#if error.cause}
          <div>
            <p class="font-semibold">Cause:</p>
            <pre
              class="text-sm mt-1 whitespace-pre-wrap"
            >{JSON.stringify(error.cause, null, 2)}</pre>
          </div>
        {/if}
        {#if error._tag === "SupabaseError" && error.error}
          <div>
            <p class="font-semibold">Database Error:</p>
            <pre
              class="text-sm mt-1 whitespace-pre-wrap"
            >{JSON.stringify(error.error, null, 2)}</pre>
          </div>
        {/if}
      </div>
    </section>
  </div>
</Modal>
