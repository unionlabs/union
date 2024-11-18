<script lang="ts">
  import clsx from 'clsx'

  interface Props {
    rows: Array<Array<string>>
    headers: Array<string>
  }

  let { rows, headers }: Props = $props()
</script>

<div class="px-4 sm:px-6 lg:px-8 flex justify-center w-full">
  <div class="mt-8 flow-root">
    <div class="overflow-x-auto">
      <div class="inline-block min-w-full align-middle border rounded-sm border-neutral-400">
        <table class="divide-y divide-gray-300 tabular-nums">
          <thead class="w-full min-w-full">
            <tr class="divide-x divide-gray-200 w-full">
              {#each headers as header, index}
                <th
                  scope="col"
                  class={clsx([
                    `py-3.5 text-left text-md font-semibold`,
                    index === 0
                      ? 'pl-4'
                      : index === headers.length - 1
                        ? 'pl-4 text-center'
                        : 'px-4',
                  ])}
                >
                  {header}
                </th>
              {/each}
            </tr>
          </thead>
          <tbody class="divide-y divide-gray-200 w-full min-w-full">
            {#each rows as row, index}
              <tr class="w-full">
                {#each row as cell, index}
                  <td class="whitespace-nowrap p-4 text-sm font-medium">{cell}</td>
                {/each}
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    </div>
  </div>
</div>

<style lang="postcss"></style>
