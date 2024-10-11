.PHONY: all root-ssl server-ssl client-ssl

# Rebuilds self-signed root/server/client certs/keys in a consistent way
all: root-ssl server-ssl client-ssl
	rm -f .srl

root-ssl:
	openssl req -new -sha256 -nodes -newkey rsa:2048 \
		-config ./certs/root.cnf \
		-keyout /tmp/root.key \
		-out /tmp/root.csr
	openssl x509 -req -days 3653 -sha256 \
		-in /tmp/root.csr  \
		-extfile /etc/ssl/openssl.cnf -extensions v3_ca \
		-signkey /tmp/root.key \
		-out ./certs/root.crt

server-ssl:
	openssl req -new -sha256 -nodes -newkey rsa:2048 \
		-config ./certs/server.cnf \
		-keyout ./certs/server.key \
		-out /tmp/server.csr
	openssl x509 -req -days 3653 -sha256 \
		-extfile ./certs/server.cnf -extensions req_ext \
		-CA ./certs/root.crt -CAkey /tmp/root.key -CAcreateserial \
		-in /tmp/server.csr \
		-out ./certs/server.crt

client-ssl:
	openssl req -new -sha256 -nodes -newkey rsa:2048 \
		-config ./certs/postgresql.cnf \
		-keyout ./certs/postgresql.key \
		-out /tmp/postgresql.csr
	openssl x509 -req -days 3653 -sha256 \
		-CA ./certs/root.crt -CAkey /tmp/root.key -CAcreateserial \
		-in /tmp/postgresql.csr \
		-out ./certs/postgresql.crt
