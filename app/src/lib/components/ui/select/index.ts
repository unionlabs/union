import { Select as SelectPrimitive } from "bits-ui"

import GroupHeading from "./select-group-heading.svelte"
import Item from "./select-item.svelte"
import Content from "./select-content.svelte"
import Trigger from "./select-trigger.svelte"
import Separator from "./select-separator.svelte"
import ScrollDownButton from "./select-scroll-down-button.svelte"
import ScrollUpButton from "./select-scroll-up-button.svelte"

const Root = SelectPrimitive.Root
const Group = SelectPrimitive.Group

export {
  Root,
  Group,
  GroupHeading,
  Item,
  Content,
  Trigger,
  Separator,
  ScrollDownButton,
  ScrollUpButton,
  //
  Root as Select,
  Group as SelectGroup,
  GroupHeading as SelectGroupHeading,
  Item as SelectItem,
  Content as SelectContent,
  Trigger as SelectTrigger,
  Separator as SelectSeparator,
  ScrollDownButton as SelectScrollDownButton,
  ScrollUpButton as SelectScrollUpButton
}
