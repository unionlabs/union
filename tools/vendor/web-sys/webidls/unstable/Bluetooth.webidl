dictionary BluetoothDataFilterInit {
  BufferSource dataPrefix;
  BufferSource mask;
};

dictionary BluetoothLEScanFilterInit {
  sequence<BluetoothServiceUUID> services;
  DOMString name;
  DOMString namePrefix;
  // Maps unsigned shorts to BluetoothDataFilters.
  object manufacturerData;
  // Maps BluetoothServiceUUIDs to BluetoothDataFilters.
  object serviceData;
};

dictionary RequestDeviceOptions {
  sequence<BluetoothLEScanFilterInit> filters;
  sequence<BluetoothServiceUUID> optionalServices = [];
  boolean acceptAllDevices = false;
};

[Exposed=Window, SecureContext]
interface Bluetooth : EventTarget {
  Promise<boolean> getAvailability();
  attribute EventHandler onavailabilitychanged;
  [SameObject]
  readonly attribute BluetoothDevice? referringDevice;
  Promise<sequence<BluetoothDevice>> getDevices();
  Promise<BluetoothDevice> requestDevice(RequestDeviceOptions options);
};

Bluetooth includes BluetoothDeviceEventHandlers;
Bluetooth includes CharacteristicEventHandlers;
Bluetooth includes ServiceEventHandlers;

dictionary BluetoothPermissionDescriptor : PermissionDescriptor {
  DOMString deviceId;
  // These match RequestDeviceOptions.
  sequence<BluetoothLEScanFilterInit> filters;
  sequence<BluetoothServiceUUID> optionalServices = [];
  boolean acceptAllDevices = false;
};

dictionary AllowedBluetoothDevice {
  required DOMString deviceId;
  required boolean mayUseGATT;
  // An allowedServices of "all" means all services are allowed.
  required (DOMString or sequence<UUID>) allowedServices;
};
dictionary BluetoothPermissionStorage {
  required sequence<AllowedBluetoothDevice> allowedDevices;
};

[Exposed=Window]
interface BluetoothPermissionResult : PermissionStatus {
  attribute FrozenArray<BluetoothDevice> devices;
};

[
  Exposed=Window,
  SecureContext
]
interface ValueEvent : Event {
  constructor(DOMString type, optional ValueEventInit initDict = {});
  readonly attribute any value;
};

dictionary ValueEventInit : EventInit {
  any value = null;
};

[Exposed=Window, SecureContext]
interface BluetoothDevice : EventTarget {
  readonly attribute DOMString id;
  readonly attribute DOMString? name;
  readonly attribute BluetoothRemoteGATTServer? gatt;

  Promise<undefined> watchAdvertisements(
      optional WatchAdvertisementsOptions options = {});
  readonly attribute boolean watchingAdvertisements;
};
BluetoothDevice includes BluetoothDeviceEventHandlers;
BluetoothDevice includes CharacteristicEventHandlers;
BluetoothDevice includes ServiceEventHandlers;

dictionary WatchAdvertisementsOptions {
  AbortSignal signal;
};

[Exposed=Window, SecureContext]
interface BluetoothManufacturerDataMap {
  readonly maplike<unsigned short, DataView>;
};
[Exposed=Window, SecureContext]
interface BluetoothServiceDataMap {
  readonly maplike<UUID, DataView>;
};
[
  Exposed=Window,
  SecureContext
]
interface BluetoothAdvertisingEvent : Event {
  constructor(DOMString type, BluetoothAdvertisingEventInit init);
  [SameObject]
  readonly attribute BluetoothDevice device;
  readonly attribute FrozenArray<UUID> uuids;
  readonly attribute DOMString? name;
  readonly attribute unsigned short? appearance;
  readonly attribute byte? txPower;
  readonly attribute byte? rssi;
  [SameObject]
  readonly attribute BluetoothManufacturerDataMap manufacturerData;
  [SameObject]
  readonly attribute BluetoothServiceDataMap serviceData;
};
dictionary BluetoothAdvertisingEventInit : EventInit {
  required BluetoothDevice device;
  sequence<(DOMString or unsigned long)> uuids;
  DOMString name;
  unsigned short appearance;
  byte txPower;
  byte rssi;
  BluetoothManufacturerDataMap manufacturerData;
  BluetoothServiceDataMap serviceData;
};

