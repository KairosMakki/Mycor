#!/usr/bin/env bash
set -e
PREFIX="${PREFIX:-/usr/local}"
mkdir -p "$PREFIX/bin"
cp myc "$PREFIX/bin/"
echo "installed myc to $PREFIX/bin"
