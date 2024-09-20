<script lang="ts">
import H1 from "$lib/components/typography/H1.svelte"
import Button from "$lib/components/Button.svelte"
import Text from "$lib/components/typography/Text.svelte"
import { callJoinQueue, checkIfOpen } from "$lib/supabase"
import type { ContributorState } from "$lib/stores/state.svelte.ts"
import { toast } from "svelte-sonner"
import Code from "$lib/components/Code.svelte"
import H2 from "$lib/components/typography/H2.svelte"

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
  <Text>Have an invite? Enter your code below.</Text>
  <form class="flex flex-col items-center">
    <Code {contributor}/>
  </form>
  <Text class="py-8">Or</Text>
  <H2>Don't have an invite?</H2>
  <Text>You can join the waitlist now to get priority access when the ceremony opens.</Text>
  <Button class="bg-transparent text-white hover:text-white border-2 border-white hover:bg-neutral-800" loading={waitlistLoading} onclick={handleWaitlistJoin} type="button">
    {isOpenToPublic ? "Join the queue" : "Join the waitlist"}
  </Button>
</div>
