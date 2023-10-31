/* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this file,
* You can obtain one at http://mozilla.org/MPL/2.0/. */

/**
* This dictionary holds the parameters sent to the network worker.
*/
dictionary NetworkCommandOptions
{
  long id = 0;                        // opaque id.
  DOMString cmd = "";                 // the command name.
  DOMString ifname;                   // for "removeNetworkRoute", "setDNS",
                                      //     "setDefaultRouteAndDNS", "removeDefaultRoute"
                                      //     "addHostRoute", "removeHostRoute"
                                      //     "removeHostRoutes".
  DOMString ip;                       // for "removeNetworkRoute", "setWifiTethering".
  unsigned long prefixLength;         // for "removeNetworkRoute".
  DOMString domain;                   // for "setDNS"
  sequence<DOMString> dnses;          // for "setDNS", "setDefaultRouteAndDNS".
  DOMString gateway;                  // for "addSecondaryRoute", "removeSecondaryRoute".
  sequence<DOMString> gateways;       // for "setDefaultRouteAndDNS", "removeDefaultRoute".
  DOMString mode;                     // for "setWifiOperationMode".
  boolean report;                     // for "setWifiOperationMode".
  boolean enabled;                    // for "setDhcpServer".
  DOMString wifictrlinterfacename;    // for "setWifiTethering".
  DOMString internalIfname;           // for "setWifiTethering".
  DOMString externalIfname;           // for "setWifiTethering".
  boolean enable;                     // for "setWifiTethering".
  DOMString ssid;                     // for "setWifiTethering".
  DOMString security;                 // for "setWifiTethering".
  DOMString key;                      // for "setWifiTethering".
  DOMString prefix;                   // for "setWifiTethering", "setDhcpServer".
  DOMString link;                     // for "setWifiTethering", "setDhcpServer".
  sequence<DOMString> interfaceList;  // for "setWifiTethering".
  DOMString wifiStartIp;              // for "setWifiTethering".
  DOMString wifiEndIp;                // for "setWifiTethering".
  DOMString usbStartIp;               // for "setWifiTethering".
  DOMString usbEndIp;                 // for "setWifiTethering".
  DOMString dns1;                     // for "setWifiTethering".
  DOMString dns2;                     // for "setWifiTethering".
  long long threshold;                // for "setNetworkInterfaceAlarm",
                                      //     "enableNetworkInterfaceAlarm".
  DOMString startIp;                  // for "setDhcpServer".
  DOMString endIp;                    // for "setDhcpServer".
  DOMString serverIp;                 // for "setDhcpServer".
  DOMString maskLength;               // for "setDhcpServer".
  DOMString preInternalIfname;        // for "updateUpStream".
  DOMString preExternalIfname;        // for "updateUpStream".
  DOMString curInternalIfname;        // for "updateUpStream".
  DOMString curExternalIfname;        // for "updateUpStream".

  long ipaddr;                        // for "ifc_configure".
  long mask;                          // for "ifc_configure".
  long gateway_long;                  // for "ifc_configure".
  long dns1_long;                     // for "ifc_configure".
  long dns2_long;                     // for "ifc_configure".

  long mtu;                           // for "setMtu".
};

/**
* This dictionary holds the parameters sent back to NetworkService.js.
*/
dictionary NetworkResultOptions
{
  long id = 0;                        // opaque id.
  boolean ret = false;                // for sync command.
  boolean broadcast = false;          // for netd broadcast message.
  DOMString topic = "";               // for netd broadcast message.
  DOMString reason = "";              // for netd broadcast message.

  long resultCode = 0;                // for all commands.
  DOMString resultReason = "";        // for all commands.
  boolean error = false;              // for all commands.

  boolean enable = false;             // for "setWifiTethering", "setUSBTethering"
                                      //     "enableUsbRndis".
  boolean result = false;             // for "enableUsbRndis".
  boolean success = false;            // for "setDhcpServer".
  DOMString curExternalIfname = "";   // for "updateUpStream".
  DOMString curInternalIfname = "";   // for "updateUpStream".

  DOMString reply = "";               // for "command".
  DOMString route = "";               // for "ifc_get_default_route".
  DOMString ipaddr_str = "";          // The following are for the result of
                                      // dhcp_do_request.
  DOMString gateway_str = "";
  DOMString dns1_str = "";
  DOMString dns2_str = "";
  DOMString mask_str = "";
  DOMString server_str = "";
  DOMString vendor_str = "";
  long      lease = 0;
  long      prefixLength = 0;
  long      mask = 0;
  long      ipaddr = 0;
  long      gateway = 0;
  long      dns1 = 0;
  long      dns2 = 0;
  long      server = 0;

  DOMString netId = "";               // for "getNetId".

  sequence<DOMString> interfaceList;  // for "getInterfaceList".

  DOMString flag = "down";            // for "getInterfaceConfig".
  DOMString macAddr = "";             // for "getInterfaceConfig".
  DOMString ipAddr = "";              // for "getInterfaceConfig".
};
