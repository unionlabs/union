// invalid widl
// interface PluginTag;

[Constructor(DOMString type, optional HiddenPluginEventInit eventInit), ChromeOnly]
interface HiddenPluginEvent : Event
{
  readonly attribute PluginTag? tag;
};

dictionary HiddenPluginEventInit : EventInit
{
  PluginTag? tag = null;
};
