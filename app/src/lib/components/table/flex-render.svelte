<script lang="ts" module>
	import type { CellContext, ColumnDefTemplate, HeaderContext } from "@tanstack/table-core";

	type TData = unknown;
	type TValue = unknown;
	type TContext = unknown;
</script>

<script
	lang="ts"
	generics="TData, TValue, TContext extends HeaderContext<TData, TValue> | CellContext<TData, TValue>"
>
	import {
		RenderComponentConfig,
		RenderSnippetConfig,
	} from "./render-helpers.js";
	type Props = {
		/** The cell or header field of the current cell's column definition. */
		content?: TContext extends HeaderContext<TData, TValue>
			? ColumnDefTemplate<HeaderContext<TData, TValue>>
			: TContext extends CellContext<TData, TValue>
				? ColumnDefTemplate<CellContext<TData, TValue>>
				: never;
		/** The result of the `getContext()` function of the header or cell */
		context: TContext;
	};

	let { content, context }: Props = $props();
</script>

{#if typeof content === "string"}
	{content}
{:else if content instanceof Function}
	<!-- It's unlikely that a CellContext will be passed to a Header -->
	<!-- eslint-disable-next-line @typescript-eslint/no-explicit-any -->
	{@const result = content(context as any)}
	{#if result instanceof RenderComponentConfig}
		{@const { component: Component, props } = result}
		<Component {...props} />
	{:else if result instanceof RenderSnippetConfig}
		{@const { snippet, params } = result}
		{@render snippet(params)}
	{:else}
		{result}
	{/if}
{/if}
