[Constructor(DOMString type, optional RTCDTMFToneChangeEventInit eventInitDict)]
interface RTCDTMFToneChangeEvent : Event {
    readonly attribute DOMString tone;
};

dictionary RTCDTMFToneChangeEventInit : EventInit {
    DOMString tone = "";
};
