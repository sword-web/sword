#!/usr/bin/env bash
set -euo pipefail

DIR="$(cd "$(dirname "$0")" && pwd)"
CERTS="$DIR/certs"

mkdir -p "$CERTS"
cd "$CERTS"

if [ ! -f ca.pem ]; then
    openssl req -x509 -newkey rsa:2048 -days 3650 -nodes \
        -keyout ca-key.pem -out ca.pem \
        -subj "/CN=Sword Test CA"
fi

if [ ! -f server.pem ]; then
    openssl req -newkey rsa:2048 -nodes \
        -keyout server-key.pem -out server.csr \
        -subj "/CN=localhost"

    openssl x509 -req -in server.csr \
        -CA ca.pem -CAkey ca-key.pem -CAcreateserial \
        -out server.pem -days 3650 \
        -extfile <(printf "subjectAltName=DNS:localhost,IP:127.0.0.1")
fi

if [ ! -f client.pem ]; then
    openssl req -newkey rsa:2048 -nodes \
        -keyout client-key.pem -out client.csr \
        -subj "/CN=sword-client"

    openssl x509 -req -in client.csr \
        -CA ca.pem -CAkey ca-key.pem -CAcreateserial \
        -out client.pem -days 3650
fi

chmod 644 ./*.pem
