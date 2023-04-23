// Taken from taipei-torrent.
// Just enough UPnP to be able to forward ports
// For more information, see: http://www.upnp-hacks.org/upnp.html
package upnp

// TODO: use syscalls to get actual ourIP, see issue #712

import (
	"bytes"
	"encoding/xml"
	"errors"
	"fmt"
	"io"
	"net"
	"net/http"
	"strconv"
	"strings"
	"time"
)

type upnpNAT struct {
	serviceURL string
	ourIP      string
	urnDomain  string
}

// protocol is either "udp" or "tcp"
type NAT interface {
	GetExternalAddress() (addr net.IP, err error)
	AddPortMapping(
		protocol string,
		externalPort,
		internalPort int,
		description string,
		timeout int) (mappedExternalPort int, err error)
	DeletePortMapping(protocol string, externalPort, internalPort int) (err error)
}

func Discover() (nat NAT, err error) {
	ssdp, err := net.ResolveUDPAddr("udp4", "239.255.255.250:1900")
	if err != nil {
		return
	}
	conn, err := net.ListenPacket("udp4", ":0")
	if err != nil {
		return
	}
	socket := conn.(*net.UDPConn)
	defer socket.Close()

	if err := socket.SetDeadline(time.Now().Add(3 * time.Second)); err != nil {
		return nil, err
	}

	st := "InternetGatewayDevice:1"

	buf := bytes.NewBufferString(
		"M-SEARCH * HTTP/1.1\r\n" +
			"HOST: 239.255.255.250:1900\r\n" +
			"ST: ssdp:all\r\n" +
			"MAN: \"ssdp:discover\"\r\n" +
			"MX: 2\r\n\r\n")
	message := buf.Bytes()
	answerBytes := make([]byte, 1024)
	for i := 0; i < 3; i++ {
		_, err = socket.WriteToUDP(message, ssdp)
		if err != nil {
			return
		}
		var n int
		_, _, err = socket.ReadFromUDP(answerBytes)
		if err != nil {
			return
		}
		for {
			n, _, err = socket.ReadFromUDP(answerBytes)
			if err != nil {
				break
			}
			answer := string(answerBytes[0:n])
			if !strings.Contains(answer, st) {
				continue
			}
			// HTTP header field names are case-insensitive.
			// http://www.w3.org/Protocols/rfc2616/rfc2616-sec4.html#sec4.2
			locString := "\r\nlocation:"
			answer = strings.ToLower(answer)
			locIndex := strings.Index(answer, locString)
			if locIndex < 0 {
				continue
			}
			loc := answer[locIndex+len(locString):]
			endIndex := strings.Index(loc, "\r\n")
			if endIndex < 0 {
				continue
			}
			locURL := strings.TrimSpace(loc[0:endIndex])
			var serviceURL, urnDomain string
			serviceURL, urnDomain, err = getServiceURL(locURL)
			if err != nil {
				return
			}
			var ourIP net.IP
			ourIP, err = localIPv4()
			if err != nil {
				return
			}
			nat = &upnpNAT{serviceURL: serviceURL, ourIP: ourIP.String(), urnDomain: urnDomain}
			return
		}
	}
	err = errors.New("upnp port discovery failed")
	return nat, err
}

type Envelope struct {
	XMLName xml.Name `xml:"http://schemas.xmlsoap.org/soap/envelope/ Envelope"`
	Soap    *SoapBody
}
type SoapBody struct {
	XMLName    xml.Name `xml:"http://schemas.xmlsoap.org/soap/envelope/ Body"`
	ExternalIP *ExternalIPAddressResponse
}

type ExternalIPAddressResponse struct {
	XMLName   xml.Name `xml:"GetExternalIPAddressResponse"`
	IPAddress string   `xml:"NewExternalIPAddress"`
}

type ExternalIPAddress struct {
	XMLName xml.Name `xml:"NewExternalIPAddress"`
	IP      string
}

type Service struct {
	ServiceType string `xml:"serviceType"`
	ControlURL  string `xml:"controlURL"`
}

type DeviceList struct {
	Device []Device `xml:"device"`
}

type ServiceList struct {
	Service []Service `xml:"service"`
}

