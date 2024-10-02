import { Axiom } from "@axiomhq/js"
import { AXIOM_KEY } from "$lib/constants"

export const axiom = new Axiom({
  token: AXIOM_KEY,
  orgId: "union-qaca",
  onError: err => {
    console.error("AXIOM ERROR:", err)
  }
})
