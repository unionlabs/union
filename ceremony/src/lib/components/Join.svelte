<script lang="ts">
import H1 from "$lib/components/typography/H1.svelte"
import Button from "$lib/components/Button.svelte"
import Text from "$lib/components/typography/Text.svelte"
import { callJoinQueue, checkIfOpen } from "$lib/supabase"
import type { ContributorState } from "$lib/stores/state.svelte.ts"
import { toast } from "svelte-sonner"
import Code from "$lib/components/Code.svelte"

type Props = {
  contributor: ContributorState
}

let { contributor }: Props = $props()

let isOpenToPublic = $state(false)
let waitlistLoading = $state(false)

async function handleWaitlistJoin() {
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

const checkOpen = async () => {
  isOpenToPublic = await checkIfOpen()
}

$effect(() => {
  checkOpen()
})
</script>

<div class="text-center flex flex-col gap-4 items-center">
  <H1>Join the ceremony</H1>
  <form class="flex flex-col items-center">
    <Code {contributor}/>
  </form>
  <Text>Or</Text>
  <Button loading={waitlistLoading} onclick={handleWaitlistJoin} type="button">
    {isOpenToPublic ? "Join the queue" : "Join the waitlist"}
  </Button>

</div>
