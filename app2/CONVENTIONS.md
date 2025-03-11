# Project Conventions

## Type Safety

- Use Effect's type-safe alternatives where possible:

  ```typescript
  // Instead of:
  let value: string | null | undefined

  // Use:
  let value: Option<string>
  ```

- Never use `try {} catch {}` blocks. always use Effect. When dealing with unsafe functions from libraries, use `Effect.tryPromie(() => somePromiseFn())`

- Do NOT do `Option.isSome(Option.fromNullable(T))` if `T` is already an `Option` type

## Styling

- Use Tailwind's zinc color palette instead of gray for neutral colors
  ```css
  /* Instead of: */
  bg-gray-100 dark:bg-gray-800

  /* Use: */
  bg-zinc-100 dark:bg-zinc-800
  ```
- never use `mx-auto` to style. use flexbox instead.
- If you want to apply a gap between components, use `flex gap-*`, rather than `space-x-*`

## UI Components

- All UI components must accept a `class` prop for styling customization
- Use the `cn()` utility for combining class names
- Example pattern:
  ```svelte
  type Props = HTMLAttributes<HTMLDivElement> & {
    children: Snippet
    class?: string
  }

  const { children, class: className = "", ...rest }: Props = $props()

  const classes = cn(
    "base classes",
    className
  )
  ```

## Svelte 5 Syntax

- Always use Svelte 5's reactive syntax
- Never use `$:` statements or `$store` syntax
- Use `$state()`, `$derived()`, and `$effect()` instead
- Don't legacy syntax like `on:input` / `on:click`, use `oninput` and `onclick` instead.

### Props

Use Svelte 5's prop syntax:

```svelte
interface Props {
  title: string
  count: number
}

let { title, count }: Props = $props()
```

### Children

Use Svelte 5's render children statement with Snippet type:

```svelte
type Props = {
  children: Snippet
}

const { children }: Props = $props()

{@render children()}
```

### State Management

```svelte
// Instead of:
$: total = count * 2
let $store

// Use:
let total = $derived(count * 2)
let store = $state(initialValue)
```

## Array syntax

use `Array<T>` rather than `T[]`
