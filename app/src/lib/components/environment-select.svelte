<script lang="ts">
import * as Select from "$lib/components/ui/select/index.js"

const environments = ["testnet", "mainnet"] as const

let selectedEnvironment = "testnet"
</script>

<Select.Root portal={null} open={false}>
  <Select.Trigger class="w-[105px]">
    <Select.Value class="capitalize" placeholder={selectedEnvironment} />
  </Select.Trigger>
  <Select.Content class="mx-auto text-center">
    <Select.Group>
      {#each environments as environment, index (environment)}
        <Select.Item value={environment} label={environment} class="capitalize mx-auto text-center">
          {environment}
        </Select.Item>
      {/each}
    </Select.Group>
  </Select.Content>
  <Select.Input name="environment" />
</Select.Root>
