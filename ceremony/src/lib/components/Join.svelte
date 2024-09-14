<script lang="ts">
import H1 from "$lib/components/typography/H1.svelte"
import Button from "$lib/components/Button.svelte"
import Text from "$lib/components/typography/Text.svelte"
import { callJoinQueue } from "$lib/supabase"
import Spinner from "$lib/components/Spinner.svelte";

let code = $state("")
let loading = $state(false)

async function handleCode() {
  loading = true
  //Maybe check format etc
  const codeValid = await callJoinQueue(code)
  if (codeValid) {
    console.log("valid")
  }
}

async function joinWaitlist() {
  loading = true

}

async function handleWaitlist() {}
</script>

<!--Todo handle invite code and update contributor-->
<!--if no code, add to waitlist and update contributor-->

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
            type="button"
            onclick={handleCode}
            disabled={code.length === 0}
    >
      ENTER
    </Button>
  </form>

</div>

<Text>Or</Text>

<Button>Join the waitlist <Spinner /></Button>