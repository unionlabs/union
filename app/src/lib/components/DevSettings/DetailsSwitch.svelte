<script lang="ts">
import { showTokenDetails } from "$lib/stores/user.ts"
import { Label } from "$lib/components/ui/label/index.ts"
import { Switch } from "$lib/components/ui/switch/index.ts"
</script>

<div class="flex items-center space-x-2">
  <Switch bind:checked={$showTokenDetails} id="show-token-details" />
  <Label for="show-token-details">Show Token Details</Label>
</div>