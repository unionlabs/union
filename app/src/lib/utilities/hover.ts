export function hover(node: {
  onmousemove: (event: any) => void
  getElementsByClassName: (arg0: string) => any
}) {
  node.onmousemove = event => {
    for (const card of node.getElementsByClassName("card")) {
      const rect = card.getBoundingClientRect()
      const x = event.clientX - rect.left
      const y = event.clientY - rect.top

      card.style.setProperty("--mouse-x", `${x}px`)
      card.style.setProperty("--mouse-y", `${y}px`)
    }
  }
}
