<script lang="ts">
import Modal from "./ui/Modal.svelte"
import AppErrors from "$lib/components/layout/AppErrors/index.svelte"

type Props = {
  isOpen: boolean
  onClose: () => void
}

const { isOpen, onClose }: Props = $props()
</script>

<Modal persistent={true} {isOpen} {onClose} class="w-full max-w-4xl">
  <h2 class="text-xl font-bold mb-4">Errors</h2>
  <AppErrors />
</Modal>
