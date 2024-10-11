// Copyright 2015 The go-ethereum Authors
// This file is part of the go-ethereum library.
//
// The go-ethereum library is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// The go-ethereum library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with the go-ethereum library. If not, see <http://www.gnu.org/licenses/>.

package nat

import (
	"fmt"
	"io"
	"net"
	"net/http"
	"os"
	"runtime"
	"strings"
	"testing"

	"github.com/huin/goupnp/httpu"
)

func TestUPNP_DDWRT(t *testing.T) {
	if runtime.GOOS == "windows" {
		t.Skipf("disabled to avoid firewall prompt")
	}

	dev := &fakeIGD{
		t: t,
		ssdpResp: "HTTP/1.1 200 OK\r\n" +
			"Cache-Control: max-age=300\r\n" +
			"Date: Sun, 10 May 2015 10:05:33 GMT\r\n" +
			"Ext: \r\n" +
			"Location: http://{{listenAddr}}/InternetGatewayDevice.xml\r\n" +
			"Server: POSIX UPnP/1.0 DD-WRT Linux/V24\r\n" +
			"ST: urn:schemas-upnp-org:device:WANConnectionDevice:1\r\n" +
			"USN: uuid:CB2471CC-CF2E-9795-8D9C-E87B34C16800::urn:schemas-upnp-org:device:WANConnectionDevice:1\r\n" +
			"\r\n",
		httpResps: map[string]string{
			"GET /InternetGatewayDevice.xml": `
				 <?xml version="1.0"?>
				 <root xmlns="urn:schemas-upnp-org:device-1-0">
					 <specVersion>
						 <major>1</major>
						 <minor>0</minor>
					 </specVersion>
					 <device>
						 <deviceType>urn:schemas-upnp-org:device:InternetGatewayDevice:1</deviceType>
						 <manufacturer>DD-WRT</manufacturer>
						 <manufacturerURL>http://www.dd-wrt.com</manufacturerURL>
						 <modelDescription>Gateway</modelDescription>
						 <friendlyName>Asus RT-N16:DD-WRT</friendlyName>
						 <modelName>Asus RT-N16</modelName>
						 <modelNumber>V24</modelNumber>
						 <serialNumber>0000001</serialNumber>
						 <modelURL>http://www.dd-wrt.com</modelURL>
						 <UDN>uuid:A13AB4C3-3A14-E386-DE6A-EFEA923A06FE</UDN>
						 <serviceList>
							 <service>
								 <serviceType>urn:schemas-upnp-org:service:Layer3Forwarding:1</serviceType>
								 <serviceId>urn:upnp-org:serviceId:L3Forwarding1</serviceId>
								 <SCPDURL>/x_layer3forwarding.xml</SCPDURL>
								 <controlURL>/control?Layer3Forwarding</controlURL>
								 <eventSubURL>/event?Layer3Forwarding</eventSubURL>
							 </service>
						 </serviceList>
						 <deviceList>
							 <device>
								 <deviceType>urn:schemas-upnp-org:device:WANDevice:1</deviceType>
								 <friendlyName>WANDevice</friendlyName>
								 <manufacturer>DD-WRT</manufacturer>
								 <manufacturerURL>http://www.dd-wrt.com</manufacturerURL>
								 <modelDescription>Gateway</modelDescription>
								 <modelName>router</modelName>
								 <modelURL>http://www.dd-wrt.com</modelURL>
								 <UDN>uuid:48FD569B-F9A9-96AE-4EE6-EB403D3DB91A</UDN>
								 <serviceList>
									 <service>
										 <serviceType>urn:schemas-upnp-org:service:WANCommonInterfaceConfig:1</serviceType>
										 <serviceId>urn:upnp-org:serviceId:WANCommonIFC1</serviceId>
										 <SCPDURL>/x_wancommoninterfaceconfig.xml</SCPDURL>
										 <controlURL>/control?WANCommonInterfaceConfig</controlURL>
										 <eventSubURL>/event?WANCommonInterfaceConfig</eventSubURL>
									 </service>
								 </serviceList>
								 <deviceList>
									 <device>
										 <deviceType>urn:schemas-upnp-org:device:WANConnectionDevice:1</deviceType>
										 <friendlyName>WAN Connection Device</friendlyName>
										 <manufacturer>DD-WRT</manufacturer>
										 <manufacturerURL>http://www.dd-wrt.com</manufacturerURL>
										 <modelDescription>Gateway</modelDescription>
										 <modelName>router</modelName>
										 <modelURL>http://www.dd-wrt.com</modelURL>
										 <UDN>uuid:CB2471CC-CF2E-9795-8D9C-E87B34C16800</UDN>
										 <serviceList>
											 <service>
												 <serviceType>urn:schemas-upnp-org:service:WANIPConnection:1</serviceType>
												 <serviceId>urn:upnp-org:serviceId:WANIPConn1</serviceId>
												 <SCPDURL>/x_wanipconnection.xml</SCPDURL>
												 <controlURL>/control?WANIPConnection</controlURL>
												 <eventSubURL>/event?WANIPConnection</eventSubURL>
											 </service>
										 </serviceList>
									 </device>
								 </deviceList>
							 </device>
							 <device>
								 <deviceType>urn:schemas-upnp-org:device:LANDevice:1</deviceType>
								 <friendlyName>LANDevice</friendlyName>
								 <manufacturer>DD-WRT</manufacturer>
								 <manufacturerURL>http://www.dd-wrt.com</manufacturerURL>
								 <modelDescription>Gateway</modelDescription>
								 <modelName>router</modelName>
								 <modelURL>http://www.dd-wrt.com</modelURL>
								 <UDN>uuid:04021998-3B35-2BDB-7B3C-99DA4435DA09</UDN>
								 <serviceList>
									 <service>
										 <serviceType>urn:schemas-upnp-org:service:LANHostConfigManagement:1</serviceType>
										 <serviceId>urn:upnp-org:serviceId:LANHostCfg1</serviceId>
										 <SCPDURL>/x_lanhostconfigmanagement.xml</SCPDURL>
										 <controlURL>/control?LANHostConfigManagement</controlURL>
										 <eventSubURL>/event?LANHostConfigManagement</eventSubURL>
									 </service>
								 </serviceList>
							 </device>
						 </deviceList>
						 <presentationURL>http://{{listenAddr}}</presentationURL>
					 </device>
				 </root>
			`,
			// The response to our GetNATRSIPStatus call. This
			// particular implementation has a bug where the elements
			// inside u:GetNATRSIPStatusResponse are not properly
			// namespaced.
			"POST /control?WANIPConnection": `
				 <s:Envelope xmlns:s="http://schemas.xmlsoap.org/soap/envelope/" s:encodingStyle="http://schemas.xmlsoap.org/soap/encoding/">
				 <s:Body>
				 <u:GetNATRSIPStatusResponse xmlns:u="urn:schemas-upnp-org:service:WANIPConnection:1">
				 <NewRSIPAvailable>0</NewRSIPAvailable>
				 <NewNATEnabled>1</NewNATEnabled>
				 </u:GetNATRSIPStatusResponse>
				 </s:Body>
				 </s:Envelope>
			`,
		},
	}
	if err := dev.listen(); err != nil {
		t.Skipf("cannot listen: %v", err)
	}
	dev.serve()
	defer dev.close()

	// Attempt to discover the fake device.
	discovered := discoverUPnP()
	if discovered == nil {
		if os.Getenv("CI") != "" {
			t.Fatalf("not discovered")
		} else {
			t.Skipf("UPnP not discovered (known issue, see https://github.com/ethereum/go-ethereum/issues/21476)")
		}
	}
	upnp, _ := discovered.(*upnp)
	if upnp.service != "IGDv1-IP1" {
		t.Errorf("upnp.service mismatch: got %q, want %q", upnp.service, "IGDv1-IP1")
	}
	wantURL := "http://" + dev.listener.Addr().String() + "/InternetGatewayDevice.xml"
	if upnp.dev.URLBaseStr != wantURL {
		t.Errorf("upnp.dev.URLBaseStr mismatch: got %q, want %q", upnp.dev.URLBaseStr, wantURL)
	}
}

