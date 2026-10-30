#!/bin/sh
# Build aa-proxy-rs for AAWireless 1 CS317 (armv7) inside WSL2/Linux with Docker.
# Output: ./output/aa-proxy-rs and ./output/SHA256SUMS
set -eu

cd "$(dirname "$0")/../.."

COMMIT="$(git rev-parse --short=12 HEAD 2>/dev/null || echo unknown)"
BRANCH="$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo unknown)"

# build.rs shows "git: <AA_PROXY_COMMIT[..7]>-br#<BUILDROOT_COMMIT[..7]>" when .git
# is not in the Docker context, so pass the commit date and short hash.
DATE="$(git log -1 --format=%cd --date=format:%Y%m%d 2>/dev/null || echo 0)"
SHORT="$(git rev-parse --short=7 HEAD 2>/dev/null || echo unknown)"

DOCKER_BUILDKIT=1 docker build -f Dockerfile.aaw1 --target export \
    --build-arg AA_PROXY_COMMIT="${DATE}" \
    --build-arg BUILDROOT_COMMIT="${SHORT}" \
    --output type=local,dest=./output .

cd output
sha256sum aa-proxy-rs > SHA256SUMS
echo "source: ${BRANCH}@${COMMIT}" >> SHA256SUMS
cat SHA256SUMS
