#!/bin/sh
# Build aa-proxy-rs for AAWireless 1 CS317 (armv7) inside WSL2/Linux with Docker.
# Output: ./output/aa-proxy-rs and ./output/SHA256SUMS
set -eu

cd "$(dirname "$0")/../.."

COMMIT="$(git rev-parse --short=12 HEAD 2>/dev/null || echo unknown)"
BRANCH="$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo unknown)"

DOCKER_BUILDKIT=1 docker build -f Dockerfile.aaw1 --target export \
    --build-arg AA_PROXY_COMMIT="${BRANCH}@${COMMIT}" \
    --output type=local,dest=./output .

cd output
sha256sum aa-proxy-rs > SHA256SUMS
echo "source: ${BRANCH}@${COMMIT}" >> SHA256SUMS
cat SHA256SUMS
