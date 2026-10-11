#!/usr/bin/env bash
set -euo pipefail
cd /home/kitti/Projects/line-linux/linejs
exec deno run -A --config deno.json ../engine/daemon.ts
