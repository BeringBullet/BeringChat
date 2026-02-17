# BeringShare Debug System - Getting Started Checklist

## ✅ What Was Done

### Code Changes
- [x] Added debug console HTML panel to chat UI
- [x] Added debug toggle button (bottom-right corner)
- [x] Enhanced `requestJson()` to log all network requests
- [x] Enhanced `loadMessages()` to show message counts
- [x] Enhanced `sendMessage()` to show operation details
- [x] Added remote user detection logging
- [x] Added timestamp tracking for all logs
- [x] Added circular buffer (100 logs max)
- [x] Code compiles without errors ✅

### Documentation Created
- [x] `DEBUG_GUIDE.md` - Comprehensive debug guide with scenarios
- [x] `QUICK_REFERENCE.md` - Quick lookup for common patterns
- [x] `BUILD_GUIDE.md` - Build and deployment instructions
- [x] `DEBUG_IMPLEMENTATION.md` - Technical details of what was added
- [x] `GETTING_STARTED_CHECKLIST.md` - This file!

## 📋 Next Steps - In Order

### Step 1: Rebuild Code
```bash
cd d:\code\beringshare
cargo build --release
```
**Expected**: No errors (takes ~30-60 seconds)

### Step 2: Deploy to Docker
```bash
docker-compose down
docker-compose up --build -d
```
**Expected**: Two containers running on ports 8081 and 8082

### Step 3: Test Login
1. Open http://localhost:8081/chat/ui
2. Login with:
   - Username: `alice`
   - Password: `pass`
3. Should see chat UI with sidebar
4. **Check debug console** (click "Debug" button, bottom-right)

### Step 4: Test Local Chat (Same Server)
1. Open http://localhost:8081/chat/ui in Window 1
2. Login as `bob`
3. Open http://localhost:8081/chat/ui in Window 2
4. Login as `alice`
5. In Window 1, click `alice` in Direct Messages
6. Send message: "Hello!"
7. **Monitor debug console**:
   - Look for `→ POST /api/messages/dm`
   - Look for `← Status: 200`
   - Look for `Got N messages` after
8. Does message appear in Window 2 immediately? → ✅ **Local chat works!**

### Step 5: Test Cross-Server Chat (Federation)
1. Go to Admin Panel: http://localhost:8081/admin
2. Login: `admin` / `password`
3. Create users if needed:
   - In "Create user" section, add `bob` and `alice`
4. Register Server B:
   - Name: `server_b`
   - URL: `http://server_b:8080`
   - Token: (leave blank for auto-generate)
5. Click "Fetch Users From Federated Servers"
   - Should see success message
6. Click "View All Users"
   - Should see both local and remote users listed

### Step 6: Test Remote Message Send
1. Open http://localhost:8081/chat/ui → Login as `bob`
2. Open http://localhost:8082/chat/ui → Login as `alice`
3. In Bob's chat, look in Direct Messages:
   - If `alice` appears with gray text → She's marked as remote ✅
4. Click on `alice`
5. Send message: "Hi Alice from server_a!"
6. **Monitor Bob's debug console**:
   ```
   Sending message: "Hi Alice from server_a!" to target: ... (user)
   Remote user detected: alice (server_id: ...) → sending as alice@server_b
   POSTing to /api/messages/dm with recipient: alice@server_b
   → POST /api/messages/dm
   ← Status: 200
   ```
7. **Check Alice's inbox**:
   - Does message appear from bob?
   - If yes → ✅ **Federation works!**
   - If no → ⚠️ **See troubleshooting below**

## 🔍 If Messages Aren't Appearing

### For Local Chat:
**Symptom**: Bob sends to Alice (same server), Alice doesn't see it

**Debug Steps**:
1. Open debug console on sender's browser
2. Look for error indicators:
   - Any `← Status: 401` (re-login needed)
   - Any `← Status: 500` (server error)
   - `Got 0 messages` (not saved to DB)
3. Check database:
   ```bash
   sqlite3 data.db "SELECT COUNT(*) FROM messages;"
   ```
   - If 0, messages not being saved
   - Check backend error logs

### For Remote Chat:
**Symptom**: Bob sends to Alice@server_b, Alice doesn't see it

**Debug Steps**:
1. **Sender side** - Check debug console:
   - Does log show `recipient: alice@server_b`? (not just `alice`)
   - Does POST return Status 200?
   - If 401/500, that's the problem
   
2. **Receiver side** - Check debug console:
   - Does it show `→ GET /api/messages/dm/...` shortly after send?
   - If no GET request, message never synchronized to remote server
   - Federation URL might be wrong

3. **Database check** on server B:
   ```bash
   sqlite3 data_b.db "SELECT COUNT(*) FROM messages;"
   sqlite3 data_b.db "SELECT author_username, body FROM messages LIMIT 5;"
   ```
   - If 0 messages: federation didn't deliver
   - If messages exist but don't appear in UI: frontend bug

