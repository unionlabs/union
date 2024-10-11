**Generating certificates and keys for testing mTLS-S2A**

Create root CA
```
openssl req -x509 -sha256 -days 7305 -newkey rsa:2048 -keyout mds_root_key.pem -out mds_root_cert.pem
```

Generate private keys for server and client
```
openssl genrsa -out mds_server_key.pem 2048
openssl genrsa -out mds_client_key.pem 2048
```

Generate CSRs for server and client
```
openssl req -key mds_server_key.pem -new -out mds_server.csr -config config.cnf
openssl req -key mds_client_key.pem -new -out mds_client.csr -config config.cnf
```

Look at CSR
```
openssl req -noout -text -in mds_server.csr
openssl req -noout -text -in mds_client.csr
```

Sign CSRs for server and client
```
openssl x509 -req -CA mds_root_cert.pem -CAkey mds_root_key.pem -in mds_server.csr -out mds_server_cert.pem -days 7305 -extfile config.cnf -extensions req_ext
openssl x509 -req -CA mds_root_cert.pem -CAkey mds_root_key.pem -in mds_client.csr -out mds_client_cert.pem -days 7305
```

Look at signed certs
```
openssl x509 -in mds_server_cert.pem -noout -text
openssl x509 -in mds_client_cert.pem -noout -text
```

Verify server and client certs using root CA
```
openssl verify -CAfile mds_root_cert.pem mds_server_cert.pem
openssl verify -CAfile mds_root_cert.pem mds_client_cert.pem
```

Create self-signed key/cert to test failure case
```
openssl genrsa -out self_signed_key.pem 2048
openssl req -new -key self_signed_key.pem -out self_signed.csr
openssl x509 -req -in self_signed.csr -signkey self_signed_key.pem -out self_signed_cert.pem -days 7305
```
