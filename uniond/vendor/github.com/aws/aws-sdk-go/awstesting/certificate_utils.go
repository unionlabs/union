package awstesting

import (
	"bytes"
	"crypto/rand"
	"crypto/rsa"
	"crypto/tls"
	"crypto/x509"
	"crypto/x509/pkix"
	"encoding/pem"
	"fmt"
	"io/ioutil"
	"math/big"
	"net"
	"net/http"
	"net/http/httptest"
	"os"
	"strings"
	"time"
)

var (
	// TLSBundleCA is the CA PEM
	TLSBundleCA []byte

	// TLSBundleCert is the Server PEM
	TLSBundleCert []byte

	// TLSBundleKey is the Server private key PEM
	TLSBundleKey []byte

	// ClientTLSCert is the Client PEM
	ClientTLSCert []byte

	// ClientTLSKey is the Client private key PEM
	ClientTLSKey []byte
)

func init() {
	caPEM, _, caCert, caPrivKey, err := generateRootCA()
	if err != nil {
		panic("failed to generate testing root CA, " + err.Error())
	}
	TLSBundleCA = caPEM

	serverCertPEM, serverCertPrivKeyPEM, err := generateLocalCert(caCert, caPrivKey)
	if err != nil {
		panic("failed to generate testing server cert, " + err.Error())
	}
	TLSBundleCert = serverCertPEM
	TLSBundleKey = serverCertPrivKeyPEM

	clientCertPEM, clientCertPrivKeyPEM, err := generateLocalCert(caCert, caPrivKey)
	if err != nil {
		panic("failed to generate testing client cert, " + err.Error())
	}
	ClientTLSCert = clientCertPEM
	ClientTLSKey = clientCertPrivKeyPEM
}

func generateRootCA() (
	caPEM, caPrivKeyPEM []byte, caCert *x509.Certificate, caPrivKey *rsa.PrivateKey, err error,
) {
	caCert = &x509.Certificate{
		SerialNumber: big.NewInt(42),
		Subject: pkix.Name{
			Country:      []string{"US"},
			Organization: []string{"AWS SDK for Go Test Certificate"},
			CommonName:   "Test Root CA",
		},
		NotBefore: time.Now().Add(-time.Minute),
		NotAfter:  time.Now().AddDate(1, 0, 0),
		KeyUsage:  x509.KeyUsageCertSign | x509.KeyUsageCRLSign | x509.KeyUsageDigitalSignature,
		ExtKeyUsage: []x509.ExtKeyUsage{
			x509.ExtKeyUsageClientAuth,
			x509.ExtKeyUsageServerAuth,
		},
		BasicConstraintsValid: true,
		IsCA:                  true,
	}

	// Create CA private and public key
	caPrivKey, err = rsa.GenerateKey(rand.Reader, 4096)
	if err != nil {
		return nil, nil, nil, nil, fmt.Errorf("failed generate CA RSA key, %w", err)
	}

	// Create CA certificate
	caBytes, err := x509.CreateCertificate(rand.Reader, caCert, caCert, &caPrivKey.PublicKey, caPrivKey)
	if err != nil {
		return nil, nil, nil, nil, fmt.Errorf("failed generate CA certificate, %w", err)
	}

	// PEM encode CA certificate and private key
	var caPEMBuf bytes.Buffer
	pem.Encode(&caPEMBuf, &pem.Block{
		Type:  "CERTIFICATE",
		Bytes: caBytes,
	})

	var caPrivKeyPEMBuf bytes.Buffer
	pem.Encode(&caPrivKeyPEMBuf, &pem.Block{
		Type:  "RSA PRIVATE KEY",
		Bytes: x509.MarshalPKCS1PrivateKey(caPrivKey),
	})

	return caPEMBuf.Bytes(), caPrivKeyPEMBuf.Bytes(), caCert, caPrivKey, nil
}

