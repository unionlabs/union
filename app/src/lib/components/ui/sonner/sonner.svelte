<script lang="ts">
import { Toaster as Sonner, type ToasterProps as SonnerProps } from "svelte-sonner"
import { mode } from "mode-watcher"
	interface Props {
		[key: string]: any
	}

	let { ...rest }: Props = $props();

type $$Props = SonnerProps
</script>

<Sonner
	theme={$mode}
	class="toaster group"
	toastOptions={{
		classes: {
			toast: "group toast rounded-none border group-[.toaster]:bg-card group-[.toaster]:text-foreground group-[.toaster]:border-border group-[.toaster]:shadow-lg",
			description: "group-[.toast]:text-muted-foreground",
			actionButton: "group-[.toast]:bg-primary group-[.toast]:text-primary-foreground",
			cancelButton: "group-[.toast]:bg-muted group-[.toast]:text-muted-foreground",
		},
	}}
	{...rest}
/>
