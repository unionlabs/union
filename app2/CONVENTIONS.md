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

## Error Handling

Error handling is what distinguishes production-grade apps from toys.

Be extremely mindful and considerate of how you implement errors. 

Always use Effect's Error system. If you have a function that can fail, it must be an `Effect`.
Create new [Tagged Errors](https://effect.website/docs/error-management/expected-errors/) and `yield` them whenever an effect errors.

Never do things like this:

```typescript
catch: error => (error instanceof Error ? error : new Error("Unknown error"))
```

Never use the keywords `try`, `catch`, and `throw`.

When you deal with a _library function outside of our control_, you can use `Effect.try` to wrap that function into an Effect. If you do this, put the wrapped function as a standalone utility in `ts-sdk/`. Do not put a big codeblock inside of an `Effect.try`, it is not meant as a substitute for

```typescript
try {
  // lots of logic
} catch (e) {
  // did not see this coming i guess ill just panic bc i have no idea what went wrong here
}
```

### Never lose details of an error

It is very bad to do something like this

```typescript
const myError; // coming from somewhere
return "My operation went wrong! ${myError.message}"
```

If you do this, all previously provided details of that error will be gone and you will only see a message. The error object may have many more details. Even worse is this:

```typescript
const myError; // coming from somewhere
return "my operation went wrong!"
```

Instead, create a new `TaggedError` if `myError` is not already a yieldable tagged effect error. Extract all details of the extracted error, and `yield*` your new error type. This will make sure that your new error is tracked at the type system. For example:

```typescript
export class ReadContractError extends Data.TaggedError("ReadContractError")<{
  myErrorSpecificDetail: number,
  cause: unknown // the unknown error we received
}> {}
```

then, when you get an error, `yield*` your new tagged error.

```typescript
const myError; // coming from somewhere
yield* new ReadContractError({ myErrorSpecificDetails: 4, cause: extractErrorDetails(myError) })
```

> "But I cannot yield* an error in <context in which you encounter an error>!!!"

In that case, your function definition (context) is wrong. If your operation can error, it is an `Effect`.

### Propagate error details all the way to the top

As an extension of the previous section, do make sure that the full error object with all of its details is always exposed to the user in the UI. It is not acceptable to leave out any of the details here. We cannot expect a user to open the browser console. You can use `<ErrorComponent/>` if you don't know how to display an error. Do not only show `error.message`, show the full error.


### Use `extractErrorDetails`

When wrapping incoming unknown Errors into `Data.TaggedError`s, use [`extractErrorDetails`](https://github.com/unionlabs/union/blob/15a294f6ebbb06bd5ad404212c48c564dcd909b4/ts-sdk/src/utils/extract-error-details.ts#L1) to extract all details from that external error.



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

## Use of existing components

- Don't create your own cards, instead, use the `components/ui/Card.svelte`
- When creating a label, use `components/ui/Label.svelte`
- When creating skeletons, use `components/ui/Skeleton.svelte`
- When displaying an error somewhere, use `components/model/ErrorComponent.svelte`
- When displaying dates or times somewhere, use `components/ui/DateTimeComponent.svelte`

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

- Do not create self-closing div tags `<div />`, instead do `<div></div>`

## Svelte 5 Syntax

- Always use Svelte 5's reactive syntax
- Never use `$:` statements or `$store` syntax
- Use `$state()`, `$derived()`, and `$effect()` instead
- Don't legacy syntax like `on:input` / `on:click`, use `oninput` and `onclick` instead.
- Don't use self closing div tags (`<div/>`). instead do `<div></div>`

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
