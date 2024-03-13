import { Tabs as TabsPrimitive } from "bits-ui"

const Root = TabsPrimitive.Root

export {
  Root,
  //
  Root as Tabs
}

export {
  default as Content,
  default as TabsContent
} from "./tabs-content.svelte"
export {
  default as Trigger,
  default as TabsTrigger
} from "./tabs-trigger.svelte"
export { default as List, default as TabsList } from "./tabs-list.svelte"
