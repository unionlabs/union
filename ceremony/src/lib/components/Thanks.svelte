<script lang="ts">
import Text from "$lib/components/typography/Text.svelte"
import H1 from "$lib/components/typography/H1.svelte"
import Tweet from "$lib/components/Tweet.svelte"
import { ContributorState } from "$lib/stores/state.svelte.js"

type Props = {
  contributor: ContributorState
}

let { contributor }: Props = $props()
</script>
<div class="flex flex-col justify-center items-center gap-4">
  <H1>Thank you!</H1>
  <Text>Your contribution is completed. Thank you for securing the Union proving system!</Text>
  <Text>You can tweet the cryptographic attestation of your contribution for extra transparency.</Text>

  <Tweet/>
  <a href="/contributions" class="underline underline-offset-4 decoration-union-accent-500 text-white uppercase"
  >View contributions</a>

</div>