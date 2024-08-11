<script lang="ts">
import type { Writable } from "svelte/store"
import { cn } from "$lib/utilities/shadcn.ts"
import LockLockedIcon from "virtual:icons/lucide/lock"
import LockOpenIcon from "virtual:icons/lucide/lock-open"
import { Input } from "$lib/components/ui/input/index.ts"
import Button from "$lib/components/ui/button/button.svelte"

export let recipient: Writable<string>

let recipientInputState: "locked" | "unlocked" | "invalid" = "unlocked"

const onUnlockClick = (_event: MouseEvent) => {
  if (recipientInputState === "locked") {
    $recipient = ""
    recipientInputState = "unlocked"
  } else {
    recipientInputState = "locked"
  }
}
</script>

<div class="flex gap-2 flex-row">
  <Input
    minlength={1}
    maxlength={64}
    autocorrect="off"
    autocomplete="off"
    spellcheck="false"
    autocapitalize="none"
    bind:value={recipient}
    placeholder="Destination address"
    data-transfer-recipient-address=""
    disabled={recipientInputState === 'locked' && $recipient.length > 0}
    class={cn('font-mono placeholder:font-sans focus:ring-0 focus-visible:ring-0')}
  />
  <Button
    size="icon"
    type="button"
    variant="outline"
    name="recipient-lock"
    on:click={onUnlockClick}
  >
    <LockLockedIcon
      class={cn(recipientInputState === 'locked' && $recipient.length > 0 ? 'size-5' : 'hidden')}
    />
    <LockOpenIcon
      class={cn(recipientInputState === 'unlocked' || !$recipient.length ? 'size-5' : 'hidden')}
    />
  </Button>
</div>
