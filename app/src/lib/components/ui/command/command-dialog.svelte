<script lang="ts">
import type {
  Command as CommandPrimitive,
  Dialog as DialogPrimitive,
  WithoutChildrenOrChild
} from "bits-ui"
import type { Snippet } from "svelte"
import Command from "./command.svelte"
import * as Dialog from "$lib/components/ui/dialog/index.js"

let {
  open = $bindable(false),
  ref = $bindable(null),
  value = $bindable(""),
  children,
  ...restProps
}: WithoutChildrenOrChild<DialogPrimitive.RootProps> &
  WithoutChildrenOrChild<CommandPrimitive.RootProps> & {
    children: Snippet
  } = $props()
</script>

<Dialog.Root bind:open {...restProps}>
	<Dialog.Content class="overflow-hidden p-0 shadow-lg">
		<Command
			class="[&_[data-command-group]:not([hidden])_~[data-command-group]]:pt-0 [&_[data-command-group]]:px-2 [&_[data-command-input-wrapper]_svg]:h-5 [&_[data-command-input-wrapper]_svg]:w-5 [&_[data-command-input]]:h-12 [&_[data-command-item]]:px-2 [&_[data-command-item]]:py-3 [&_[data-command-item]_svg]:h-5 [&_[data-command-item]_svg]:w-5"
			{...restProps}
			bind:value
			bind:ref
			{children}
		/>
	</Dialog.Content>
</Dialog.Root>
