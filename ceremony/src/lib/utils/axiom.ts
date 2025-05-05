import { AXIOM_KEY } from "$lib/constants"
import { Axiom } from "@axiomhq/js"

export const axiom = new Axiom({
  token: AXIOM_KEY,
  orgId: "union-qaca",
  onError: err => {
    console.error("AXIOM ERROR:", err)
  },
})