[Exposed=Window, SecureContext]
interface BluetoothRemoteGATTServer {
  [SameObject]
  readonly attribute BluetoothDevice device;
  readonly attribute boolean connected;
  Promise<BluetoothRemoteGATTServer> connect();
  undefined disconnect();
  Promise<BluetoothRemoteGATTService> getPrimaryService(BluetoothServiceUUID service);
  Promise<sequence<BluetoothRemoteGATTService>>
    getPrimaryServices(optional BluetoothServiceUUID service);
};

[Exposed=Window, SecureContext]
interface BluetoothRemoteGATTService : EventTarget {
  [SameObject]
  readonly attribute BluetoothDevice device;
  readonly attribute UUID uuid;
  readonly attribute boolean isPrimary;
  Promise<BluetoothRemoteGATTCharacteristic>
    getCharacteristic(BluetoothCharacteristicUUID characteristic);
  Promise<sequence<BluetoothRemoteGATTCharacteristic>>
    getCharacteristics(optional BluetoothCharacteristicUUID characteristic);
  Promise<BluetoothRemoteGATTService>
    getIncludedService(BluetoothServiceUUID service);
  Promise<sequence<BluetoothRemoteGATTService>>
    getIncludedServices(optional BluetoothServiceUUID service);
};
BluetoothRemoteGATTService includes CharacteristicEventHandlers;
BluetoothRemoteGATTService includes ServiceEventHandlers;

[Exposed=Window, SecureContext]
interface BluetoothRemoteGATTCharacteristic : EventTarget {
  [SameObject]
  readonly attribute BluetoothRemoteGATTService service;
  readonly attribute UUID uuid;
  readonly attribute BluetoothCharacteristicProperties properties;
  readonly attribute DataView? value;
  Promise<BluetoothRemoteGATTDescriptor> getDescriptor(BluetoothDescriptorUUID descriptor);
  Promise<sequence<BluetoothRemoteGATTDescriptor>>
    getDescriptors(optional BluetoothDescriptorUUID descriptor);
  Promise<DataView> readValue();
  Promise<undefined> writeValue(BufferSource value);
  Promise<undefined> writeValueWithResponse(BufferSource value);
  Promise<undefined> writeValueWithoutResponse(BufferSource value);
  Promise<BluetoothRemoteGATTCharacteristic> startNotifications();
  Promise<BluetoothRemoteGATTCharacteristic> stopNotifications();
};
BluetoothRemoteGATTCharacteristic includes CharacteristicEventHandlers;

[Exposed=Window, SecureContext]
interface BluetoothCharacteristicProperties {
  readonly attribute boolean broadcast;
  readonly attribute boolean read;
  readonly attribute boolean writeWithoutResponse;
  readonly attribute boolean write;
  readonly attribute boolean notify;
  readonly attribute boolean indicate;
  readonly attribute boolean authenticatedSignedWrites;
  readonly attribute boolean reliableWrite;
  readonly attribute boolean writableAuxiliaries;
};

[Exposed=Window, SecureContext]
interface BluetoothRemoteGATTDescriptor {
  [SameObject]
  readonly attribute BluetoothRemoteGATTCharacteristic characteristic;
  readonly attribute UUID uuid;
  readonly attribute DataView? value;
  Promise<DataView> readValue();
  Promise<undefined> writeValue(BufferSource value);
};

[SecureContext]
interface mixin CharacteristicEventHandlers {
  attribute EventHandler oncharacteristicvaluechanged;
};

[SecureContext]
interface mixin BluetoothDeviceEventHandlers {
  attribute EventHandler onadvertisementreceived;
  attribute EventHandler ongattserverdisconnected;
};

[SecureContext]
interface mixin ServiceEventHandlers {
  attribute EventHandler onserviceadded;
  attribute EventHandler onservicechanged;
  attribute EventHandler onserviceremoved;
};

typedef DOMString UUID;

[Exposed=Window]
interface BluetoothUUID {
  static UUID getService((DOMString or unsigned long) name);
  static UUID getCharacteristic((DOMString or unsigned long) name);
  static UUID getDescriptor((DOMString or unsigned long) name);

  static UUID canonicalUUID([EnforceRange] unsigned long alias);
};

typedef (DOMString or unsigned long) BluetoothServiceUUID;
typedef (DOMString or unsigned long) BluetoothCharacteristicUUID;
typedef (DOMString or unsigned long) BluetoothDescriptorUUID;

[SecureContext]
partial interface Navigator {
  [SameObject]
  readonly attribute Bluetooth? bluetooth;
};
