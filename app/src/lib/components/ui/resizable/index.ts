import Handle from "./resizable-handle.svelte"
import PaneGroup from "./resizable-pane-group.svelte"
import { Pane, type PaneProps, type PaneGroupProps } from "paneforge"

export {
  type PaneProps,
  type PaneGroupProps,
  PaneGroup,
  Pane,
  Handle,
  //
  PaneGroup as ResizablePaneGroup,
  Pane as ResizablePane,
  Handle as ResizableHandle
}