type Device struct {
	XMLName     xml.Name    `xml:"device"`
	DeviceType  string      `xml:"deviceType"`
	DeviceList  DeviceList  `xml:"deviceList"`
	ServiceList ServiceList `xml:"serviceList"`
}

type Root struct {
	Device Device
}

func getChildDevice(d *Device, deviceType string) *Device {
	dl := d.DeviceList.Device
	for i := 0; i < len(dl); i++ {
		if strings.Contains(dl[i].DeviceType, deviceType) {
			return &dl[i]
		}
	}
	return nil
}

func getChildService(d *Device, serviceType string) *Service {
	sl := d.ServiceList.Service
	for i := 0; i < len(sl); i++ {
		if strings.Contains(sl[i].ServiceType, serviceType) {
			return &sl[i]
		}
	}
	return nil
}

func localIPv4() (net.IP, error) {
	tt, err := net.Interfaces()
	if err != nil {
		return nil, err
	}
	for _, t := range tt {
		aa, err := t.Addrs()
		if err != nil {
			return nil, err
		}
		for _, a := range aa {
			ipnet, ok := a.(*net.IPNet)
			if !ok {
				continue
			}
			v4 := ipnet.IP.To4()
			if v4 == nil || v4[0] == 127 { // loopback address
				continue
			}
			return v4, nil
		}
	}
	return nil, errors.New("cannot find local IP address")
}

func getServiceURL(rootURL string) (url, urnDomain string, err error) {
	r, err := http.Get(rootURL) //nolint: gosec
	if err != nil {
		return
	}
	defer r.Body.Close()

	if r.StatusCode >= 400 {
		err = errors.New(string(rune(r.StatusCode)))
		return
	}
	var root Root
	err = xml.NewDecoder(r.Body).Decode(&root)
	if err != nil {
		return
	}
	a := &root.Device
	if !strings.Contains(a.DeviceType, "InternetGatewayDevice:1") {
		err = errors.New("no InternetGatewayDevice")
		return
	}
	b := getChildDevice(a, "WANDevice:1")
	if b == nil {
		err = errors.New("no WANDevice")
		return
	}
	c := getChildDevice(b, "WANConnectionDevice:1")
	if c == nil {
		err = errors.New("no WANConnectionDevice")
		return
	}
	d := getChildService(c, "WANIPConnection:1")
	if d == nil {
		// Some routers don't follow the UPnP spec, and put WanIPConnection under WanDevice,
		// instead of under WanConnectionDevice
		d = getChildService(b, "WANIPConnection:1")

		if d == nil {
			err = errors.New("no WANIPConnection")
			return
		}
	}
	// Extract the domain name, which isn't always 'schemas-upnp-org'
	urnDomain = strings.Split(d.ServiceType, ":")[1]
	url = combineURL(rootURL, d.ControlURL)
	return url, urnDomain, err
}

func combineURL(rootURL, subURL string) string {
	protocolEnd := "://"
	protoEndIndex := strings.Index(rootURL, protocolEnd)
	a := rootURL[protoEndIndex+len(protocolEnd):]
	rootIndex := strings.Index(a, "/")
	return rootURL[0:protoEndIndex+len(protocolEnd)+rootIndex] + subURL
}

func soapRequest(url, function, message, domain string) (r *http.Response, err error) {
	fullMessage := "<?xml version=\"1.0\" ?>" +
		"<s:Envelope xmlns:s=\"http://schemas.xmlsoap.org/soap/envelope/\" " +
		"s:encodingStyle=\"http://schemas.xmlsoap.org/soap/encoding/\">\r\n" +
		"<s:Body>" + message + "</s:Body></s:Envelope>"

	req, err := http.NewRequest("POST", url, strings.NewReader(fullMessage))
	if err != nil {
		return nil, err
	}
	req.Header.Set("Content-Type", "text/xml ; charset=\"utf-8\"")
	req.Header.Set("User-Agent", "Darwin/10.0.0, UPnP/1.0, MiniUPnPc/1.3")
	// req.Header.Set("Transfer-Encoding", "chunked")
	req.Header.Set("SOAPAction", "\"urn:"+domain+":service:WANIPConnection:1#"+function+"\"")
	req.Header.Set("Connection", "Close")
	req.Header.Set("Cache-Control", "no-cache")
	req.Header.Set("Pragma", "no-cache")

	// log.Stderr("soapRequest ", req)

	r, err = http.DefaultClient.Do(req)
	if err != nil {
		return nil, err
	}
	/*if r.Body != nil {
	    defer r.Body.Close()
	}*/

	if r.StatusCode >= 400 {
		// log.Stderr(function, r.StatusCode)
		err = errors.New("error " + strconv.Itoa(r.StatusCode) + " for " + function)
		r = nil
		return
	}
	return r, err
}