4. **Server registration check**:
   - Admin panel (8081) → "Federated Servers"
   - Is server_b registered?
   - Is URL `http://server_b:8080` (not localhost:8082)?

## 🚨 Common Issues & Quick Fixes

### Issue: "Got 0 messages" always shown
- **Cause**: Messages not saving to database
- **Fix**: Check if POST /api/messages/dm returns 200
- **If yes**: Database issue, check backend logs
- **Commands**:
  ```bash
  sqlite3 data.db "SELECT * FROM messages;" # Should show message
  ```

### Issue: Status 401 on login
- **Cause**: Invalid credentials or token expired
- **Fix**: Clear browser storage and re-login
- **Steps**:
  1. Open DevTools (F12)
  2. Application → localStorage
  3. Delete `user_token`
  4. Refresh chat page
  5. Re-login

### Issue: Remote users not appearing
- **Cause**: Federation sync didn't work
- **Fix**: Admin panel → "Fetch Users From Federated Servers"
- **Check**:
  - Is server_b registered correctly?
  - Is URL reachable? Try visiting `http://server_b:8080` in browser
  - After sync, refresh chat page

### Issue: "alice@server_b" not showing in debug
- **Cause**: `serverMap` not populated
- **Fix**: Check if `/admin/servers` loaded successfully
- **Debug**:
  - Should see `→ GET /admin/servers` in debug on init
  - Should see `← Status: 200` with server list

### Issue: Message appears for sender but not receiver
- **Cause**: Works locally BUT federation didn't deliver
- **Fix**: Check federation URL configuration
- **Steps**:
  1. Verify Server B URL in Admin → "Federated Servers"
  2. Test connectivity: `ping server_b` (from container) or `curl http://server_b:8080`
  3. Check if `/federation/messages` endpoint exists on remote

## 📊 Debug Console Interpretation

### Green light (✅ working):
```
→ POST /api/messages/dm
← Status: 200
Got N messages
```

### Yellow light (⚠️ possible issue):
```
Got 0 messages  # Check if message was sent
```

### Red light (❌ definitely broken):
```
← Status: 401   # Auth failed, re-login
← Status: 500   # Server error, check logs
⚠ User not found # User doesn't exist
```

## 📚 Documentation Files

- **`DEBUG_GUIDE.md`** - Start here for comprehensive guide
  - Scenario-based testing
  - Issue troubleshooting
  - Advanced API testing

- **`QUICK_REFERENCE.md`** - Quick lookup card
  - Log patterns
  - Status codes
  - Testing checklist

- **`BUILD_GUIDE.md`** - Setup and deployment
  - Build steps
  - Docker configuration
  - Environment variables

- **`DEBUG_IMPLEMENTATION.md`** - Technical details
  - Code changes made
  - Feature list
  - Integration points

## 🎯 Success Criteria

Your chat system is working when:

- [ ] Local users can send DMs to each other (same server)
- [ ] Local users see message history
- [ ] Remote users appear in Direct Messages after sync
- [ ] Remote DMs show recipient as "user@server_name" in debug
- [ ] Remote DMs deliver to other server (appear in inbox)
- [ ] Channel messages work locally
- [ ] New channels can be created
- [ ] Debug console shows Status 200 on successful operations

## ⏱️ Estimated Testing Time

- **Setup**: 5-10 minutes (build + deploy)
- **Local chat test**: 5 minutes
- **Federation setup**: 5-10 minutes
- **Remote chat test**: 5 minutes
- **Debugging (if needed)**: 15-30 minutes

**Total**: 30-60 minutes for full validation

## 🆘 If You Get Stuck

1. **Check debug console first** - it's your main diagnostic tool
2. **Copy the error** in debug console (with timestamp)
3. **Check the relevant doc**:
   - Network issues → `BUILD_GUIDE.md`
   - Message flow → `DEBUG_GUIDE.md`
   - Log interpretation → `QUICK_REFERENCE.md`
4. **Database query** to verify state:
   ```bash
   sqlite3 data.db "SELECT username, COUNT(*) as msg_count FROM messages GROUP BY username;"
   ```

## 📝 Notes

- **Debug console is safe to leave open** - zero performance impact
- **Logs clear on page reload** - screenshot important ones
- **Status 200 doesn't guarantee delivery** - check remote inbox too
- **Use two browsers** for best local testing experience

---

## Ready to Begin? 

**Start with Step 1**: Rebuild code and deploy!

```bash
cd d:\code\beringshare
cargo build --release
# Then deploy with docker-compose
```

Once deployed, the debug console will be your best friend. Happy debugging! 🐛→✅

---

**Version**: 1.0
**Last Updated**: 2026-02-08
**Status**: Ready to test!
