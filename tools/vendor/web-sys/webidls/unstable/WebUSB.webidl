dictionary USBDeviceFilter {
  unsigned short vendorId;
  unsigned short productId;
  octet classCode;
  octet subclassCode;
  octet protocolCode;
  DOMString serialNumber;
};

dictionary USBDeviceRequestOptions {
  required sequence<USBDeviceFilter> filters;
};

[Exposed=(DedicatedWorker,SharedWorker,Window), SecureContext]
interface USB : EventTarget {
  attribute EventHandler onconnect;
  attribute EventHandler ondisconnect;
  Promise<sequence<USBDevice>> getDevices();
  [Exposed=Window] Promise<USBDevice> requestDevice(USBDeviceRequestOptions options);
};

[Exposed=Window, SecureContext]
partial interface Navigator {
  [SameObject] readonly attribute USB usb;
};

[Exposed=(DedicatedWorker,SharedWorker), SecureContext]
partial interface WorkerNavigator {
  [SameObject] readonly attribute USB usb;
};

dictionary USBConnectionEventInit : EventInit {
    required USBDevice device;
};

[
  Exposed=(DedicatedWorker,SharedWorker,Window),
  SecureContext
]
interface USBConnectionEvent : Event {
  constructor(DOMString type, USBConnectionEventInit eventInitDict);
  [SameObject] readonly attribute USBDevice device;
};

[Exposed=(DedicatedWorker,SharedWorker,Window), SecureContext]
interface USBDevice {
  readonly attribute octet usbVersionMajor;
  readonly attribute octet usbVersionMinor;
  readonly attribute octet usbVersionSubminor;
  readonly attribute octet deviceClass;
  readonly attribute octet deviceSubclass;
  readonly attribute octet deviceProtocol;
  readonly attribute unsigned short vendorId;
  readonly attribute unsigned short productId;
  readonly attribute octet deviceVersionMajor;
  readonly attribute octet deviceVersionMinor;
  readonly attribute octet deviceVersionSubminor;
  readonly attribute DOMString? manufacturerName;
  readonly attribute DOMString? productName;
  readonly attribute DOMString? serialNumber;
  readonly attribute USBConfiguration? configuration;
  readonly attribute FrozenArray<USBConfiguration> configurations;
  readonly attribute boolean opened;
  Promise<undefined> open();
  Promise<undefined> close();
  Promise<undefined> selectConfiguration(octet configurationValue);
  Promise<undefined> claimInterface(octet interfaceNumber);
  Promise<undefined> releaseInterface(octet interfaceNumber);
  Promise<undefined> selectAlternateInterface(octet interfaceNumber, octet alternateSetting);
  Promise<USBInTransferResult> controlTransferIn(USBControlTransferParameters setup, unsigned short length);
  Promise<USBOutTransferResult> controlTransferOut(USBControlTransferParameters setup, optional BufferSource data);
  Promise<undefined> clearHalt(USBDirection direction, octet endpointNumber);
  Promise<USBInTransferResult> transferIn(octet endpointNumber, unsigned long length);
  Promise<USBOutTransferResult> transferOut(octet endpointNumber, BufferSource data);
  Promise<USBIsochronousInTransferResult> isochronousTransferIn(octet endpointNumber, sequence<unsigned long> packetLengths);
  Promise<USBIsochronousOutTransferResult> isochronousTransferOut(octet endpointNumber, BufferSource data, sequence<unsigned long> packetLengths);
  Promise<undefined> reset();
};

enum USBRequestType {
  "standard",
  "class",
  "vendor"
};

enum USBRecipient {
  "device",
  "interface",
  "endpoint",
  "other"
};

enum USBTransferStatus {
  "ok",
  "stall",
  "babble"
};

dictionary USBControlTransferParameters {
  required USBRequestType requestType;
  required USBRecipient recipient;
  required octet request;
  required unsigned short value;
  required unsigned short index;
};

