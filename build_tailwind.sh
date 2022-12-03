#!/usr/bin/env bash
set -e
NODE_ENV=production tailwindcss -c ./tailwind.config.js -i src/tailwind.css -o tailwind.css --minify