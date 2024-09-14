<script lang="ts">
  import H1 from "$lib/components/typography/H1.svelte"
  import Button from "$lib/components/Button.svelte"
  import Text from "$lib/components/typography/Text.svelte"
  import {callJoinQueue} from "$lib/supabase"
  import type {ContributorState} from "$lib/stores/state.svelte.ts";
  import {user} from "$lib/stores/user.svelte.ts";
  import {toast} from "svelte-sonner";

  type Props = {
    contributor: ContributorState
  }

  let {contributor}: Props = $props()

  let code = $state("")
  let codeLoading = $state(false)
  let waitlistLoading = $state(false)

  async function handleCode() {
    codeLoading = true
    const codeValid = await callJoinQueue(code)
    if (codeValid) {
      await contributor.checkAllowanceState(user.session?.user.id)
      codeLoading = false
    } else {
      toast.error('The code is not valid')
      codeLoading = false
      code = ''
    }
  }

  async function joinWaitlist() {
    waitlistLoading = true

  }
</script>

<div>
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
            class="text-md font-supermolot h-9 px-2 outline-none border-2"
    />
    <Button
            loading={codeLoading}
            type="button"
            onclick={handleCode}
            disabled={code.length === 0}
    >
      ENTER
    </Button>
  </form>

</div>

<Text>Or</Text>

<Button loading={waitlistLoading}>Join the waitlist</Button>