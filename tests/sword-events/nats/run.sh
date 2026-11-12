#!/usr/bin/env bash
set -euo pipefail

DIR="$(cd "$(dirname "$0")" && pwd)"
NAMES=(sword-nats-userpass sword-nats-token sword-nats-mtls)

cleanup() {
    docker rm -f "${NAMES[@]}" >/dev/null 2>&1 || true
}
trap cleanup EXIT

"$DIR/generate-certs.sh"
cleanup

docker run -d --name "${NAMES[0]}" -p 4222:4222 \
    -v "$DIR:/etc/nats:ro" -v "$DIR/certs:/certs:ro" \
    nats:2-alpine -c /etc/nats/nats-userpass.conf

docker run -d --name "${NAMES[1]}" -p 4223:4223 \
    -v "$DIR:/etc/nats:ro" \
    nats:2-alpine -c /etc/nats/nats-token.conf

docker run -d --name "${NAMES[2]}" -p 4224:4224 \
    -v "$DIR:/etc/nats:ro" -v "$DIR/certs:/certs:ro" \
    nats:2-alpine -c /etc/nats/nats-mtls.conf

for name in "${NAMES[@]}"; do
    ready=0

    for _ in $(seq 1 50); do
        if docker logs "$name" 2>&1 | grep -q "Server is ready"; then
            ready=1
            break
        fi

        if [ "$(docker inspect -f '{{.State.Running}}' "$name" 2>/dev/null)" != "true" ]; then
            echo "container $name stopped before becoming ready" >&2
            docker logs "$name" >&2 || true
            exit 1
        fi

        sleep 0.2
    done

    if [ "$ready" -ne 1 ]; then
        echo "container $name did not become ready" >&2
        docker logs "$name" >&2 || true
        exit 1
    fi
done

cd "$DIR/.."
cargo test -p sword-events-tests -- --ignored
