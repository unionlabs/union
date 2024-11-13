import {
  type RowData,
  type TableState,
  type TableOptions,
  type TableOptionsResolved,
  _createTable as createTable,
} from "@tanstack/table-core"

/**
 * Creates a reactive TanStack table object for Svelte.
 * @param options Table options to create the table with.
 * @returns A reactive table object.
 * @example
 * ```svelte
 * <script>
 *   const table = createSvelteTable({ ... })
 * </script>
 *
 * <table>
 *   <thead>
 *     {#each table.getHeaderGroups() as headerGroup}
 *       <tr>
 *         {#each headerGroup.headers as header}
 *           <th colspan={header.colSpan}>
 *         	   <FlexRender content={header.column.columnDef.header} context={header.getContext()} />
 *         	 </th>
 *         {/each}
 *       </tr>
 *     {/each}
 *   </thead>
 * 	 <!-- ... -->
 * </table>
 * ```
 */
export function createSvelteTable<TData extends RowData>(options: TableOptions<TData>) {
  const resolvedOptions: TableOptionsResolved<TData> = mergeObjects(
    {
      state: {},
      // biome-ignore lint/suspicious/noEmptyBlockStatements: <explanation>
      onStateChange() {},
      renderFallbackValue: null,
      mergeOptions: (
        defaultOptions: TableOptions<TData>,
        options: Partial<TableOptions<TData>>
      ) => {
        return mergeObjects(defaultOptions, options)
      }
    },
    options
  )

  const table = createTable(resolvedOptions)
  let state = $state<Partial<TableState>>(table.initialState)

  function updateOptions() {
    table.setOptions(prev => {
      return mergeObjects(prev, options, {
        state: mergeObjects(state, options.state || {}),

        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        onStateChange: (updater: any) => {
          if (updater instanceof Function) state = updater(state)
          else state = mergeObjects(state, updater)

          options.onStateChange?.(updater)
        }
      })
    })
  }

  updateOptions()

  $effect.pre(() => {
    updateOptions()
  })

  return table
}

/**
 * Merges objects together while keeping their getters alive.
 * Taken from SolidJS: {@link https://github.com/solidjs/solid/blob/24abc825c0996fd2bc8c1de1491efe9a7e743aff/packages/solid/src/server/rendering.ts#L82-L115}
 */
function mergeObjects<T>(source: T): T
function mergeObjects<T, U>(source: T, source1: U): T & U
function mergeObjects<T, U, V>(source: T, source1: U, source2: V): T & U & V
function mergeObjects<T, U, V, W>(source: T, source1: U, source2: V, source3: W): T & U & V & W
// eslint-disable-next-line @typescript-eslint/no-explicit-any
function mergeObjects(...sources: any): any {
  const target = {}
  for (let i = 0; i < sources.length; i++) {
    let source = sources[i]
    if (typeof source === "function") source = source()
    if (source) {
      const descriptors = Object.getOwnPropertyDescriptors(source)
      for (const key in descriptors) {
        if (key in target) continue
        Object.defineProperty(target, key, {
          enumerable: true,
          get() {
            for (let i = sources.length - 1; i >= 0; i--) {
              let s = sources[i]
              if (typeof s === "function") s = s()
              const v = s?.[key]
              if (v !== undefined) return v
            }
          }
        })
      }
    }
  }
  return target
}
