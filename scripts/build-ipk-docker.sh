#!/bin/sh
# Build the ARM package (dist/cc.ncls.luma_<version>_arm.ipk, sha256sums, repo.json) inside a Nix
# container, so the pinned toolchain (Rust 1.93, Node 24, pnpm 10) is used whatever the host has.
# This is the procedure used for the releases; it needs Docker and network access, and takes a while
# the first time (the Nix store, webOS SDK and SDL-webOS are downloaded into volumes and .build/).
#
#   scripts/build-ipk-docker.sh version   # consume the changesets and bump every version file
#   scripts/build-ipk-docker.sh           # build the package from the committed tree
#
# The host's /lib64 and /lib/x86_64-linux-gnu are mounted read-only so the webOS SDK compiler (a
# prebuilt x86-64 binary) can run; this assumes an x86-64 Linux host.
set -eu
cd "$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
command -v docker >/dev/null 2>&1 || { echo "error: docker is required" >&2; exit 1; }
NIX_VOLUME=${NIX_VOLUME:-lg-hush-nix-store}
uid=$(id -u); gid=$(id -g)
case "${1:-build}" in
  version) STEPS='pnpm version:release && pnpm version:check' ;;
  build) STEPS='pnpm assets
      pnpm sdk
      pnpm runtime
      export WEBOS_NDK=/work/.build/toolchain/arm-webos-linux-gnueabi_sdk-buildroot
      pnpm run package' ;;
  *) echo "usage: $0 [version|build]" >&2; exit 1 ;;
esac

docker run --rm \
  -v "$NIX_VOLUME":/nix -v "$PWD":/work \
  -v /lib64:/lib64:ro -v /lib/x86_64-linux-gnu:/lib/x86_64-linux-gnu:ro \
  -w /work -e HOST_UID="$uid" -e HOST_GID="$gid" -e STEPS="$STEPS" \
  -e 'NIX_CONFIG=experimental-features = nix-command flakes' \
  nixos/nix:2.35.4 sh -ec '
    # Git refuses to read a checkout owned by another user, so the flake is copied out.
    git config --global --add safe.directory /work
    mkdir -p /tmp/luma-flake
    cp /work/flake.nix /work/flake.lock /work/package.json /tmp/luma-flake/
    nix develop path:/tmp/luma-flake --command sh -ec "
      pnpm install --frozen-lockfile --store-dir .build/pnpm-store
      $STEPS
    "
    chown -R "$HOST_UID:$HOST_GID" /work 2>/dev/null || true
  '
[ -d dist ] && ls -l dist || true