type statusInfo struct {
	externalIPAddress string
}

func (n *upnpNAT) getExternalIPAddress() (info statusInfo, err error) {

	message := "<u:GetExternalIPAddress xmlns:u=\"urn:" + n.urnDomain + ":service:WANIPConnection:1\">\r\n" +
		"</u:GetExternalIPAddress>"

	var response *http.Response
	response, err = soapRequest(n.serviceURL, "GetExternalIPAddress", message, n.urnDomain)
	if response != nil {
		defer response.Body.Close()
	}
	if err != nil {
		return
	}
	var envelope Envelope
	data, err := io.ReadAll(response.Body)
	if err != nil {
		return
	}
	reader := bytes.NewReader(data)
	err = xml.NewDecoder(reader).Decode(&envelope)
	if err != nil {
		return
	}

	info = statusInfo{envelope.Soap.ExternalIP.IPAddress}

	if err != nil {
		return
	}

	return info, err
}

// GetExternalAddress returns an external IP. If GetExternalIPAddress action
// fails or IP returned is invalid, GetExternalAddress returns an error.
func (n *upnpNAT) GetExternalAddress() (addr net.IP, err error) {
	info, err := n.getExternalIPAddress()
	if err != nil {
		return
	}
	addr = net.ParseIP(info.externalIPAddress)
	if addr == nil {
		err = fmt.Errorf("failed to parse IP: %v", info.externalIPAddress)
	}
	return
}

func (n *upnpNAT) AddPortMapping(
	protocol string,
	externalPort,
	internalPort int,
	description string,
	timeout int) (mappedExternalPort int, err error) {
	// A single concatenation would break ARM compilation.
	message := "<u:AddPortMapping xmlns:u=\"urn:" + n.urnDomain + ":service:WANIPConnection:1\">\r\n" +
		"<NewRemoteHost></NewRemoteHost><NewExternalPort>" + strconv.Itoa(externalPort)
	message += "</NewExternalPort><NewProtocol>" + protocol + "</NewProtocol>"
	message += "<NewInternalPort>" + strconv.Itoa(internalPort) + "</NewInternalPort>" +
		"<NewInternalClient>" + n.ourIP + "</NewInternalClient>" +
		"<NewEnabled>1</NewEnabled><NewPortMappingDescription>"
	message += description +
		"</NewPortMappingDescription><NewLeaseDuration>" + strconv.Itoa(timeout) +
		"</NewLeaseDuration></u:AddPortMapping>"

	var response *http.Response
	response, err = soapRequest(n.serviceURL, "AddPortMapping", message, n.urnDomain)
	if response != nil {
		defer response.Body.Close()
	}
	if err != nil {
		return
	}

	// TODO: check response to see if the port was forwarded
	// log.Println(message, response)
	// JAE:
	// body, err := io.ReadAll(response.Body)
	// fmt.Println(string(body), err)
	mappedExternalPort = externalPort
	_ = response
	return mappedExternalPort, err
}

func (n *upnpNAT) DeletePortMapping(protocol string, externalPort, internalPort int) (err error) {

	message := "<u:DeletePortMapping xmlns:u=\"urn:" + n.urnDomain + ":service:WANIPConnection:1\">\r\n" +
		"<NewRemoteHost></NewRemoteHost><NewExternalPort>" + strconv.Itoa(externalPort) +
		"</NewExternalPort><NewProtocol>" + protocol + "</NewProtocol>" +
		"</u:DeletePortMapping>"

	var response *http.Response
	response, err = soapRequest(n.serviceURL, "DeletePortMapping", message, n.urnDomain)
	if response != nil {
		defer response.Body.Close()
	}
	if err != nil {
		return
	}

	// TODO: check response to see if the port was deleted
	// log.Println(message, response)
	_ = response
	return
}
