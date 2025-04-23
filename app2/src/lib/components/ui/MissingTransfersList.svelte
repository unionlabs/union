<script lang="ts">
import { Option } from "effect"
import type { TransferList } from "@unionlabs/sdk/schema"
import Button from "./Button.svelte"
import DateTimeComponent from "./DateTimeComponent.svelte"

type Props = {
  data: Option.Option<typeof TransferList.Type>
}
</script>
