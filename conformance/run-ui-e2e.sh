#!/usr/bin/env bash
# Runs the real PlanDev UI's Playwright e2e suite against this repo's server.
# Usage: conformance/run-ui-e2e.sh [playwright args...]   e.g. -g "Plans"
set -euo pipefail

UI_REPO=${UI_REPO:-https://github.com/NASA-AMMOS/aerie-ui}
UI_REF=${UI_REF:-bd8312fde8ea722a56870e592c8420e013abae02}   # pinned; matches ui-operations.json
UI_DIR=${UI_DIR:-$(pwd)/target/aerie-ui}
SERVER_PORT=${SERVER_PORT:-8080}
GATEWAY_PORT=${GATEWAY_PORT:-9000}

if [ ! -d "$UI_DIR/.git" ]; then
  mkdir -p "$UI_DIR"
  git -C "$UI_DIR" init -q
  git -C "$UI_DIR" remote add origin "$UI_REPO"
fi
GIT_LFS_SKIP_SMUDGE=1 git -C "$UI_DIR" fetch -q --depth 1 origin "$UI_REF"
git -C "$UI_DIR" checkout -q FETCH_HEAD

cargo build -q -p plandev-server
cargo run -q -p plandev-server -- --port "$SERVER_PORT" --gateway-port "$GATEWAY_PORT" &
SERVER_PID=$!
trap 'kill $SERVER_PID 2>/dev/null || true' EXIT

for _ in $(seq 60); do
  curl -sf "http://localhost:$GATEWAY_PORT/health" >/dev/null && break
  sleep 1
done

cd "$UI_DIR"
npm ci --silent
cat > .env <<EOF
ORIGIN=http://localhost:3000
PUBLIC_GATEWAY_CLIENT_URL=http://localhost:$GATEWAY_PORT
PUBLIC_GATEWAY_SERVER_URL=http://localhost:$GATEWAY_PORT
PUBLIC_HASURA_CLIENT_URL=http://localhost:$SERVER_PORT/v1/graphql
PUBLIC_HASURA_SERVER_URL=http://localhost:$SERVER_PORT/v1/graphql
PUBLIC_HASURA_WEB_SOCKET_URL=ws://localhost:$SERVER_PORT/v1/graphql
PUBLIC_AUTH_SSO_ENABLED=false
EOF
npx playwright test --trace off "$@"
