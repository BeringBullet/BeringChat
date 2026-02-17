#!/usr/bin/env bash
set -euo pipefail

root_dir=$(cd "$(dirname "$0")/../.." && pwd)
cd "$root_dir"

docker compose up -d --build
trap 'docker compose down -v' EXIT

wait_for_health() {
  local url=$1
  for _ in {1..40}; do
    if curl -fsS "$url/health" >/dev/null; then
      return 0
    fi
    sleep 0.5
  done
  echo "health check failed for $url" >&2
  return 1
}

wait_for_health "http://localhost:8081"
wait_for_health "http://localhost:8082"

create_user() {
  local base=$1
  local admin=$2
  local username=$3
  curl -sS -X POST "$base/admin/users" \
    -H "x-admin-token: $admin" \
    -H "content-type: application/json" \
    -d "{\"username\":\"$username\"}"
}

register_server() {
  local base=$1
  local admin=$2
  local name=$3
  local url=$4
  local token=$5
  curl -sS -X POST "$base/admin/servers" \
    -H "x-admin-token: $admin" \
    -H "content-type: application/json" \
    -d "{\"name\":\"$name\",\"base_url\":\"$url\",\"token\":\"$token\"}"
}

json_get() {
  local json=$1
  local key=$2
  python3 - "$json" "$key" <<'PY'
import json,sys
payload=json.loads(sys.argv[1])
print(payload[sys.argv[2]])
PY
}

alice_json=$(create_user "http://localhost:8081" "admin-a" "alice")
bob_json=$(create_user "http://localhost:8082" "admin-b" "bob")

alice_token=$(json_get "$alice_json" token)
bob_token=$(json_get "$bob_json" token)

register_server "http://localhost:8081" "admin-a" "b" "http://server_b:8080" "token-b" >/dev/null
register_server "http://localhost:8082" "admin-b" "a" "http://server_a:8080" "token-a" >/dev/null

channel_json=$(curl -sS -X POST "http://localhost:8081/admin/channels" \
  -H "x-admin-token: admin-a" \
  -H "content-type: application/json" \
  -d "{\"name\":\"lobby\"}")
channel_id=$(json_get "$channel_json" id)

curl -sS -X POST "http://localhost:8081/admin/channels/$channel_id/members" \
  -H "x-admin-token: admin-a" \
  -H "content-type: application/json" \
  -d "{\"username\":\"bob\",\"server_name\":\"b\"}" >/dev/null

curl -sS -X POST "http://localhost:8081/api/messages/dm" \
  -H "authorization: Bearer $alice_token" \
  -H "content-type: application/json" \
  -d "{\"recipient\":\"bob@b\",\"body\":\"hello federated\"}" >/dev/null

found=0
for _ in {1..20}; do
  inbox=$(curl -sS "http://localhost:8082/api/messages/inbox" -H "authorization: Bearer $bob_token")
  found=$(python3 - "$inbox" <<'PY'
import json,sys
payload=json.loads(sys.argv[1])
print("true" if any(item.get('body')=='hello federated' for item in payload) else "false")
PY
)
  if [ "$found" = "true" ]; then
    found=1
    break
  fi
  sleep 0.5
done

if [ "$found" -ne 1 ]; then
  echo "message not replicated" >&2
  exit 1
fi

curl -sS -X POST "http://localhost:8081/api/messages/channel" \
  -H "authorization: Bearer $alice_token" \
  -H "content-type: application/json" \
  -d "{\"channel\":\"lobby\",\"body\":\"hello channel\"}" >/dev/null

found=0
for _ in {1..20}; do
  inbox=$(curl -sS "http://localhost:8082/api/messages/inbox" -H "authorization: Bearer $bob_token")
  found=$(python3 - "$inbox" <<'PY'
import json,sys
payload=json.loads(sys.argv[1])
print("true" if any(item.get('body')=='hello channel' for item in payload) else "false")
PY
)
  if [ "$found" = "true" ]; then
    found=1
    break
  fi
  sleep 0.5
done

if [ "$found" -ne 1 ]; then
  echo "channel message not replicated" >&2
  exit 1
fi

echo "federation test passed"
