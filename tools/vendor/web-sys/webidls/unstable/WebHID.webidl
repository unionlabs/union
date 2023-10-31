// GENERATED CONTENT - DO NOT EDIT
// Content was automatically extracted by Reffy into webref
// (https://github.com/w3c/webref)
// Source: WebHID API (https://wicg.github.io/webhid/)

dictionary HIDDeviceFilter {
    unsigned long vendorId;
    unsigned short productId;
    unsigned short usagePage;
    unsigned short usage;
};

dictionary HIDDeviceRequestOptions {
    required sequence<HIDDeviceFilter> filters;
};

[
    Exposed=Window,
    SecureContext
]
interface HID : EventTarget {
    attribute EventHandler onconnect;
    attribute EventHandler ondisconnect;
    Promise<sequence<HIDDevice>> getDevices();
    Promise<sequence<HIDDevice>> requestDevice(
        HIDDeviceRequestOptions options);
};

[SecureContext] partial interface Navigator {
    [SameObject] readonly attribute HID hid;
};

dictionary HIDConnectionEventInit : EventInit {
    required HIDDevice device;
};

[
    Exposed=Window,
    SecureContext
] interface HIDConnectionEvent : Event {
    constructor(DOMString type, HIDConnectionEventInit eventInitDict);
    [SameObject] readonly attribute HIDDevice device;
};

dictionary HIDInputReportEventInit : EventInit {
    required HIDDevice device;
    required octet reportId;
    required DataView data;
};

[
    Exposed=Window,
    SecureContext
] interface HIDInputReportEvent : Event {
    constructor(DOMString type, HIDInputReportEventInit eventInitDict);
    [SameObject] readonly attribute HIDDevice device;
    readonly attribute octet reportId;
    readonly attribute DataView data;
};

enum HIDUnitSystem {
    "none", "si-linear", "si-rotation", "english-linear",
    "english-rotation", "vendor-defined", "reserved"
};

dictionary HIDReportItem {
    boolean isAbsolute;
    boolean isArray;
    boolean isBufferedBytes;
    boolean isConstant;
    boolean isLinear;
    boolean isRange;
    boolean isVolatile;
    boolean hasNull;
    boolean hasPreferredState;
    boolean wrap;
    sequence<unsigned long> usages;
    unsigned long usageMinimum;
    unsigned long usageMaximum;
    unsigned short reportSize;
    unsigned short reportCount;
    byte unitExponent;
    HIDUnitSystem unitSystem;
    byte unitFactorLengthExponent;
    byte unitFactorMassExponent;
    byte unitFactorTimeExponent;
    byte unitFactorTemperatureExponent;
    byte unitFactorCurrentExponent;
    byte unitFactorLuminousIntensityExponent;
    long logicalMinimum;
    long logicalMaximum;
    long physicalMinimum;
    long physicalMaximum;
    sequence<DOMString> strings;
};

dictionary HIDReportInfo {
    octet reportId;
    sequence<HIDReportItem> items;
};

dictionary HIDCollectionInfo {
    unsigned short usagePage;
    unsigned short usage;
    octet type;
    sequence<HIDCollectionInfo> children;
    sequence<HIDReportInfo> inputReports;
    sequence<HIDReportInfo> outputReports;
    sequence<HIDReportInfo> featureReports;
};

[
    Exposed=Window,
    SecureContext
] interface HIDDevice : EventTarget {
    attribute EventHandler oninputreport;
    readonly attribute boolean opened;
    readonly attribute unsigned short vendorId;
    readonly attribute unsigned short productId;
    readonly attribute DOMString productName;
    readonly attribute FrozenArray<HIDCollectionInfo> collections;
    Promise<undefined> open();
    Promise<undefined> close();
    Promise<undefined> sendReport([EnforceRange] octet reportId, BufferSource data);
    Promise<undefined> sendFeatureReport([EnforceRange] octet reportId, BufferSource data);
    Promise<DataView> receiveFeatureReport([EnforceRange] octet reportId);
};