// fakeIGD presents itself as a discoverable UPnP device which sends
// canned responses to HTTPU and HTTP requests.
type fakeIGD struct {
	t *testing.T // for logging

	listener      net.Listener
	mcastListener *net.UDPConn

	// This should be a complete HTTP response (including headers).
	// It is sent as the response to any sspd packet. Any occurrence
	// of "{{listenAddr}}" is replaced with the actual TCP listen
	// address of the HTTP server.
	ssdpResp string
	// This one should contain XML payloads for all requests
	// performed. The keys contain method and path, e.g. "GET /foo/bar".
	// As with ssdpResp, "{{listenAddr}}" is replaced with the TCP
	// listen address.
	httpResps map[string]string
}

// httpu.Handler
func (dev *fakeIGD) ServeMessage(r *http.Request) {
	dev.t.Logf(`HTTPU request %s %s`, r.Method, r.RequestURI)
	conn, err := net.Dial("udp4", r.RemoteAddr)
	if err != nil {
		fmt.Printf("reply Dial error: %v", err)
		return
	}
	defer conn.Close()
	io.WriteString(conn, dev.replaceListenAddr(dev.ssdpResp))
}

// http.Handler
func (dev *fakeIGD) ServeHTTP(w http.ResponseWriter, r *http.Request) {
	if resp, ok := dev.httpResps[r.Method+" "+r.RequestURI]; ok {
		dev.t.Logf(`HTTP request "%s %s" --> %d`, r.Method, r.RequestURI, 200)
		io.WriteString(w, dev.replaceListenAddr(resp))
	} else {
		dev.t.Logf(`HTTP request "%s %s" --> %d`, r.Method, r.RequestURI, 404)
		w.WriteHeader(http.StatusNotFound)
	}
}

func (dev *fakeIGD) replaceListenAddr(resp string) string {
	return strings.ReplaceAll(resp, "{{listenAddr}}", dev.listener.Addr().String())
}

func (dev *fakeIGD) listen() (err error) {
	if dev.listener, err = net.Listen("tcp", "127.0.0.1:0"); err != nil {
		return err
	}
	laddr := &net.UDPAddr{IP: net.ParseIP("239.255.255.250"), Port: 1900}
	if dev.mcastListener, err = net.ListenMulticastUDP("udp", nil, laddr); err != nil {
		dev.listener.Close()
		return err
	}
	return nil
}

func (dev *fakeIGD) serve() {
	go httpu.Serve(dev.mcastListener, dev)
	go http.Serve(dev.listener, dev)
}

func (dev *fakeIGD) close() {
	dev.mcastListener.Close()
	dev.listener.Close()
}
