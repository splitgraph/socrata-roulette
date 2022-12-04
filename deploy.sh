#!/usr/bin/env bash
set -ex

TARGET="$HOME/splitgraph.github.io/"

echo "Creating a production build"
trunk build --release --public-url socrata-roulette/
DIST="$PWD/dist"

echo "Copying"
pushd "$TARGET"

git rm "socrata-roulette/" -r || echo "Directory doesn't exist, creating"
mkdir -p "socrata-roulette/"

echo "Copying the files"
cp -r "$DIST"/* "socrata-roulette/"

echo "Creating the Git commit"
git add "socrata-roulette/"

rel_msg="Release $(git rev-parse HEAD | cut -c-10) at $(date -u +"%Y-%m-%dT%H:%M:%SZ")"
git commit -m "$rel_msg"
git push
