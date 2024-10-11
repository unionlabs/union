**How example certificates and keys generated in this directory:**

To create a self signed cert(and private key), Run the following commands using openssl:
```
openssl req -x509 -sha256 -days 7305 -newkey rsa:2048 -keyout root_key.pem -out
root_cert.pem
```
To create a chain of certs:

```leafCert.pem``` < ```intermediateCert.pem``` < ```rootCert.pem```

Run the following commands using openssl:

Create a self signed root:
```
openssl req -x509 -sha256 -days 7305 -newkey rsa:2048 -keyout root_key.pem -out root_cert.pem
```
Create a configuration file config.cnf:
```
basicConstraints=CA:TRUE
```
Create the intermediate cert private key:
```
openssl genrsa -out intermediate_key.pem 2048
```
Create a certificate signing request:
```
openssl req -key intermediate_key.pem -new -out intermediate.csr
```
Sign the CSR with the root:
```
openssl x509 -req -CA root_cert.pem -CAkey root_key.pem -in intermediate.csr
-out intermediate_cert.pem -days 7305 -CAcreateserial -extfile config.cnf
```
Create the leaf cert private key:
```
openssl genrsa -out leaf_key.pem 2048
```
Create a certificate signing request:
```
openssl req -key leaf_key.pem -new -out leaf.csr
```
Sign the CSR with the intermediate
```
openssl x509 -req -CA intermediate_cert.pem -CAkey intermediate_key.pem -in
leaf.csr -out leaf_cert.pem -days 7305 -CAcreateserial -extfile config
```
TODO(rmehta19): Perhaps put these commands together into a script to make
generation of example certs/keys and cert chains simpler.
