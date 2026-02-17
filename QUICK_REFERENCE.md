# BeringShare Chat - Quick Debug Reference

## Debug Console Features

| Feature | Purpose |
|---------|---------|
| **Network Logs** | Shows every HTTP request (→) and response (←) |
| **Status Codes** | 200 = success, 401 = auth error, 500 = server error |
| **Request Body** | Shows what data was sent in POST requests |
| **Response Preview** | First 100 chars of API response |
| **Real-time Timestamps** | Every log entry stamped HH:MM:SS |
| **Message Operations** | Tracks send/receive/load operations |
| **User Detection** | Shows if recipient is local or remote |

## Key Log Patterns

### ✅ Successful DM Send Flow
```
Sending message: "text" to target: id (user)
Remote user detected: name (server_id: ...) → sending as name@server
POSTing to /api/messages/dm with recipient: name@server
→ POST /api/messages/dm
  Body: {"recipient":"name@server","body":"text"}
← Status: 200 from /api/messages/dm
  Response: success
Loading dm messages for ID: id
→ GET /api/messages/dm/id
← Status: 200
Got N messages
```

### ❌ Common Failure Patterns

**Pattern: 401 Unauthorized**
```
← Status: 401 from /api/messages/dm
⚠ 401 Unauthorized - logging out
```
→ **Fix**: Re-login, token may have expired

**Pattern: User Not Found**
```
⚠ User not found in allUsers for ID: ...
```
→ **Fix**: User doesn't exist, create in Admin panel

**Pattern: Got 0 Messages**
```
Loading dm messages for ID: user-456
← Status: 200
Got 0 messages
```
→ **Fix**: Messages not saved, check database persistence

**Pattern: No Remote User**
```
Local user: alice
POSTing to /api/messages/dm with recipient: alice
```
→ **Fix**: Alice is local, not remote. Send to alice@server_b if she's on server_b

## Testing Checklist

- [ ] Can login with credentials (username + password)?
- [ ] Do local users appear in "Direct Messages"?
- [ ] Do remote users appear in "Direct Messages" (after sync)?
- [ ] Can send message to local user?
- [ ] Does message appear in recipient's inbox (same server)?
- [ ] Does message appear in sender's sent view?
- [ ] Can send message to remote user?
- [ ] Does remote message reach other server?
- [ ] Can create channels?
- [ ] Can post in channels?
- [ ] Do channel messages appear for all members?

## One-Minute Diagnosis

1. **Open Debug console** (bottom-right "Debug" button)
2. **Send a test message**
3. **Look for red/warning lines** (⚠ or "error")
4. **Check HTTP status codes** ( ← Status: )
5. **If Status 200**: Backend received it, check recipient's inbox
6. **If Status 4xx/5xx**: Request failed, check error message

## Database Quick Checks

```bash
# Check if messages were saved
sqlite3 data.db "SELECT COUNT(*) FROM messages;"

# List recent messages
sqlite3 data.db "SELECT author_username, body, sent_at FROM messages ORDER BY sent_at DESC LIMIT 10;"

# Check users exist
sqlite3 data.db "SELECT username, is_local, server_id FROM users;"

# Check if message is for correct receiver
sqlite3 data.db "SELECT recipient_user_id, body FROM messages LIMIT 5;"
```

## Common Terminal Commands

```bash
# Build code
cargo build

# Check for errors only
cargo build 2>&1 | Select-String -Pattern "error"

# Rebuild from scratch
cargo clean && cargo build

# View server logs (if running locally)
# Look for "POST /api/messages/dm" entries
```

## Config Verification

**Server A (8081):**
- Admin: http://localhost:8081/admin
- Chat: http://localhost:8081/chat/ui
- API Base: http://localhost:8081

**Server B (8082):**
- Admin: http://localhost:8082/admin
- Chat: http://localhost:8082/chat/ui
- API Base: http://localhost:8082

**Docker Addresses:**
- Server A internal: http://server_a:8080
- Server B internal: http://server_b:8080

## Token Types

| Token | Where Used | Lifetime |
|-------|-----------|----------|
| **session token** | Admin panel (x-admin-token header) | 3600s |
| **user token** | Chat UI, stored in localStorage | Session |

---

**Pro Tip**: Keep debug console open while testing. Watch for "⚠" warnings - they're usually the first sign of a problem!
