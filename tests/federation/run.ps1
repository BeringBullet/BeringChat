$ErrorActionPreference = "Stop"

$root = Resolve-Path (Join-Path $PSScriptRoot "..\..")
Set-Location $root

docker compose up -d --build
try {
    function Wait-ForHealth($url) {
        for ($i = 0; $i -lt 40; $i++) {
            try {
                Invoke-RestMethod "$url/health" | Out-Null
                return
            } catch {
                Start-Sleep -Milliseconds 500
            }
        }
        throw "health check failed for $url"
    }

    Wait-ForHealth "http://localhost:8081"
    Wait-ForHealth "http://localhost:8082"

    function Create-User($base, $admin, $username) {
        return Invoke-RestMethod "$base/admin/users" -Method Post -Headers @{"x-admin-token"=$admin} -ContentType "application/json" -Body ("{""username"":""$username""}")
    }

    function Register-Server($base, $admin, $name, $url, $token) {
        Invoke-RestMethod "$base/admin/servers" -Method Post -Headers @{"x-admin-token"=$admin} -ContentType "application/json" -Body ("{""name"":""$name"",""base_url"":""$url"",""token"":""$token""}") | Out-Null
    }

    $alice = Create-User "http://localhost:8081" "admin-a" "alice"
    $bob = Create-User "http://localhost:8082" "admin-b" "bob"

    Register-Server "http://localhost:8081" "admin-a" "b" "http://server_b:8080" "token-b"
    Register-Server "http://localhost:8082" "admin-b" "a" "http://server_a:8080" "token-a"

    $channel = Invoke-RestMethod "http://localhost:8081/admin/channels" -Method Post -Headers @{"x-admin-token"="admin-a"} -ContentType "application/json" -Body "{""name"":""lobby""}"
    Invoke-RestMethod "http://localhost:8081/admin/channels/$($channel.id)/members" -Method Post -Headers @{"x-admin-token"="admin-a"} -ContentType "application/json" -Body "{""username"":""bob"",""server_name"":""b""}" | Out-Null

    Invoke-RestMethod "http://localhost:8081/api/messages/dm" -Method Post -Headers @{"authorization"="Bearer $($alice.token)"} -ContentType "application/json" -Body "{""recipient"":""bob@b"",""body"":""hello federated""}" | Out-Null

    $found = $false
    for ($i = 0; $i -lt 20; $i++) {
        $inbox = Invoke-RestMethod "http://localhost:8082/api/messages/inbox" -Headers @{"authorization"="Bearer $($bob.token)"}
        if ($inbox | Where-Object { $_.body -eq "hello federated" }) {
            $found = $true
            break
        }
        Start-Sleep -Milliseconds 500
    }

    if (-not $found) {
        throw "message not replicated"
    }

    Invoke-RestMethod "http://localhost:8081/api/messages/channel" -Method Post -Headers @{"authorization"="Bearer $($alice.token)"} -ContentType "application/json" -Body "{""channel"":""lobby"",""body"":""hello channel""}" | Out-Null

    $found = $false
    for ($i = 0; $i -lt 20; $i++) {
        $inbox = Invoke-RestMethod "http://localhost:8082/api/messages/inbox" -Headers @{"authorization"="Bearer $($bob.token)"}
        if ($inbox | Where-Object { $_.body -eq "hello channel" }) {
            $found = $true
            break
        }
        Start-Sleep -Milliseconds 500
    }

    if (-not $found) {
        throw "channel message not replicated"
    }

    Write-Host "federation test passed"
}
finally {
    docker compose down -v
}