[
  Exposed=(DedicatedWorker,SharedWorker,Window),
  SecureContext
]
interface USBInTransferResult {
  constructor(USBTransferStatus status, optional DataView? data);
  readonly attribute DataView? data;
  readonly attribute USBTransferStatus status;
};

[
  Exposed=(DedicatedWorker,SharedWorker,Window),
  SecureContext
]
interface USBOutTransferResult {
  constructor(USBTransferStatus status, optional unsigned long bytesWritten = 0);
  readonly attribute unsigned long bytesWritten;
  readonly attribute USBTransferStatus status;
};

[
  Exposed=(DedicatedWorker,SharedWorker,Window),
  SecureContext
]
interface USBIsochronousInTransferPacket {
  constructor(USBTransferStatus status, optional DataView? data);
  readonly attribute DataView? data;
  readonly attribute USBTransferStatus status;
};

[
  Exposed=(DedicatedWorker,SharedWorker,Window),
  SecureContext
]
interface USBIsochronousInTransferResult {
  constructor(sequence<USBIsochronousInTransferPacket> packets, optional DataView? data);
  readonly attribute DataView? data;
  readonly attribute FrozenArray<USBIsochronousInTransferPacket> packets;
};

[
  Exposed=(DedicatedWorker,SharedWorker,Window),
  SecureContext
]
interface USBIsochronousOutTransferPacket {
  constructor(USBTransferStatus status, optional unsigned long bytesWritten = 0);
  readonly attribute unsigned long bytesWritten;
  readonly attribute USBTransferStatus status;
};

[
  Exposed=(DedicatedWorker,SharedWorker,Window),
  SecureContext
]
interface USBIsochronousOutTransferResult {
  constructor(sequence<USBIsochronousOutTransferPacket> packets);
  readonly attribute FrozenArray<USBIsochronousOutTransferPacket> packets;
};

[
  Exposed=(DedicatedWorker,SharedWorker,Window),
  SecureContext
]
interface USBConfiguration {
  constructor(USBDevice device, octet configurationValue);
  readonly attribute octet configurationValue;
  readonly attribute DOMString? configurationName;
  readonly attribute FrozenArray<USBInterface> interfaces;
};

[
  Exposed=(DedicatedWorker,SharedWorker,Window),
  SecureContext
]
interface USBInterface {
  constructor(USBConfiguration configuration, octet interfaceNumber);
  readonly attribute octet interfaceNumber;
  readonly attribute USBAlternateInterface alternate;
  readonly attribute FrozenArray<USBAlternateInterface> alternates;
  readonly attribute boolean claimed;
};

[
  Exposed=(DedicatedWorker,SharedWorker,Window),
  SecureContext
]
interface USBAlternateInterface {
  constructor(USBInterface deviceInterface, octet alternateSetting);
  readonly attribute octet alternateSetting;
  readonly attribute octet interfaceClass;
  readonly attribute octet interfaceSubclass;
  readonly attribute octet interfaceProtocol;
  readonly attribute DOMString? interfaceName;
  readonly attribute FrozenArray<USBEndpoint> endpoints;
};

enum USBDirection {
  "in",
  "out"
};

enum USBEndpointType {
  "bulk",
  "interrupt",
  "isochronous"
};

[
  Exposed=(DedicatedWorker,SharedWorker,Window),
  SecureContext
]
interface USBEndpoint {
  constructor(USBAlternateInterface alternate, octet endpointNumber, USBDirection direction);
  readonly attribute octet endpointNumber;
  readonly attribute USBDirection direction;
  readonly attribute USBEndpointType type;
  readonly attribute unsigned long packetSize;
};

dictionary USBPermissionDescriptor : PermissionDescriptor {
  sequence<USBDeviceFilter> filters;
};

dictionary AllowedUSBDevice {
  required octet vendorId;
  required octet productId;
  DOMString serialNumber;
};

dictionary USBPermissionStorage {
  sequence<AllowedUSBDevice> allowedDevices = [];
};

[Exposed=(DedicatedWorker,SharedWorker,Window)]
interface USBPermissionResult : PermissionStatus {
  attribute FrozenArray<USBDevice> devices;
};