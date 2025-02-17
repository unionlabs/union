<script lang="ts">
import { Switch } from "$lib/components/ui/switch"
import { Label } from "$lib/components/ui/label"
import { showDetailedTrace } from "$lib/stores/user.ts"
</script>

<div class="flex items-center space-x-2">
  <Switch bind:checked={$showDetailedTrace} id="show-ack" />
  <Label for="show-ack">Show Detailed Trace</Label>
</div>

