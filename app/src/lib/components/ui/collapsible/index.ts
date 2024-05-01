import { Collapsible as CollapsiblePrimitive } from "bits-ui"
import Content from "./collapsible-content.svelte"

const Root = CollapsiblePrimitive.Root
const Trigger = CollapsiblePrimitive.Trigger

export {
  Root,
  Content,
  Trigger,
  //
  Root as Collapsible,
  Content as CollapsibleContent,
  Trigger as CollapsibleTrigger
}
