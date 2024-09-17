<script lang="ts">
import H1 from "$lib/components/typography/H1.svelte"
import Button from "$lib/components/Button.svelte"
import Text from "$lib/components/typography/Text.svelte"
import { callJoinQueue, checkIfOpen } from "$lib/supabase"
import type { ContributorState } from "$lib/stores/state.svelte.ts"
import { toast } from "svelte-sonner"

type Props = {
  contributor: ContributorState
}

let { contributor }: Props = $props()

let isOpenToPublic = $state(false)
let code = $state("")
let codeLoading = $state(false)
let waitlistLoading = $state(false)

async function handleJoin() {
  if (code) {
    codeLoading = true
    const normalizedCode = code.replace(/,\s*/g, "").toLowerCase()
    try {
      const codeOk = await callJoinQueue(normalizedCode)
      if (codeOk) {
        contributor.setAllowanceState("hasRedeemed")
        toast.success("Code successfully redeemed")
      } else {
        toast.error("The code is not valid")
      }
    } catch (error) {
      console.error("Error redeeming code:", error)
      toast.error("An error occurred while redeeming the code")
    } finally {
      codeLoading = false
      code = ""
    }
  } else {
    waitlistLoading = true
    try {
      await callJoinQueue(null)
      if (isOpenToPublic) {
        contributor.setAllowanceState("inQueue")
        toast.success("Successfully joined the queue")
      } else {
        contributor.setAllowanceState("inWaitlist")
        toast.success("Successfully joined the waitlist")
      }
    } catch (error) {
      console.error("Error joining waitlist:", error)
      toast.error("An error occurred while joining the waitlist")
    } finally {
      waitlistLoading = false
    }
  }
}

const checkOpen = async () => {
  isOpenToPublic = await checkIfOpen()
}

$effect(() => {
  checkOpen()
})
</script>

<div class="text-center flex flex-col gap-4">
  <H1>Join the ceremony</H1>
  <form class="flex flex-col gap-2 min-w-[355px]">
    <input
            type="text"
            autocorrect="off"
            autocomplete="off"
            spellcheck="false"
            autocapitalize="none"
            bind:value={code}
            placeholder="Secret code"
            class="text-center text-md font-supermolot h-9 px-2 outline-none border-2"
    />
    <Button
            loading={codeLoading}
            type="button"
            disabled={code.length === 0}
            onclick={handleJoin}
    >
      ENTER
    </Button>
  </form>

  <Text>Or</Text>

  <Button loading={waitlistLoading} onclick={handleJoin}>
    {isOpenToPublic ? "Join the queue" : "Join the waitlist"}
  </Button>

</div>
