# BeringShare Chat - Debug & Testing Guide

## Overview
A comprehensive debug console has been added to the chat UI to help diagnose message flow issues between federated servers.

## How to Access Debug Console

1. **Open Chat UI**: Navigate to `http://localhost:8081/chat/ui` or `http://localhost:8082/chat/ui`
2. **Login**: Use username `bob` (or any user) with password `pass`
3. **Toggle Debug**: Click the **Debug** button in the bottom-right corner to open the debug console

## What the Debug Console Shows

The debug console displays real-time logs of all network activity and operations:

### Log Entry Format
- **Timestamp**: `[HH:MM:SS]`
- **Request arrows**: 
  - `→` = outgoing request
  - `←` = response received
  - `⚠` = warning/error

### Examples of Log Entries

**Successful Login:**
```
[14:32:15] → POST /api/login
  Body: {"username":"bob","password":"pass"}
[14:32:15] ← Status: 200 from /api/login
  Response: {"user_id":"abc...","username":"bob","token":"def..."}
```

**Loading Messages:**
```
[14:32:20] Loading dm messages for ID: user-123
[14:32:20] → GET /api/messages/dm/user-123
[14:32:20] ← Status: 200 from /api/messages/dm/user-123
  Response: [{"id":"msg-1","body":"hello","author_username"...
[14:32:20] Got 2 messages
```

**Sending DM to Remote User:**
```
[14:32:45] Sending message: "Hi Alice" to target: user-456 (user)
[14:32:45]   Remote user detected: alice (server_id: server-b) → sending as alice@server_b
[14:32:45]   POSTing to /api/messages/dm with recipient: alice@server_b
  Body: {"recipient":"alice@server_b","body":"Hi Alice"}
[14:32:45] ← Status: 200 from /api/messages/dm
  Response: success
[14:32:45] Send result: success
[14:32:45] Loading dm messages for ID: user-456
[14:32:45] → GET /api/messages/dm/user-456
[14:32:45] ← Status: 200 from /api/messages/dm/user-456
[14:32:45] Got 3 messages
```

## Testing Scenarios

### Scenario 1: Local Chat (Same Server)
**Goal**: Verify messages work within the same server

**Steps**:
1. Open two browser windows to `http://localhost:8081/chat/ui`
2. Login as `bob` in window 1
3. Login as `alice` in window 2
4. In window 1, click on "alice" in Direct Messages
5. Send a message: "Hello Alice!"
6. Watch the debug console:
   - Should see `→ POST /api/messages/dm` with recipient: `alice` (no @)
   - Should see `← Status: 200`
   - Then `→ GET /api/messages/dm/...` to load history
   - Message should appear in both windows

**Expected Result**: ✅ Message appears in both windows immediately

**If Messages Don't Appear**:
- Check debug console for error status codes
- If Status: 401, user is not authenticated
- If Status: 400, check if user exists in `/api/users`
- If `Got 0 messages` after sending, messages aren't being saved to DB

### Scenario 2: Cross-Server Chat (Federation)
**Goal**: Verify messages work between two federated servers

**Setup**:
1. Run two servers: `server_a` (8081) and `server_b` (8082)
2. In Admin panel (8081), register server_b at `http://server_b:8080`
3. In Admin panel (8081), click "Fetch Users From Federated Servers"
4. In Admin panel (8082), do the same for server_a

**Steps**:
1. Open `http://localhost:8081/chat/ui` and login as `bob`
2. Open `http://localhost:8082/chat/ui` and login as `alice`
3. In bob's chat (8081), look for "alice" in Direct Messages
   - If `alice` has a grey icon (remote user), she's from server_b ✅
   - Debug console should show: `Remote user detected: alice (server_id: server-b) → sending as alice@server_b`
4. Send message: "Hi from server_a"
5. Check debug console on bob's browser:
   - Should see `→ POST /api/messages/dm` with recipient: `alice@server_b`
   - Should see `← Status: 200`

6. Check alice's chat (8082):
   - Should see message from bob in DM history
   - Is message there? ✅ Federation works!
   - Is message missing? ❌ See troubleshooting below

**Expected Result**: ✅ Message appears in alice's inbox on server_b

**If Messages Don't Appear on Remote Server**:
- **Check bob's debug console**: 
  - If Status 401/500, there's an auth or server error
  - If Status 200 but no message on alice's side, federation endpoint issue
  
- **Check alice's debug console**:
  - Manually send a message to bob@server_a
  - Does it reach bob? (If yes, federation works one-way)
  - Debug output should show: `→ POST /api/messages/dm` with recipient: `bob@server_a`

### Scenario 3: Channel Messaging
**Goal**: Verify channel messages sync between users

**Steps**:
1. In Admin panel (8081), create a channel called `lobby`
2. Add both `bob` and `alice` as channel members
3. In bob's chat (8081), click `# lobby` channel
4. Send a message: "Hello channel!"
5. Debug console should show:
   ```
   Sending message: "Hello channel!" to target: channel-id (channel)
   POSTing to /api/messages/channel with channel: lobby
   → POST /api/messages/channel
   ← Status: 200
   ```
6. In alice's chat (8082), check if she sees the message in the same channel

**Expected Result**: ✅ Both users see channel messages

## Common Issues & Solutions

### Issue 1: "Got 0 messages" after sending
**Cause**: Message wasn't saved to database
**Fix**:
1. Check if `/api/login` returned a valid `user_id`
2. Verify user exists: `GET /api/users` in debug console
3. Check database:
   ```bash
   sqlite3 data.db "SELECT COUNT(*) FROM messages;"
   ```

### Issue 2: Remote user recipient not formatted
**Cause**: `serverMap` wasn't populated during init
**Debug**: Check if server load succeeded:
```
→ GET /admin/servers
← Status: 200
```
If status is 401, user token isn't admin token (should be user token!)

### Issue 3: DM recipient shown as "[object Object]"
**Cause**: allUsers array contains user object, not string
**Fix**: Already fixed in code - rebuild with latest

### Issue 4: Message shows on sender's side but not recipient's
**Cause**: Federation message didn't reach remote server
**Debug**:
- Check sender's debug console for Status 200 on POST /api/messages/dm
- Check recipient's debug console - do they see a GET request for `/api/messages/dm/...`?
- If no GET request, message isn't being synced to their inbox
- **Likely cause**: Recipient username mismatch or federation URL misconfigured

## Testing Messages Directly (Advanced)

If UI testing doesn't work, test the raw API:

### Get User Token
In Admin panel (8081):
1. Go to "Federation Sync Test" section
2. Fill in username `bob` and password `pass` in a login endpoint
3. Use the token in `User token` field

### Send DM via API
1. Open Debug console
2. Go to "Federation Test" in Admin
3. Fill:
   - **User token**: Your bearer token
   - **DM recipient**: `alice` (local) or `alice@server_b` (remote)
   - **Message**: `hello from api`
4. Click "Send DM"
5. Check the response in debug section

### View Inbox
1. Set User token in Admin panel
2. Click "Load inbox" button
3. Raw JSON will show all messages received

## Debug Console Max Capacity

- Debug console stores last **100 log entries**
- Older entries are automatically discarded
- You can scroll up in the console to see older entries
- Copy/paste logs to share with developers

## Performance Notes

- Debug logging has minimal performance impact
- Each log entry is <200 characters
- Network requests are the bottleneck, not logging

## Next Steps

After debugging message flow:
1. Fix any identified issues in backend (messages.rs)
2. Verify message persistence in database
3. Check federation routing in admin.rs
4. Re-run scenarios to confirm fixes

---

**Last Updated**: 2026-02-08
**Version**: Debug Console v1.0
