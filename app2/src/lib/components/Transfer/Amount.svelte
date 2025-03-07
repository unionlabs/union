<script lang="ts">
  import Input from "$lib/components/ui/Input.svelte";
  import {getTransfer} from "../../../routes/transfer/transfer.svelte.js";

  const {transfer} = getTransfer()
</script>
<Input id="amount"
       label="amount"
       type="text"
       required={true}
       disabled={!transfer.url.asset}
       autocorrect="off"
       placeholder="0.00"
       spellcheck="false"
       autocomplete="off"
       inputmode="decimal"
       data-field="amount"
       autocapitalize="none"
       pattern="^[0-9]*[.]?[0-9]*$"
       value={transfer.url.amount}
       oninput={(event) => {
                const input = event.currentTarget;
                const value = input.value;
                if (value === '' || (/^\d*\.?\d*$/.test(value) &&
                  (value.includes('.')
                    ? value.split('.')[1].length <= (transfer.baseToken?.representations[0]?.decimals ?? 0)
                    : true)
                )) {
                  transfer.url.updateField('amount', event);
                } else {
                  input.value = transfer.amount;
                }
              }}
       class="text-center"
/>