import { Axiom } from "@axiomhq/js"

export const axiom = new Axiom({
  token: import.meta.env.VITE_AXIOM_KEY,
  orgId: "union-qaca",
  onError: err => {
    console.error("AXIOM ERROR:", err)
  }
})
