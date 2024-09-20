<script lang="ts">
import Text from "$lib/components/typography/Text.svelte"

type Props = {
  stupid?: boolean
}

let { stupid = true } = $props()
</script>

<div class="text-center font-bold text-lg">
  {#if stupid}
    <Text>Your MPC Client is connected.</Text>
    <Text>Do not close this tab or your Terminal.</Text>
  {/if}
  <Text>Ensure you have a reliable internet connection <br> and that your computer <a
          class="underline underline-offset-4 decoration-union-accent-500"
          href="https://support.apple.com/guide/mac-help/set-sleep-and-wake-settings-mchle41a6ccd/mac" target="_blank">does
    not go to sleep</a>.
  </Text>
</div>