func generateLocalCert(parentCert *x509.Certificate, parentPrivKey *rsa.PrivateKey) (
	certPEM, certPrivKeyPEM []byte, err error,
) {
	cert := &x509.Certificate{
		SerialNumber: big.NewInt(42),
		Subject: pkix.Name{
			Country:      []string{"US"},
			Organization: []string{"AWS SDK for Go Test Certificate"},
			CommonName:   "Test Root CA",
		},
		IPAddresses: []net.IP{
			net.IPv4(127, 0, 0, 1),
			net.IPv6loopback,
		},
		NotBefore: time.Now().Add(-time.Minute),
		NotAfter:  time.Now().AddDate(1, 0, 0),
		ExtKeyUsage: []x509.ExtKeyUsage{
			x509.ExtKeyUsageClientAuth,
			x509.ExtKeyUsageServerAuth,
		},
		KeyUsage: x509.KeyUsageDigitalSignature,
	}

	// Create server private and public key
	certPrivKey, err := rsa.GenerateKey(rand.Reader, 4096)
	if err != nil {
		return nil, nil, fmt.Errorf("failed to generate server RSA private key, %w", err)
	}

	// Create server certificate
	certBytes, err := x509.CreateCertificate(rand.Reader, cert, parentCert, &certPrivKey.PublicKey, parentPrivKey)
	if err != nil {
		return nil, nil, fmt.Errorf("failed to generate server certificate, %w", err)
	}

	// PEM encode certificate and private key
	var certPEMBuf bytes.Buffer
	pem.Encode(&certPEMBuf, &pem.Block{
		Type:  "CERTIFICATE",
		Bytes: certBytes,
	})

	var certPrivKeyPEMBuf bytes.Buffer
	pem.Encode(&certPrivKeyPEMBuf, &pem.Block{
		Type:  "RSA PRIVATE KEY",
		Bytes: x509.MarshalPKCS1PrivateKey(certPrivKey),
	})

	return certPEMBuf.Bytes(), certPrivKeyPEMBuf.Bytes(), nil
}

// NewTLSClientCertServer creates a new HTTP test server initialize to require
// HTTP clients authenticate with TLS client certificates.
func NewTLSClientCertServer(handler http.Handler) (*httptest.Server, error) {
	server := httptest.NewUnstartedServer(handler)

	if server.TLS == nil {
		server.TLS = &tls.Config{}
	}
	server.TLS.ClientAuth = tls.RequireAndVerifyClientCert

	if server.TLS.ClientCAs == nil {
		server.TLS.ClientCAs = x509.NewCertPool()
	}
	certPem := append(ClientTLSCert, ClientTLSKey...)
	if ok := server.TLS.ClientCAs.AppendCertsFromPEM(certPem); !ok {
		return nil, fmt.Errorf("failed to append client certs")
	}

	return server, nil
}

// CreateClientTLSCertFiles returns a set of temporary files for the client
// certificate and key files.
func CreateClientTLSCertFiles() (cert, key string, err error) {
	cert, err = createTmpFile(ClientTLSCert)
	if err != nil {
		return "", "", err
	}

	key, err = createTmpFile(ClientTLSKey)
	if err != nil {
		return "", "", err
	}

	return cert, key, nil
}

func availableLocalAddr(ip string) (string, error) {
	l, err := net.Listen("tcp", ip+":0")
	if err != nil {
		return "", err
	}
	defer l.Close()

	return l.Addr().String(), nil
}

// CreateTLSServer will create the TLS server on an open port using the
// certificate and key. The address will be returned that the server is running on.
func CreateTLSServer(cert, key string, mux *http.ServeMux) (string, error) {
	addr, err := availableLocalAddr("127.0.0.1")
	if err != nil {
		return "", err
	}

	if mux == nil {
		mux = http.NewServeMux()
		mux.HandleFunc("/", func(w http.ResponseWriter, r *http.Request) {})
	}

	go func() {
		if err := http.ListenAndServeTLS(addr, cert, key, mux); err != nil {
			panic(err)
		}
	}()

	for i := 0; i < 60; i++ {
		if _, err := http.Get("https://" + addr); err != nil && !strings.Contains(err.Error(), "connection refused") {
			break
		}

		time.Sleep(1 * time.Second)
	}

	return "https://" + addr, nil
}

// CreateTLSBundleFiles returns the temporary filenames for the certificate
// key, and CA PEM content. These files should be deleted when no longer
// needed. CleanupTLSBundleFiles can be used for this cleanup.
func CreateTLSBundleFiles() (cert, key, ca string, err error) {
	cert, err = createTmpFile(TLSBundleCert)
	if err != nil {
		return "", "", "", err
	}

	key, err = createTmpFile(TLSBundleKey)
	if err != nil {
		return "", "", "", err
	}

	ca, err = createTmpFile(TLSBundleCA)
	if err != nil {
		return "", "", "", err
	}

	return cert, key, ca, nil
}

// CleanupTLSBundleFiles takes variadic list of files to be deleted.
func CleanupTLSBundleFiles(files ...string) error {
	for _, file := range files {
		if err := os.Remove(file); err != nil {
			return err
		}
	}

	return nil
}

func createTmpFile(b []byte) (string, error) {
	bundleFile, err := ioutil.TempFile(os.TempDir(), "aws-sdk-go-session-test")
	if err != nil {
		return "", err
	}

	_, err = bundleFile.Write(b)
	if err != nil {
		return "", err
	}

	defer bundleFile.Close()
	return bundleFile.Name(), nil
}
