use axum::response::Html;

pub async fn admin_ui() -> Html<String> {
  // Inject the current server name from environment/config into the admin UI
  let cfg = crate::config::Config::from_env();
  let html = UI_HTML.replace("{{SERVER_NAME}}", &cfg.server_name);
  Html(html)
}

pub async fn chat_ui() -> Html<&'static str> {
    Html(CHAT_UI_HTML)
}

pub async fn settings_ui() -> Html<&'static str> {
    Html(SETTINGS_UI_HTML)
}


const UI_HTML: &str = r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <title>Federated Server Admin</title>
  <style>
    @import url('https://fonts.googleapis.com/css2?family=Space+Grotesk:wght@400;500;600;700&display=swap');
    :root {
      --bg: #0b1112;
      --text: #e7f2f4;
      --muted: #9ab2b7;
      --accent: #49d3b0;
      --danger: #ff6b6b;
      --border: #20373c;
      --shadow: 0 20px 60px rgba(0, 0, 0, 0.45);
    }
    * { box-sizing: border-box; }
    body {
      margin: 0;
      font-family: 'Space Grotesk', system-ui, sans-serif;
      color: var(--text);
      background: linear-gradient(180deg, #060909 0%, var(--bg) 100%);
      min-height: 100vh;
    }
    .login-screen {
      display: flex;
      align-items: center;
      justify-content: center;
      min-height: 100vh;
      padding: 20px;
    }
    .login-card {
      background: rgba(19, 36, 41, 0.95);
      border: 1px solid var(--border);
      border-radius: 18px;
      padding: 40px;
      max-width: 380px;
      width: 100%;
      box-shadow: var(--shadow);
    }
    .login-card h1 {
      margin: 0 0 8px;
      font-size: 28px;
    }
    .login-card p {
      color: var(--muted);
      margin: 0 0 30px;
      font-size: 14px;
    }
    header {
      padding: 32px 6vw 12px;
      display: flex;
      justify-content: space-between;
      align-items: center;
    }
    header h1 {
      margin: 0;
      font-size: 32px;
    }
    .main {
      display: grid;
      gap: 20px;
      padding: 20px 6vw 60px;
    }
    .section {
      border: 1px solid var(--border);
      border-radius: 18px;
      padding: 24px;
      background: rgba(19, 36, 41, 0.95);
      box-shadow: var(--shadow);
    }
    .section h2 {
      margin: 0 0 20px;
      font-size: 22px;
    }
    .form-row {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
      gap: 12px;
      margin-bottom: 16px;
    }
    .form-group {
      display: flex;
      flex-direction: column;
    }
    label {
      font-size: 12px;
      color: var(--muted);
      margin-bottom: 6px;
      text-transform: uppercase;
      letter-spacing: 0.08em;
    }
    input, textarea, select {
      padding: 10px 12px;
      border-radius: 10px;
      border: 1px solid var(--border);
      background: #0b1417;
      color: var(--text);
      font-family: inherit;
      font-size: 14px;
    }
    textarea { min-height: 90px; }
    button {
      padding: 8px 16px;
      border-radius: 8px;
      border: none;
      background: var(--accent);
      color: #08211c;
      font-weight: 600;
      cursor: pointer;
      transition: transform 0.2s, box-shadow 0.2s;
      font-size: 14px;
    }
    button:hover { transform: translateY(-1px); box-shadow: 0 8px 18px rgba(73, 211, 176, 0.2); }
    button.secondary {
      background: transparent;
      border: 1px solid var(--border);
      color: var(--text);
    }
    button.danger {
      background: var(--danger);
      color: #fff;
    }
    button.small {
      padding: 4px 8px;
      font-size: 12px;
    }
    .items-list {
      display: flex;
      flex-direction: column;
      gap: 8px;
    }
    .item {
      display: flex;
      justify-content: space-between;
      align-items: center;
      padding: 12px;
      background: #0f1c1f;
      border-radius: 8px;
      border: 1px solid var(--border);
    }
    .item-info {
      flex: 1;
      font-size: 13px;
    }
    .item-info .name {
      font-weight: 600;
      margin-bottom: 4px;
    }
    .item-info .detail {
      color: var(--muted);
      font-size: 12px;
    }
    .item-actions {
      display: flex;
      gap: 8px;
    }
    .modal {
      display: none;
      position: fixed;
      inset: 0;
      background: rgba(0, 0, 0, 0.5);
      z-index: 1000;
      align-items: center;
      justify-content: center;
      padding: 20px;
    }
    .modal.open {
      display: flex;
    }
    .modal-content {
      background: rgba(19, 36, 41, 0.95);
      border: 1px solid var(--border);
      border-radius: 18px;
      padding: 30px;
      max-width: 500px;
      width: 100%;
      box-shadow: var(--shadow);
    }
    .modal-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      margin-bottom: 24px;
    }
    .modal-header h3 {
      margin: 0;
      font-size: 20px;
    }
    .modal-header button {
      padding: 0;
      width: 32px;
      height: 32px;
      background: transparent;
      border: 1px solid var(--border);
      color: var(--text);
      font-size: 18px;
    }
    .app.logged-in .login-screen {
      display: none;
    }
    .app.logged-in header {
      display: flex;
    }
    .app.logged-in .main {
      display: grid;
    }
    .app:not(.logged-in) header {
      display: none;
    }
    .app:not(.logged-in) .main {
      display: none;
    }
    .status {
      font-size: 12px;
      color: var(--muted);
      margin-top: 8px;
    }
    .pill {
      display: inline-flex;
      align-items: center;
      gap: 6px;
      padding: 4px 12px;
      border-radius: 999px;
      background: rgba(73, 211, 176, 0.12);
      color: var(--accent);
      font-size: 12px;
    }
    .visibility-section { margin-top: 16px; }
    .visibility-section h4 { margin: 0 0 8px; font-size: 13px; color: var(--muted); text-transform: uppercase; }
    .visibility-list { max-height: 200px; overflow-y: auto; border: 1px solid var(--border); border-radius: 8px; padding: 8px; }
    .visibility-item { display: flex; align-items: center; gap: 8px; padding: 6px 4px; }
    .visibility-item input[type="checkbox"] { accent-color: var(--accent); width: 16px; height: 16px; }
    .visibility-item label { font-size: 13px; cursor: pointer; }
  </style>
</head>
<body>
  <div id="app" class="app">
    <!-- Login Screen -->
    <div class="login-screen">
      <div class="login-card">
        <h1>Federated Server Admin</h1>
        <p>Sign in to manage users, servers, and test federation.</p>
        <div style="display: grid; gap: 12px;">
          <div class="form-group">
            <label>Username</label>
            <input id="loginUsername" placeholder="admin" />
          </div>
          <div class="form-group">
            <label>Password</label>
            <input id="loginPassword" type="password" placeholder="password" />
          </div>
          <button onclick="performLogin()">Sign in</button>
          <div id="loginError" style="color: var(--danger); font-size: 12px;"></div>
        </div>
      </div>
    </div>

    <!-- Admin UI -->
    <header>
      <div style="display:flex; align-items:center; gap:12px;">
        <h1 style="margin:0;">Federated Server Admin</h1>
        <div id="serverBadge" class="pill">{{SERVER_NAME}}</div>
      </div>
      <div style="display: flex; gap: 12px; align-items: center;">
        <a href="/chat/ui" style="color: var(--accent); text-decoration: none; font-size: 14px; font-weight: 500; padding: 6px 12px; border: 1px solid var(--border); border-radius: 8px; transition: all 0.2s;" onmouseover="this.style.borderColor='var(--accent)'" onmouseout="this.style.borderColor='var(--border)'">&larr; Back to Chat</a>
        <button class="secondary" onclick="logout()">Logout</button>
      </div>
    </header>

    <div class="main">
      <!-- Server Identity & Federation Tokens -->
      <div class="section">
        <h2>Federation Tokens</h2>
        <p style="color: var(--muted); font-size: 13px; margin: 0 0 16px;">
            Your server token is used by other servers to authenticate with you.
            Share a token with a remote server admin so they can register your server.
        </p>
        <div id="serverIdentity" style="margin-bottom: 16px;"></div>
        <div class="form-row">
            <div class="form-group" style="grid-column: 1 / -1;">
                <label>Create additional token</label>
                <div style="display: flex; gap: 8px;">
                    <input id="fedTokenLabel" placeholder="e.g. For Server B" style="flex: 1;" />
                    <button onclick="createFedToken()">Create</button>
                </div>
            </div>
        </div>
        <div class="items-list" id="fedTokensList"></div>
      </div>

      <!-- Users Section -->
      <div class="section">
        <h2>Users</h2>
        <div class="form-row">
          <div class="form-group" style="grid-column: 1 / -1;">
            <label>Create user</label>
            <div style="display: flex; gap: 8px;">
              <input id="newUsername" placeholder="alice" style="flex: 1;" />
              <input id="newUserPassword" type="password" placeholder="Password (optional)" style="flex: 1;" />
              <button onclick="createUser()">Create</button>
            </div>
          </div>
        </div>
        <div class="items-list" id="usersList"></div>
      </div>

      <!-- Servers Section -->
      <div class="section">
        <h2>Federated Servers</h2>
        <div class="form-row">
          <div class="form-group">
            <label>Server name</label>
            <input id="serverName" placeholder="b" />
          </div>
          <div class="form-group">
            <label>Base URL</label>
            <input id="serverUrl" placeholder="http://server_b:8080" />
          </div>
          <div class="form-group">
            <label>Their server token</label>
            <input id="serverToken" placeholder="auto" />
          </div>
          <div class="form-group" style="justify-content: flex-end;">
            <button onclick="registerServer()" style="margin-top: auto;">Register</button>
          </div>
        </div>
        <div class="items-list" id="serversList"></div>
      </div>

      <!-- Sync All Users Section -->
      <div class="section">
        <h2>All Users Across Servers</h2>
        <div class="form-row">
          <div class="form-group" style="grid-column: 1 / -1;">
            <button onclick="fetchFederatedUsers()" style="width: 100%; margin-bottom: 12px;">Fetch Users From Federated Servers</button>
            <button onclick="syncAllUsers()" style="width: 100%;">View All Users</button>
          </div>
        </div>
        <div class="items-list" id="allUsersList"></div>
      </div>

      <!-- Channels Section -->
      <div class="section">
        <h2>Channels</h2>
        <div class="form-row">
          <div class="form-group" style="grid-column: 1 / -1;">
            <label>Create channel</label>
            <div style="display: flex; gap: 8px;">
              <input id="channelName" placeholder="lobby" style="flex: 1;" />
              <button onclick="createChannel()">Create</button>
            </div>
          </div>
        </div>
        <button onclick="fetchFederatedChannels()" style="width: 100%; margin-bottom: 12px;">Fetch Channels From Federated Servers</button>
        <div class="items-list" id="channelsList"></div>
      </div>

      <!-- Add Channel Member -->
      <div class="section">
        <h2>Add Channel Member</h2>
        <div class="form-row">
          <div class="form-group">
            <label>Channel</label>
            <select id="channelId">
              <option value="">Select a channel...</option>
            </select>
          </div>
          <div class="form-group">
            <label>Member</label>
            <select id="memberUsername">
              <option value="">Select a user...</option>
            </select>
          </div>
          <div class="form-group" style="justify-content: flex-end;">
            <button onclick="addChannelMember()" style="margin-top: auto;">Add member</button>
          </div>
        </div>
      </div>
    </div>

    <!-- Edit User Modal -->
    <div id="editUserModal" class="modal">
      <div class="modal-content">
        <div class="modal-header">
          <h3>Edit User</h3>
          <button onclick="closeModal('editUserModal')">✕</button>
        </div>
        <div class="form-group">
          <label>Username</label>
          <input id="editUserName" />
        </div>
        <div class="form-group">
          <label>Display Name</label>
          <input id="editUserDisplayName" placeholder="Optional display name" />
        </div>
        <div class="form-group">
          <label>Reset Password (leave blank to keep current)</label>
          <input id="editUserPassword" type="password" placeholder="New password" />
        </div>
        <div style="display: flex; gap: 8px; margin-top: 20px;">
          <button onclick="saveUserEdit()" style="flex: 1;">Save</button>
          <button class="secondary" onclick="closeModal('editUserModal')" style="flex: 1;">Cancel</button>
        </div>
      </div>
    </div>

    <!-- Edit Server Modal -->
    <div id="editServerModal" class="modal">
      <div class="modal-content" style="max-width: 600px;">
        <div class="modal-header">
          <h3>Edit Server</h3>
          <button onclick="closeModal('editServerModal')">✕</button>
        </div>
        <div class="form-group">
          <label>Server name</label>
          <input id="editServerName" />
        </div>
        <div class="form-group">
          <label>Base URL</label>
          <input id="editServerUrl" />
        </div>
        <div class="form-group">
          <label>Server token</label>
          <input id="editServerToken" />
        </div>
        <div class="visibility-section">
          <h4>User Visibility</h4>
          <div id="visibilityUserList" class="visibility-list">
            <span style="color: var(--muted); font-size: 12px;">Loading...</span>
          </div>
        </div>
        <div class="visibility-section">
          <h4>Channel Visibility</h4>
          <div id="visibilityChannelList" class="visibility-list">
            <span style="color: var(--muted); font-size: 12px;">Loading...</span>
          </div>
        </div>
        <div style="display: flex; gap: 8px; margin-top: 20px;">
          <button onclick="saveServerEdit()" style="flex: 1;">Save</button>
          <button class="secondary" onclick="closeModal('editServerModal')" style="flex: 1;">Cancel</button>
        </div>
      </div>
    </div>

    <!-- Edit Channel Modal -->
    <div id="editChannelModal" class="modal">
      <div class="modal-content">
        <div class="modal-header">
          <h3>Edit Channel</h3>
          <button onclick="closeModal('editChannelModal')">✕</button>
        </div>
        <div class="form-group">
          <label>Channel name</label>
          <input id="editChannelName" />
        </div>
        <div style="display: flex; gap: 8px; margin-top: 20px;">
          <button onclick="saveChannelEdit()" style="flex: 1;">Save</button>
          <button class="secondary" onclick="closeModal('editChannelModal')" style="flex: 1;">Cancel</button>
        </div>
      </div>
    </div>
  </div>

  <script>
    const app = document.getElementById('app');
    const serverName = document.getElementById('serverBadge').textContent.trim();
    let editingUserId = null;
    let editingServerId = null;
    let editingChannelId = null;
    let allServers = [];

    function saveSessionToken(token) {
      sessionStorage.setItem('sessionToken', token);
      app.classList.add('logged-in');
      refreshAll();
    }

    function getSessionToken() {
      return sessionStorage.getItem('sessionToken') || '';
    }

    function logout() {
      sessionStorage.removeItem('sessionToken');
      app.classList.remove('logged-in');
      document.getElementById('loginUsername').value = '';
      document.getElementById('loginPassword').value = '';
    }

    async function performLogin() {
      try {
        const username = document.getElementById('loginUsername').value.trim();
        const password = document.getElementById('loginPassword').value.trim();
        if (!username || !password) throw new Error('enter username and password');
        const response = await fetch('/admin/login', {
          method: 'POST',
          headers: { 'content-type': 'application/json' },
          body: JSON.stringify({ username, password })
        });
        if (!response.ok) throw new Error('invalid credentials');
        const payload = await response.json();
        saveSessionToken(payload.token);
        document.getElementById('loginError').textContent = '';
      } catch (error) {
        document.getElementById('loginError').textContent = error.message;
      }
    }

    function adminHeaders() {
      return {
        'content-type': 'application/json',
        'x-admin-token': getSessionToken()
      };
    }

    async function requestJson(path, options) {
      const response = await fetch(path, options);
      if (!response.ok) {
        if (response.status === 401) {
          logout();
          throw new Error('Session expired. Please login again.');
        }
        const text = await response.text();
        throw new Error(text || response.statusText);
      }
      return response.json();
    }

    function openModal(id) {
      document.getElementById(id).classList.add('open');
    }

    function closeModal(id) {
      document.getElementById(id).classList.remove('open');
    }

    async function createUser() {
      try {
        const username = document.getElementById('newUsername').value.trim();
        if (!username) throw new Error('enter username');
        const password = document.getElementById('newUserPassword').value;
        const payload = { username };
        if (password) payload.password = password;
        await requestJson('/admin/users', {
          method: 'POST',
          headers: adminHeaders(),
          body: JSON.stringify(payload)
        });
        document.getElementById('newUsername').value = '';
        document.getElementById('newUserPassword').value = '';
        await loadUsers();
      } catch (error) {
        alert('Error: ' + error.message);
      }
    }

    async function loadUsers() {
      try {
        const users = await requestJson('/admin/users', { headers: adminHeaders() });
        const local = users.filter(u => u.is_local);
        const remote = users.filter(u => !u.is_local && u.server_id);
        const remoteByServer = {};
        remote.forEach(u => {
          const server = u.server_id || 'unknown';
          if (!remoteByServer[server]) remoteByServer[server] = [];
          remoteByServer[server].push(u);
        });

        let html = '';
        if (local.length > 0) {
          html += '<h3 style="margin: 20px 0 12px; font-size: 14px; color: var(--muted); text-transform: uppercase;">Local Users</h3>';
          html += local.map(u => `
            <div class="item">
              <div class="item-info">
                <div class="name">${u.username}</div>
                <div class="detail">ID: ${u.id.substring(0, 8)}...</div>
                <div class="detail">Token: ${u.token.substring(0, 8)}...</div>
              </div>
              <div class="item-actions">
                <button class="secondary small" onclick="openEditUserModal('${u.id}', '${u.username}', '${u.display_name || ''}')">Edit</button>
                <button class="danger small" onclick="deleteUserConfirm('${u.id}', '${u.username}')">Delete</button>
              </div>
            </div>
          `).join('');
        }

        Object.entries(remoteByServer).forEach(([serverId, serverUsers]) => {
          if (serverUsers.length > 0) {
            html += `<h3 style="margin: 20px 0 12px; font-size: 14px; color: var(--muted); text-transform: uppercase;">Remote Users (Server: ${serverId.substring(0, 8)}...)</h3>`;
            html += serverUsers.map(u => `
              <div class="item" style="opacity: 0.8;">
                <div class="item-info">
                  <div class="name">${u.username}</div>
                  <div class="detail">ID: ${u.id.substring(0, 8)}...</div>
                  <div class="detail" style="color: var(--accent);">Remote</div>
                </div>
              </div>
            `).join('');
          }
        });

        document.getElementById('usersList').innerHTML = html || '<div class="status">No users yet</div>';

        // Populate the member dropdown for "Add Channel Member"
        const memberSelect = document.getElementById('memberUsername');
        const prevMember = memberSelect.value;
        memberSelect.innerHTML = '<option value="">Select a user...</option>';
        const allUsersList = [...local, ...remote];
        allUsersList.forEach(u => {
          const serverObj = u.server_id ? allServers.find(s => s.id === u.server_id) : null;
          const label = serverObj ? `${u.username}@${serverObj.name}` : u.username;
          memberSelect.innerHTML += `<option value="${label}">${label}</option>`;
        });
        if (prevMember) memberSelect.value = prevMember;
      } catch (error) {
        document.getElementById('usersList').innerHTML = `<div style="color: var(--danger);">Error: ${error.message}</div>`;
      }
    }

    function openEditUserModal(id, name, displayName) {
      editingUserId = id;
      document.getElementById('editUserName').value = name;
      document.getElementById('editUserDisplayName').value = displayName || '';
      document.getElementById('editUserPassword').value = '';
      openModal('editUserModal');
    }

    async function saveUserEdit() {
      try {
        const username = document.getElementById('editUserName').value.trim();
        if (!username) throw new Error('enter username');
        const display_name = document.getElementById('editUserDisplayName').value.trim() || null;
        const password = document.getElementById('editUserPassword').value;
        const payload = { username, display_name };
        if (password) payload.password = password;
        await requestJson(`/admin/users/${editingUserId}`, {
          method: 'PUT',
          headers: adminHeaders(),
          body: JSON.stringify(payload)
        });
        closeModal('editUserModal');
        await loadUsers();
      } catch (error) {
        alert('Error: ' + error.message);
      }
    }

    async function deleteUserConfirm(id, name) {
      if (confirm(`Delete user "${name}"?`)) {
        try {
          await requestJson(`/admin/users/${id}`, {
            method: 'DELETE',
            headers: adminHeaders()
          });
          await loadUsers();
        } catch (error) {
          alert('Error: ' + error.message);
        }
      }
    }

    async function fetchFederatedUsers() {
      try {
        document.getElementById('allUsersList').innerHTML = '<div class="status">Fetching users from federated servers...</div>';
        const synced = await requestJson('/admin/users/sync-federated', {
          method: 'POST',
          headers: adminHeaders(),
          body: JSON.stringify({})
        });
        document.getElementById('allUsersList').innerHTML = `<div class="status">Synced ${synced.length} remote users. Click "View All Users" to see them.</div>`;
      } catch (error) {
        document.getElementById('allUsersList').innerHTML = `<div style="color: var(--danger);">Error: ${error.message}</div>`;
      }
    }

    async function syncAllUsers() {
      try {
        const users = await requestJson('/admin/users', { headers: adminHeaders() });
        const local = users.filter(u => u.is_local);
        const remote = users.filter(u => !u.is_local);
        const remoteByServer = {};
        remote.forEach(u => {
          const server = u.server_id || 'unknown';
          if (!remoteByServer[server]) remoteByServer[server] = [];
          remoteByServer[server].push(u);
        });

        let html = '';
        
        if (local.length > 0) {
          html += '<div style="margin-bottom: 20px;"><h3 style="margin: 0 0 12px; font-size: 14px; color: var(--accent); text-transform: uppercase;">This Server</h3>';
          html += local.map(u => `
            <div class="item">
              <div class="item-info" style="display: flex; justify-content: space-between; align-items: center; width: 100%;">
                <div>
                  <div class="name">${u.username}</div>
                  <div class="detail">Token: ${u.token.substring(0, 12)}...</div>
                </div>
                <div style="color: var(--accent); font-size: 12px; font-weight: 600;">LOCAL</div>
              </div>
            </div>
          `).join('');
          html += '</div>';
        }

        Object.entries(remoteByServer).forEach(([serverId, serverUsers]) => {
          if (serverUsers.length > 0) {
            html += `<div style="margin-bottom: 20px;"><h3 style="margin: 0 0 12px; font-size: 14px; color: var(--muted); text-transform: uppercase;">Server: ${serverId.substring(0, 8)}...</h3>`;
            html += serverUsers.map(u => `
              <div class="item" style="opacity: 0.9;">
                <div class="item-info" style="display: flex; justify-content: space-between; align-items: center; width: 100%;">
                  <div>
                    <div class="name">${u.username}</div>
                  </div>
                  <div style="color: var(--muted); font-size: 12px;">REMOTE</div>
                </div>
              </div>
            `).join('');
            html += '</div>';
          }
        });

        document.getElementById('allUsersList').innerHTML = html || '<div class="status">No users found</div>';
      } catch (error) {
        document.getElementById('allUsersList').innerHTML = `<div style="color: var(--danger);">Error: ${error.message}</div>`;
      }
    }

    async function registerServer() {
      try {
        const name = document.getElementById('serverName').value.trim();
        const base_url = document.getElementById('serverUrl').value.trim();
        const token = document.getElementById('serverToken').value.trim();
        if (!name || !base_url) throw new Error('enter name and URL');
        await requestJson('/admin/servers', {
          method: 'POST',
          headers: adminHeaders(),
          body: JSON.stringify({ name, base_url, token: token || null })
        });
        document.getElementById('serverName').value = '';
        document.getElementById('serverUrl').value = '';
        document.getElementById('serverToken').value = '';
        await loadServers();
      } catch (error) {
        alert('Error: ' + error.message);
      }
    }

    async function loadServers() {
      try {
        const servers = await requestJson('/admin/servers', { headers: adminHeaders() });
        allServers = servers || [];
        const html = servers.map(s => `
          <div class="item">
            <div class="item-info">
              <div class="name">${s.name}</div>
              <div class="detail">URL: ${s.base_url}</div>
              <div class="detail">Token: ${s.token.substring(0, 8)}...</div>
            </div>
            <div class="item-actions">
              <button class="secondary small" onclick="openEditServerModal('${s.id}', '${s.name}', '${s.base_url}', '${s.token}')">Edit</button>
              <button class="danger small" onclick="deleteServerConfirm('${s.id}', '${s.name}')">Delete</button>
            </div>
          </div>
        `).join('');
        document.getElementById('serversList').innerHTML = html || '<div class="status">No servers yet</div>';
      } catch (error) {
        document.getElementById('serversList').innerHTML = `<div style="color: var(--danger);">Error: ${error.message}</div>`;
      }
    }

    async function openEditServerModal(id, name, url, token) {
      editingServerId = id;
      document.getElementById('editServerName').value = name;
      document.getElementById('editServerUrl').value = url;
      document.getElementById('editServerToken').value = token;
      document.getElementById('visibilityUserList').innerHTML = '<span style="color: var(--muted); font-size: 12px;">Loading...</span>';
      document.getElementById('visibilityChannelList').innerHTML = '<span style="color: var(--muted); font-size: 12px;">Loading...</span>';
      openModal('editServerModal');
      try {
        const [visibility, users, channels] = await Promise.all([
          requestJson(`/admin/servers/${id}/visibility`, { headers: adminHeaders() }),
          requestJson('/admin/users', { headers: adminHeaders() }),
          requestJson('/admin/channels', { headers: adminHeaders() }),
        ]);
        const localUsers = users.filter(u => u.is_local);
        const localChannels = channels.filter(c => c.origin_server === serverName);
        const hiddenUserSet = new Set(visibility.hidden_user_ids);
        const hiddenChannelSet = new Set(visibility.hidden_channel_ids);
        document.getElementById('visibilityUserList').innerHTML = localUsers.length === 0
          ? '<span style="color: var(--muted); font-size: 12px;">No local users</span>'
          : localUsers.map(u => `<div class="visibility-item"><input type="checkbox" id="vis-user-${u.id}" data-user-id="${u.id}" ${hiddenUserSet.has(u.id) ? '' : 'checked'} /><label for="vis-user-${u.id}">${u.display_name || u.username}</label></div>`).join('');
        document.getElementById('visibilityChannelList').innerHTML = localChannels.length === 0
          ? '<span style="color: var(--muted); font-size: 12px;">No local channels</span>'
          : localChannels.map(c => `<div class="visibility-item"><input type="checkbox" id="vis-chan-${c.id}" data-channel-id="${c.id}" ${hiddenChannelSet.has(c.id) ? '' : 'checked'} /><label for="vis-chan-${c.id}">${c.name}</label></div>`).join('');
      } catch (e) {
        document.getElementById('visibilityUserList').innerHTML = '<span style="color: var(--danger); font-size: 12px;">Failed to load</span>';
        document.getElementById('visibilityChannelList').innerHTML = '<span style="color: var(--danger); font-size: 12px;">Failed to load</span>';
      }
    }

    async function saveServerEdit() {
      try {
        const name = document.getElementById('editServerName').value.trim();
        const base_url = document.getElementById('editServerUrl').value.trim();
        const token = document.getElementById('editServerToken').value.trim();
        if (!name || !base_url) throw new Error('enter name and URL');
        await requestJson(`/admin/servers/${editingServerId}`, {
          method: 'PUT',
          headers: adminHeaders(),
          body: JSON.stringify({ name, base_url, token: token || null })
        });
        const hidden_user_ids = [];
        document.querySelectorAll('#visibilityUserList input[data-user-id]').forEach(cb => {
          if (!cb.checked) hidden_user_ids.push(cb.dataset.userId);
        });
        const hidden_channel_ids = [];
        document.querySelectorAll('#visibilityChannelList input[data-channel-id]').forEach(cb => {
          if (!cb.checked) hidden_channel_ids.push(cb.dataset.channelId);
        });
        await requestJson(`/admin/servers/${editingServerId}/visibility`, {
          method: 'PUT',
          headers: adminHeaders(),
          body: JSON.stringify({ hidden_user_ids, hidden_channel_ids })
        });
        closeModal('editServerModal');
        await loadServers();
      } catch (error) {
        alert('Error: ' + error.message);
      }
    }

    async function deleteServerConfirm(id, name) {
      if (confirm(`Delete server "${name}"?`)) {
        try {
          await requestJson(`/admin/servers/${id}`, {
            method: 'DELETE',
            headers: adminHeaders()
          });
          await loadServers();
        } catch (error) {
          alert('Error: ' + error.message);
        }
      }
    }

    async function createChannel() {
      try {
        const name = document.getElementById('channelName').value.trim();
        if (!name) throw new Error('enter channel name');
        await requestJson('/admin/channels', {
          method: 'POST',
          headers: adminHeaders(),
          body: JSON.stringify({ name })
        });
        document.getElementById('channelName').value = '';
        await loadChannels();
      } catch (error) {
        alert('Error: ' + error.message);
      }
    }

    async function loadChannels() {
      try {
        const channels = await requestJson('/admin/channels', { headers: adminHeaders() });

        // Populate the channel dropdown for "Add Channel Member"
        const select = document.getElementById('channelId');
        const prevValue = select.value;
        select.innerHTML = '<option value="">Select a channel...</option>';
        channels.forEach(c => {
          const suffix = c.origin_server ? ` (${c.origin_server})` : '';
          select.innerHTML += `<option value="${c.id}">${c.name}${suffix}</option>`;
        });
        if (prevValue) select.value = prevValue;

        // Group channels by origin_server for display
        const byServer = {};
        channels.forEach(c => {
          const key = c.origin_server || 'local';
          if (!byServer[key]) byServer[key] = [];
          byServer[key].push(c);
        });

        let html = '';
        Object.entries(byServer).forEach(([server, serverChannels]) => {
          html += `<h3 style="margin: 20px 0 12px; font-size: 14px; color: var(--muted); text-transform: uppercase;">Channels from ${server}</h3>`;
          html += serverChannels.map(c => `
            <div class="item">
              <div class="item-info">
                <div class="name">${c.name}</div>
                <div class="detail">Origin: ${c.origin_server}</div>
              </div>
              <div class="item-actions">
                <button class="secondary small" onclick="openEditChannelModal('${c.id}', '${c.name}')">Edit</button>
                <button class="danger small" onclick="deleteChannelConfirm('${c.id}', '${c.name}')">Delete</button>
              </div>
            </div>
          `).join('');
        });

        document.getElementById('channelsList').innerHTML = html || '<div class="status">No channels yet</div>';
      } catch (error) {
        document.getElementById('channelsList').innerHTML = `<div style="color: var(--danger);">Error: ${error.message}</div>`;
      }
    }

    function openEditChannelModal(id, name) {
      editingChannelId = id;
      document.getElementById('editChannelName').value = name;
      openModal('editChannelModal');
    }

    async function saveChannelEdit() {
      try {
        const name = document.getElementById('editChannelName').value.trim();
        if (!name) throw new Error('enter channel name');
        await requestJson(`/admin/channels/${editingChannelId}`, {
          method: 'PUT',
          headers: adminHeaders(),
          body: JSON.stringify({ name })
        });
        closeModal('editChannelModal');
        await loadChannels();
      } catch (error) {
        alert('Error: ' + error.message);
      }
    }

    async function deleteChannelConfirm(id, name) {
      if (confirm(`Delete channel "${name}"?`)) {
        try {
          await requestJson(`/admin/channels/${id}`, {
            method: 'DELETE',
            headers: adminHeaders()
          });
          await loadChannels();
        } catch (error) {
          alert('Error: ' + error.message);
        }
      }
    }

    async function fetchFederatedChannels() {
      try {
        document.getElementById('channelsList').innerHTML = '<div class="status">Fetching channels from federated servers...</div>';
        const synced = await requestJson('/admin/channels/sync-federated', {
          method: 'POST',
          headers: adminHeaders(),
          body: JSON.stringify({})
        });
        await loadChannels();
        alert('Synced ' + synced.length + ' new channel(s) from federated servers.');
      } catch (error) {
        alert('Error: ' + error.message);
        await loadChannels();
      }
    }

    async function addChannelMember() {
      try {
        const channelId = document.getElementById('channelId').value;
        const username = document.getElementById('memberUsername').value;
        if (!channelId || !username) throw new Error('select a channel and a user');
        await requestJson(`/admin/channels/${channelId}/members`, {
          method: 'POST',
          headers: adminHeaders(),
          body: JSON.stringify({ username })
        });
        alert('Member added');
      } catch (error) {
        alert('Error: ' + error.message);
      }
    }

    async function loadServerInfo() {
      try {
        const info = await requestJson('/admin/server-info', { headers: adminHeaders() });
        document.getElementById('serverIdentity').innerHTML = `
          <div class="item" style="border-color: var(--accent); background: rgba(73, 211, 176, 0.06);">
            <div class="item-info">
              <div class="name" style="color: var(--accent);">Primary Token (SERVER_TOKEN)</div>
              <div class="detail" style="font-family: monospace; word-break: break-all;">${info.server_token}</div>
            </div>
            <div class="item-actions">
              <button class="secondary small" onclick="copyToken('${info.server_token}', this)">Copy</button>
            </div>
          </div>`;
      } catch (error) {
        document.getElementById('serverIdentity').innerHTML = `<div style="color: var(--danger);">Error: ${error.message}</div>`;
      }
    }

    async function loadFedTokens() {
      try {
        const tokens = await requestJson('/admin/federation-tokens', { headers: adminHeaders() });
        if (tokens.length === 0) {
          document.getElementById('fedTokensList').innerHTML = '<div class="status">No additional tokens yet</div>';
          return;
        }
        const html = tokens.map(t => `
          <div class="item">
            <div class="item-info">
              <div class="name">${t.label}</div>
              <div class="detail" style="font-family: monospace;">${t.token.substring(0, 12)}...</div>
              <div class="detail">Created: ${new Date(t.created_at).toLocaleString()}</div>
            </div>
            <div class="item-actions">
              <button class="secondary small" onclick="copyToken('${t.token}', this)">Copy</button>
              <button class="danger small" onclick="deleteFedToken('${t.id}', '${t.label}')">Delete</button>
            </div>
          </div>
        `).join('');
        document.getElementById('fedTokensList').innerHTML = html;
      } catch (error) {
        document.getElementById('fedTokensList').innerHTML = `<div style="color: var(--danger);">Error: ${error.message}</div>`;
      }
    }

    async function createFedToken() {
      try {
        const label = document.getElementById('fedTokenLabel').value.trim();
        if (!label) throw new Error('enter a label');
        await requestJson('/admin/federation-tokens', {
          method: 'POST',
          headers: adminHeaders(),
          body: JSON.stringify({ label })
        });
        document.getElementById('fedTokenLabel').value = '';
        await loadFedTokens();
      } catch (error) {
        alert('Error: ' + error.message);
      }
    }

    async function deleteFedToken(id, label) {
      if (confirm('Delete federation token "' + label + '"? Remote servers using this token will be rejected.')) {
        try {
          await requestJson('/admin/federation-tokens/' + id, {
            method: 'DELETE',
            headers: adminHeaders()
          });
          await loadFedTokens();
        } catch (error) {
          alert('Error: ' + error.message);
        }
      }
    }

    function copyToken(token, btn) {
      navigator.clipboard.writeText(token).then(() => {
        const orig = btn.textContent;
        btn.textContent = 'Copied!';
        btn.style.color = 'var(--accent)';
        setTimeout(() => { btn.textContent = orig; btn.style.color = ''; }, 1500);
      }).catch(() => {
        prompt('Copy this token:', token);
      });
    }

    async function refreshAll() {
      if (getSessionToken()) {
        await Promise.all([loadServerInfo(), loadFedTokens(), loadServers()]);
        await Promise.all([loadUsers(), loadChannels()]);
      }
    }

    // Check if already logged in
    if (getSessionToken()) {
      app.classList.add('logged-in');
      refreshAll();
    }
  </script>
</body>
</html>"#;
const CHAT_UI_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8"/>
    <meta name="viewport" content="width=device-width, initial-scale=1.0"/>
    <title>BeringShare Chat</title>
    <style>
        @import url('https://fonts.googleapis.com/css2?family=Space+Grotesk:wght@400;500;600;700&display=swap');
        :root {
            --bg: #0b1112;
            --text: #e7f2f4;
            --muted: #9ab2b7;
            --accent: #49d3b0;
            --danger: #ff6b6b;
            --border: #20373c;
            --sidebar-bg: #131516;
            --msg-bg: #0f1819;
        }
        * { box-sizing: border-box; }
        html, body { margin: 0; padding: 0; }
        body {
            font-family: 'Space Grotesk', system-ui, sans-serif;
            color: var(--text);
            background: var(--bg);
            height: 100vh;
            display: flex;
            flex-direction: column;
        }
        .app { display: flex; height: 100vh; }
        .sidebar {
            width: 260px;
            background: var(--sidebar-bg);
            border-right: 1px solid var(--border);
            display: flex;
            flex-direction: column;
            overflow-y: auto;
        }
        .sidebar-header {
            padding: 20px;
            border-bottom: 1px solid var(--border);
            font-weight: 600;
            font-size: 14px;
            text-transform: uppercase;
            color: var(--muted);
        }
        .sidebar-section {
            padding: 8px;
        }
        .sidebar-section-title {
            padding: 8px 12px;
            font-size: 12px;
            text-transform: uppercase;
            color: var(--muted);
            font-weight: 600;
            margin-top: 16px;
        }
        .sidebar-item {
            padding: 10px 12px;
            margin: 2px 8px;
            border-radius: 6px;
            cursor: pointer;
            transition: all 0.2s;
            font-size: 14px;
        }
        .sidebar-item:hover { background: rgba(73, 211, 176, 0.1); color: var(--accent); }
        .sidebar-item.active { background: var(--accent); color: var(--sidebar-bg); font-weight: 600; }
        .sidebar-item.unread { background: rgba(255, 152, 0, 0.3); border-left: 3px solid #ff9800; font-weight: 500; }
        .sidebar-item.unread:hover { background: rgba(255, 152, 0, 0.5); }
        .sidebar-item-content {
            display: flex;
            align-items: center;
            justify-content: space-between;
            gap: 4px;
        }
        .sidebar-item-name {
            flex: 1;
            min-width: 0;
            overflow: hidden;
            text-overflow: ellipsis;
            white-space: nowrap;
        }
        .video-call-btn {
            display: none;
            background: transparent;
            border: none;
            cursor: pointer;
            color: var(--muted);
            font-size: 14px;
            padding: 2px 4px;
            border-radius: 4px;
            align-items: center;
            justify-content: center;
            flex-shrink: 0;
            transition: all 0.2s;
        }
        .video-call-btn:hover {
            color: var(--accent);
            background: rgba(73, 211, 176, 0.15);
        }
        .video-call-btn.online {
            display: inline-flex;
        }
        .new-btn {
            margin: 12px 8px;
            padding: 10px;
            background: var(--accent);
            color: var(--sidebar-bg);
            border: none;
            border-radius: 6px;
            cursor: pointer;
            font-weight: 600;
            font-family: inherit;
            font-size: 13px;
            transition: all 0.2s;
        }
        .new-btn:hover { opacity: 0.9; }
        .main {
            flex: 1;
            display: flex;
            flex-direction: column;
            background: var(--bg);
        }
        .header {
            padding: 16px 24px;
            border-bottom: 1px solid var(--border);
            display: flex;
            justify-content: space-between;
            align-items: center;
            background: var(--sidebar-bg);
        }
        .header-title { font-size: 18px; font-weight: 600; }
        .header-user {
            display: flex;
            align-items: center;
            font-size: 13px;
        }
        .user-menu {
            position: relative;
        }
        .user-menu-toggle {
            background: transparent;
            border: 1px solid var(--border);
            color: var(--text);
            padding: 6px 12px;
            border-radius: 6px;
            cursor: pointer;
            font-family: inherit;
            font-size: 13px;
            font-weight: 500;
            display: flex;
            align-items: center;
            gap: 6px;
            transition: all 0.2s;
        }
        .user-menu-toggle:hover {
            border-color: var(--accent);
            color: var(--accent);
        }
        .user-menu-dropdown {
            display: none;
            position: absolute;
            top: calc(100% + 6px);
            right: 0;
            background: var(--sidebar-bg);
            border: 1px solid var(--border);
            border-radius: 8px;
            min-width: 160px;
            box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
            z-index: 100;
            overflow: hidden;
        }
        .user-menu-dropdown.open {
            display: block;
        }
        .user-menu-item {
            display: block;
            width: 100%;
            padding: 10px 16px;
            background: transparent;
            border: none;
            color: var(--text);
            font-family: inherit;
            font-size: 13px;
            text-align: left;
            cursor: pointer;
            text-decoration: none;
            transition: background 0.15s;
        }
        .user-menu-item:hover {
            background: rgba(73, 211, 176, 0.1);
            color: var(--accent);
        }
        .user-menu-item.logout {
            color: var(--danger);
        }
        .user-menu-item.logout:hover {
            background: rgba(255, 107, 107, 0.1);
            color: var(--danger);
        }
        .user-menu-divider {
            height: 1px;
            background: var(--border);
            margin: 4px 0;
        }
        .messages {
            flex: 1;
            overflow-y: auto;
            padding: 20px 24px;
        }
        .message {
            margin-bottom: 12px;
            display: flex;
            gap: 12px;
        }
        .message.own { justify-content: flex-end; }
        .message-content {
            max-width: 60%;
            padding: 10px 14px;
            border-radius: 8px;
            background: var(--msg-bg);
            border: 1px solid var(--border);
        }
        .message.own .message-content {
            background: rgba(73, 211, 176, 0.15);
            border-color: var(--accent);
        }
        .message-author {
            font-size: 12px;
            color: var(--accent);
            font-weight: 600;
            margin-bottom: 4px;
        }
        .message-text { font-size: 14px; }
        .message-time {
            font-size: 11px;
            color: var(--muted);
            margin-top: 4px;
        }
        .input-area {
            padding: 16px 24px;
            border-top: 1px solid var(--border);
            display: none;
            gap: 12px;
        }
        .input-area.active { display: flex; }
        .input-area input {
            flex: 1;
            padding: 12px;
            background: var(--msg-bg);
            border: 1px solid var(--border);
            border-radius: 6px;
            color: var(--text);
            font-family: inherit;
            font-size: 13px;
        }
        .input-area input:focus { outline: none; border-color: var(--accent); }
        .send-btn {
            padding: 12px 20px;
            background: var(--accent);
            color: var(--sidebar-bg);
            border: none;
            border-radius: 6px;
            cursor: pointer;
            font-weight: 600;
            font-family: inherit;
            font-size: 13px;
        }
        .send-btn:hover { opacity: 0.9; }
        .gif-btn {
            padding: 12px 14px;
            background: transparent;
            color: var(--accent);
            border: 1px solid var(--accent);
            border-radius: 6px;
            cursor: pointer;
            font-weight: 700;
            font-family: inherit;
            font-size: 11px;
            letter-spacing: 0.5px;
            transition: all 0.2s;
        }
        .gif-btn:hover { background: rgba(73, 211, 176, 0.12); }
        .gif-picker {
            display: none;
            position: fixed;
            top: 0; left: 0;
            width: 100%; height: 100%;
            background: rgba(0, 0, 0, 0.7);
            z-index: 2000;
            align-items: center;
            justify-content: center;
        }
        .gif-picker.active { display: flex; }
        .gif-picker-panel {
            background: var(--sidebar-bg);
            border: 1px solid var(--border);
            border-radius: 12px;
            padding: 20px;
            width: 90%;
            max-width: 600px;
            max-height: 80vh;
            display: flex;
            flex-direction: column;
        }
        .gif-picker-header {
            display: flex;
            gap: 8px;
            margin-bottom: 12px;
        }
        .gif-picker-header input {
            flex: 1;
            padding: 10px 12px;
            background: rgba(255,255,255,0.06);
            border: 1px solid var(--border);
            border-radius: 8px;
            color: var(--text);
            font-family: inherit;
            font-size: 14px;
            outline: none;
        }
        .gif-picker-header input:focus { border-color: var(--accent); }
        .gif-picker-close {
            padding: 10px 14px;
            background: transparent;
            border: 1px solid var(--border);
            border-radius: 8px;
            color: var(--muted);
            cursor: pointer;
            font-size: 16px;
        }
        .gif-picker-close:hover { color: var(--text); border-color: var(--text); }
        .gif-grid {
            display: grid;
            grid-template-columns: repeat(2, 1fr);
            gap: 8px;
            overflow-y: auto;
            flex: 1;
            min-height: 200px;
        }
        .gif-grid img {
            width: 100%;
            height: 140px;
            object-fit: cover;
            border-radius: 8px;
            cursor: pointer;
            transition: transform 0.15s, opacity 0.15s;
            display: block;
        }
        .gif-grid img:hover { transform: scale(1.03); opacity: 0.85; }
        .gif-grid-empty {
            grid-column: 1 / -1;
            text-align: center;
            color: var(--muted);
            padding: 40px 0;
            font-size: 14px;
        }
        .gif-attribution {
            text-align: center;
            color: var(--muted);
            font-size: 11px;
            margin-top: 8px;
        }
        .gif-msg-img {
            max-width: 100%;
            max-height: 300px;
            border-radius: 8px;
            display: block;
        }
        .modal {
            display: none;
            position: fixed;
            top: 0; left: 0;
            width: 100%; height: 100%;
            background: rgba(0, 0, 0, 0.7);
            z-index: 1000;
            align-items: center;
            justify-content: center;
        }
        .modal.active { display: flex; }
        .modal-content {
            background: var(--sidebar-bg);
            border: 1px solid var(--border);
            border-radius: 12px;
            padding: 24px;
            max-width: 400px;
            width: 90%;
        }
        .modal h2 { margin: 0 0 16px; font-size: 18px; }
        .modal input {
            width: 100%;
            padding: 10px;
            background: var(--msg-bg);
            border: 1px solid var(--border);
            border-radius: 6px;
            color: var(--text);
            font-family: inherit;
            margin-bottom: 16px;
            font-size: 13px;
        }
        .modal-buttons {
            display: flex;
            gap: 12px;
            justify-content: flex-end;
        }
        .modal button {
            padding: 8px 16px;
            border: none;
            border-radius: 4px;
            cursor: pointer;
            font-weight: 600;
            font-family: inherit;
            font-size: 13px;
        }
        .modal .btn-primary { background: var(--accent); color: var(--sidebar-bg); }
        .modal .btn-secondary { background: var(--border); color: var(--text); }
        .login-screen {
            display: flex;
            align-items: center;
            justify-content: center;
            min-height: 100vh;
            padding: 20px;
        }
        .login-card {
            background: rgba(19, 36, 41, 0.95);
            border: 1px solid var(--border);
            border-radius: 18px;
            padding: 40px;
            max-width: 380px;
            width: 100%;
        }
        .login-card h1 {
            margin: 0 0 8px;
            font-size: 28px;
        }
        .login-card p {
            color: var(--muted);
            margin: 0 0 24px;
            font-size: 14px;
        }
        .form-group {
            display: flex;
            flex-direction: column;
            margin-bottom: 16px;
        }
        .form-group label {
            font-size: 12px;
            color: var(--muted);
            margin-bottom: 6px;
            text-transform: uppercase;
            font-weight: 600;
        }
        .form-group input {
            padding: 12px;
            background: var(--msg-bg);
            border: 1px solid var(--border);
            border-radius: 6px;
            color: var(--text);
            font-family: inherit;
            font-size: 14px;
        }
        .form-group input:focus {
            outline: none;
            border-color: var(--accent);
        }
        .login-btn {
            width: 100%;
            padding: 12px;
            background: var(--accent);
            color: var(--sidebar-bg);
            border: none;
            border-radius: 6px;
            cursor: pointer;
            font-weight: 600;
            font-family: inherit;
            font-size: 14px;
            margin-top: 8px;
        }
        .login-btn:hover { opacity: 0.9; }
        .app { display: none; }
        .app.active { display: flex; }

        /* Video Call Overlay */
        .call-overlay {
            display: none;
            position: fixed;
            inset: 0;
            background: #0b1112;
            z-index: 2000;
            flex-direction: column;
            align-items: center;
            justify-content: center;
        }
        .call-overlay.active { display: flex; }
        .call-status {
            position: absolute;
            top: 24px;
            left: 50%;
            transform: translateX(-50%);
            color: var(--muted);
            font-size: 16px;
            font-weight: 500;
            z-index: 2001;
            background: rgba(0,0,0,0.6);
            padding: 8px 20px;
            border-radius: 20px;
        }
        .call-videos { position: relative; width: 100%; height: 100%; }
        .remote-video {
            width: 100%;
            height: 100%;
            object-fit: contain;
            background: #000;
        }
        .local-video {
            position: absolute;
            bottom: 100px;
            right: 24px;
            width: 200px;
            height: 150px;
            border-radius: 12px;
            border: 2px solid var(--border);
            object-fit: cover;
            transform: scaleX(-1);
            background: #1a1a1a;
            z-index: 2001;
        }
        .call-controls {
            position: absolute;
            bottom: 32px;
            left: 50%;
            transform: translateX(-50%);
            display: flex;
            gap: 16px;
            z-index: 2001;
        }
        .call-ctrl-btn {
            padding: 14px 24px;
            border-radius: 50px;
            border: 1px solid var(--border);
            background: rgba(19, 36, 41, 0.9);
            color: var(--text);
            font-size: 16px;
            cursor: pointer;
            font-family: inherit;
            font-weight: 600;
            transition: all 0.2s;
        }
        .call-ctrl-btn:hover {
            background: rgba(73, 211, 176, 0.15);
            border-color: var(--accent);
        }
        .call-ctrl-btn.muted {
            background: rgba(255, 107, 107, 0.2);
            border-color: var(--danger);
            color: var(--danger);
        }
        .call-hangup-btn {
            background: var(--danger);
            border-color: var(--danger);
            color: #fff;
        }
        .call-hangup-btn:hover { background: #ff4444; }
        .call-accept-btn {
            background: var(--accent);
            border-color: var(--accent);
            color: #08211c;
        }
        .call-accept-btn:hover { background: #3bc49e; }
        .incoming-call {
            position: absolute;
            top: 50%;
            left: 50%;
            transform: translate(-50%, -50%);
            background: rgba(19, 36, 41, 0.95);
            border: 1px solid var(--border);
            border-radius: 18px;
            padding: 40px;
            text-align: center;
            z-index: 2002;
            box-shadow: 0 20px 60px rgba(0, 0, 0, 0.45);
        }
        .incoming-call-text {
            font-size: 18px;
            font-weight: 600;
            margin-bottom: 24px;
        }
        .incoming-call-buttons {
            display: flex;
            gap: 16px;
            justify-content: center;
        }
        .call-ctrl-btn.screen-active {
            background: rgba(73, 211, 176, 0.25);
            border-color: var(--accent);
            color: var(--accent);
        }
        .call-videos.screen-share-mode {
            overflow: auto;
        }
        .call-videos.screen-share-mode .remote-video {
            transform-origin: top left;
        }
        .screen-share-controls {
            display: none;
            position: absolute;
            top: 24px;
            right: 24px;
            z-index: 2001;
            background: rgba(0, 0, 0, 0.7);
            border: 1px solid var(--border);
            border-radius: 10px;
            padding: 8px 12px;
            gap: 8px;
            align-items: center;
        }
        .screen-share-controls.active {
            display: flex;
        }
        .screen-share-controls button {
            padding: 6px 12px;
            border-radius: 6px;
            border: 1px solid var(--border);
            background: rgba(19, 36, 41, 0.9);
            color: var(--text);
            font-size: 14px;
            cursor: pointer;
            font-family: inherit;
            font-weight: 600;
            transition: all 0.2s;
        }
        .screen-share-controls button:hover {
            background: rgba(73, 211, 176, 0.15);
            border-color: var(--accent);
        }
        .screen-share-controls .zoom-level {
            color: var(--muted);
            font-size: 13px;
            min-width: 48px;
            text-align: center;
            font-weight: 600;
        }

        /* Group Call Overlay */
        .group-call-overlay {
            display: none;
            position: fixed;
            inset: 0;
            background: #0b1112;
            z-index: 2000;
            flex-direction: column;
        }
        .group-call-overlay.active { display: flex; }
        .group-call-header {
            display: flex;
            align-items: center;
            gap: 16px;
            padding: 16px 24px;
            background: rgba(0,0,0,0.4);
            font-weight: 600;
            font-size: 16px;
        }
        .group-call-header #group-call-count {
            color: var(--muted);
            font-size: 14px;
            font-weight: 400;
        }
        .group-call-header .header-leave-btn {
            margin-left: auto;
            padding: 8px 20px;
            border-radius: 8px;
            border: none;
            background: var(--danger);
            color: #fff;
            font-size: 13px;
            font-weight: 600;
            cursor: pointer;
            font-family: inherit;
            transition: background 0.2s;
        }
        .group-call-header .header-leave-btn:hover { background: #ff4444; }
        .group-call-grid {
            flex: 1;
            display: grid;
            gap: 8px;
            padding: 8px;
            grid-template-columns: 1fr;
            align-content: center;
        }
        .group-call-grid.cols-2 { grid-template-columns: 1fr 1fr; }
        .group-call-grid.cols-3 { grid-template-columns: 1fr 1fr 1fr; }
        .group-call-grid.cols-4 { grid-template-columns: 1fr 1fr 1fr 1fr; }
        .group-call-tile {
            position: relative;
            background: #1a1a1a;
            border-radius: 12px;
            overflow: hidden;
            min-height: 180px;
            aspect-ratio: 16 / 9;
        }
        .group-call-tile video {
            width: 100%;
            height: 100%;
            object-fit: cover;
            display: block;
        }
        .group-call-tile.local video {
            transform: scaleX(-1);
        }
        .group-call-tile .tile-label {
            position: absolute;
            bottom: 8px;
            left: 8px;
            background: rgba(0,0,0,0.6);
            color: #fff;
            padding: 4px 10px;
            border-radius: 6px;
            font-size: 13px;
            font-weight: 500;
        }
        .group-call-controls {
            display: flex;
            justify-content: center;
            gap: 16px;
            padding: 16px 24px;
            background: rgba(0,0,0,0.4);
        }
        .group-call-controls button {
            padding: 14px 24px;
            border-radius: 50px;
            border: 1px solid var(--border);
            background: rgba(19, 36, 41, 0.9);
            color: var(--text);
            font-size: 16px;
            cursor: pointer;
            font-family: inherit;
            font-weight: 600;
            transition: all 0.2s;
        }
        .group-call-controls button:hover {
            background: rgba(73, 211, 176, 0.15);
            border-color: var(--accent);
        }
        .group-call-controls button.muted {
            background: rgba(255, 107, 107, 0.2);
            border-color: var(--danger);
            color: var(--danger);
        }
        .group-call-controls button.screen-active {
            background: rgba(73, 211, 176, 0.25);
            border-color: var(--accent);
            color: var(--accent);
        }
        .group-call-controls .group-call-leave-btn {
            background: var(--danger);
            border-color: var(--danger);
            color: #fff;
        }
        .group-call-controls .group-call-leave-btn:hover { background: #ff4444; }
        .channel-call-btn {
            margin-left: 12px;
            padding: 6px 14px;
            border-radius: 8px;
            border: 1px solid var(--accent);
            background: rgba(73, 211, 176, 0.1);
            color: var(--accent);
            font-size: 13px;
            cursor: pointer;
            font-family: inherit;
            font-weight: 600;
            transition: all 0.2s;
        }
        .channel-call-btn:hover { background: rgba(73, 211, 176, 0.25); }
        .channel-call-btn.in-call {
            border-color: var(--danger);
            background: rgba(255, 107, 107, 0.15);
            color: var(--danger);
        }
        .channel-call-btn.in-call:hover { background: rgba(255, 107, 107, 0.3); }
        .call-badge {
            display: inline-block;
            background: var(--accent);
            color: #08211c;
            font-size: 11px;
            font-weight: 700;
            padding: 2px 7px;
            border-radius: 10px;
            margin-left: 6px;
        }
    </style>
</head>
<body>
    <div class="login-screen" id="login-screen">
        <div class="login-card">
            <h1>BeringShare</h1>
            <p>Chat with friends across federated servers</p>
            <div class="form-group">
                <label>Username</label>
                <input type="text" id="login-username" placeholder="Your username"/>
            </div>
            <div class="form-group">
                <label>Password</label>
                <input type="password" id="login-password" placeholder="Password"/>
            </div>
            <div id="login-error" style="color:#ff6b6b;font-size:12px;margin-bottom:8px;"></div>
            <button class="login-btn" id="login-btn">Login</button>
        </div>
    </div>

    <div class="app" id="app">
        <div class="sidebar">
            <div class="sidebar-header">BeringShare Chat</div>
            <button class="new-btn" id="new-channel-btn">+ New Channel</button>
            <div class="sidebar-section">
                <div class="sidebar-section-title">Direct Messages</div>
                <div id="users-list"></div>
            </div>
            <div class="sidebar-section">
                <div class="sidebar-section-title">Channels</div>
                <div id="channels-list"></div>
            </div>
        </div>
        <div class="main">
            <div class="header">
                <div class="header-title" id="current-title">Select a conversation</div>
                <div class="header-user">
                    <div class="user-menu">
                        <button class="user-menu-toggle" id="user-menu-toggle">
                            <span id="current-user"></span> &#9662;
                        </button>
                        <div class="user-menu-dropdown" id="user-menu-dropdown">
                            <a href="/admin/ui" class="user-menu-item">Admin</a>
                            <a href="/chat/settings" class="user-menu-item">Settings</a>
                            <div class="user-menu-divider"></div>
                            <button class="user-menu-item logout" id="logout-btn">Logout</button>
                        </div>
                    </div>
                </div>
            </div>
            <div class="messages" id="messages"></div>
            <div class="input-area" id="input-area">
                <input type="text" id="msg-input" placeholder="Type a message... (/gif to search GIFs)"/>
                <button class="gif-btn" id="gif-btn" onclick="openGifPicker()" title="Send a GIF">GIF</button>
                <button class="send-btn" id="send-btn">Send</button>
            </div>
        </div>
    </div>

    <div class="gif-picker" id="gif-picker">
        <div class="gif-picker-panel">
            <div class="gif-picker-header">
                <input type="text" id="gif-search-input" placeholder="Search GIFs..." />
                <button class="gif-picker-close" onclick="closeGifPicker()">&times;</button>
            </div>
            <div class="gif-grid" id="gif-grid">
                <div class="gif-grid-empty">Type to search for GIFs</div>
            </div>
            <div class="gif-attribution">Powered by Tenor</div>
        </div>
    </div>

    <div class="modal" id="new-channel-modal">
        <div class="modal-content">
            <h2>Create Channel</h2>
            <input type="text" id="channel-name-input" placeholder="Channel name"/>
            <div class="modal-buttons">
                <button class="btn-secondary" id="modal-cancel">Cancel</button>
                <button class="btn-primary" id="modal-create">Create</button>
            </div>
        </div>
    </div>

    <!-- Video Call Overlay -->
    <div id="call-overlay" class="call-overlay">
        <div class="call-status" id="call-status">Calling...</div>
        <div class="call-videos" id="call-videos">
            <video id="remote-video" class="remote-video" autoplay playsinline></video>
            <video id="local-video" class="local-video" autoplay playsinline muted></video>
            <div class="screen-share-controls" id="screen-share-controls">
                <button id="zoom-out-btn" title="Zoom Out">-</button>
                <span class="zoom-level" id="zoom-level">100%</span>
                <button id="zoom-in-btn" title="Zoom In">+</button>
                <button id="zoom-fit-btn" title="Fit to Screen">Fit</button>
            </div>
        </div>
        <div class="call-controls">
            <button id="call-toggle-video" class="call-ctrl-btn" title="Toggle Camera">📹</button>
            <button id="call-toggle-audio" class="call-ctrl-btn" title="Toggle Mic">🎤</button>
            <button id="call-toggle-screen" class="call-ctrl-btn" title="Share Screen">🖥️</button>
            <button id="call-hangup" class="call-ctrl-btn call-hangup-btn" title="Hang Up">End</button>
        </div>
        <div id="incoming-call" class="incoming-call" style="display: none;">
            <div class="incoming-call-text" id="incoming-call-text">Incoming call from ...</div>
            <div class="incoming-call-buttons">
                <button id="call-accept" class="call-ctrl-btn call-accept-btn">Accept</button>
                <button id="call-reject" class="call-ctrl-btn call-hangup-btn">Reject</button>
            </div>
        </div>
    </div>

    <!-- Group Call Overlay -->
    <div id="group-call-overlay" class="group-call-overlay">
        <div class="group-call-header">
            <span id="group-call-channel-name"># channel</span>
            <span id="group-call-count">0 participants</span>
            <button class="header-leave-btn" onclick="leaveGroupCall()">Leave Call</button>
        </div>
        <div class="group-call-grid" id="group-call-grid"></div>
        <div class="group-call-controls">
            <button id="group-toggle-video" title="Toggle Camera">📹</button>
            <button id="group-toggle-audio" title="Toggle Mic">🎤</button>
            <button id="group-toggle-screen" title="Share Screen">🖥️</button>
            <button id="group-call-leave" class="group-call-leave-btn" title="Leave Call">Leave</button>
        </div>
    </div>


    <script>
        let currentUser = null;
        let currentTarget = null;
        let currentTargetType = null;
        let allUsers = [];
        let unreadDMs = new Set(); // Track users with unread messages
        let eventSource = null;

        // ===== WebRTC Call State =====
        let callState = 'idle';
        let peerConnection = null;
        let localStream = null;
        let callTarget = null;
        let callTargetUserId = null;
        let iceCandidateQueue = [];
        let videoMuted = false;
        let audioMuted = false;
        let screenSharing = false;
        let screenStream = null;
        let remoteScreenShare = false;
        let screenZoomLevel = 1;

        // ===== Group Call State =====
        let groupCallActive = false;
        let groupCallChannelId = null;
        let groupCallChannelName = null;
        let groupCallLocalStream = null;
        let groupCallPeers = {}; // key: "user@server" -> {pc, videoEl, tile, iceCandidateQueue}
        let groupCallVideoMuted = false;
        let groupCallAudioMuted = false;
        let groupCallScreenSharing = false;
        let groupCallScreenStream = null;
        let groupCallLocalServerName = '';
        let activeChannelCalls = {}; // channel_id -> count

        const STUN_CONFIG = {
            iceServers: [
                { urls: 'stun:stun.l.google.com:19302' },
                { urls: 'stun:stun1.l.google.com:19302' }
            ]
        };

        function setCallState(newState) {
            addDebugLog('CALL state: ' + callState + ' -> ' + newState);
            callState = newState;
        }

        function callTargetString(userObj) {
            if (userObj.server_name) {
                return userObj.username + '@' + userObj.server_name;
            }
            return userObj.username;
        }

        async function sendCallSignal(targetStr, signalType, payload) {
            addDebugLog('CALL signal -> ' + signalType + ' to ' + targetStr);
            await requestJson('/api/call/signal', 'POST', {
                target: targetStr,
                signal_type: signalType,
                payload: payload || ''
            });
        }

        async function getCallMedia() {
            const videoDeviceId = localStorage.getItem('beringshare_videoDeviceId');
            const audioDeviceId = localStorage.getItem('beringshare_audioDeviceId');
            const constraints = {
                video: videoDeviceId ? { deviceId: { exact: videoDeviceId } } : true,
                audio: audioDeviceId ? { deviceId: { exact: audioDeviceId } } : true
            };
            try {
                return await navigator.mediaDevices.getUserMedia(constraints);
            } catch (e) {
                addDebugLog('CALL getUserMedia failed: ' + e.message);
                try {
                    return await navigator.mediaDevices.getUserMedia({ video: true, audio: true });
                } catch (e2) {
                    addDebugLog('CALL getUserMedia fallback failed: ' + e2.message);
                    alert('Could not access camera/microphone. Check permissions.');
                    return null;
                }
            }
        }

        function createPeerConnection() {
            const pc = new RTCPeerConnection(STUN_CONFIG);

            pc.onicecandidate = function(event) {
                if (event.candidate && callTarget) {
                    addDebugLog('CALL ICE candidate generated');
                    var targetStr = callTargetString(callTarget);
                    sendCallSignal(targetStr, 'ice-candidate', JSON.stringify(event.candidate));
                }
            };

            pc.oniceconnectionstatechange = function() {
                addDebugLog('CALL ICE state: ' + pc.iceConnectionState);
                if (pc.iceConnectionState === 'connected' || pc.iceConnectionState === 'completed') {
                    setCallState('connected');
                    document.getElementById('call-status').textContent = 'Connected';
                }
                if (pc.iceConnectionState === 'disconnected' || pc.iceConnectionState === 'failed') {
                    addDebugLog('CALL ICE disconnected/failed, ending call');
                    endCall(false);
                }
            };

            pc.ontrack = function(event) {
                addDebugLog('CALL remote track received: ' + event.track.kind);
                var remoteVideo = document.getElementById('remote-video');
                if (remoteVideo.srcObject !== event.streams[0]) {
                    remoteVideo.srcObject = event.streams[0];
                }
            };

            return pc;
        }

        function showCallOverlay() {
            document.getElementById('call-overlay').classList.add('active');
        }

        function hideCallOverlay() {
            document.getElementById('call-overlay').classList.remove('active');
        }

        function resetCallState() {
            if (peerConnection) {
                peerConnection.close();
                peerConnection = null;
            }
            if (localStream) {
                localStream.getTracks().forEach(function(t) { t.stop(); });
                localStream = null;
            }
            if (screenStream) {
                screenStream.getTracks().forEach(function(t) { t.stop(); });
                screenStream = null;
            }
            screenSharing = false;
            remoteScreenShare = false;
            screenZoomLevel = 1;
            document.getElementById('remote-video').srcObject = null;
            document.getElementById('local-video').srcObject = null;
            document.getElementById('incoming-call').style.display = 'none';
            document.getElementById('call-status').textContent = '';
            document.getElementById('call-toggle-screen').classList.remove('screen-active');
            document.getElementById('call-videos').classList.remove('screen-share-mode');
            document.getElementById('screen-share-controls').classList.remove('active');
            var remoteVid = document.getElementById('remote-video');
            remoteVid.style.transform = '';
            hideCallOverlay();
            callState = 'idle';
            callTarget = null;
            callTargetUserId = null;
            iceCandidateQueue = [];
            videoMuted = false;
            audioMuted = false;
            document.getElementById('call-toggle-video').classList.remove('muted');
            document.getElementById('call-toggle-audio').classList.remove('muted');
        }

        async function startCall(userObj) {
            if (groupCallActive) {
                alert('Leave the group call first');
                return;
            }
            if (callState !== 'idle') {
                addDebugLog('CALL startCall ignored, state: ' + callState);
                return;
            }

            addDebugLog('CALL starting call to ' + userObj.username);
            callTarget = userObj;
            callTargetUserId = userObj.id;
            iceCandidateQueue = [];

            localStream = await getCallMedia();
            if (!localStream) {
                callTarget = null;
                callTargetUserId = null;
                return;
            }

            setCallState('calling');
            showCallOverlay();
            document.getElementById('call-status').textContent = 'Calling ' + userObj.username + '...';
            document.getElementById('local-video').srcObject = localStream;

            peerConnection = createPeerConnection();
            localStream.getTracks().forEach(function(track) {
                peerConnection.addTrack(track, localStream);
            });

            try {
                var offer = await peerConnection.createOffer();
                await peerConnection.setLocalDescription(offer);
                addDebugLog('CALL offer created, sending');
                var targetStr = callTargetString(userObj);
                await sendCallSignal(targetStr, 'offer', JSON.stringify(offer));
            } catch (e) {
                addDebugLog('CALL createOffer failed: ' + e.message);
                endCall(false);
            }
        }

        async function handleWebRtcSignal(signal) {
            addDebugLog('CALL signal <- ' + signal.signal_type + ' from ' + signal.from_user.username + '@' + signal.from_user.server);

            switch (signal.signal_type) {
                case 'offer': await handleOffer(signal); break;
                case 'answer': await handleAnswer(signal); break;
                case 'ice-candidate': await handleIceCandidate(signal); break;
                case 'hangup': handleHangup(signal); break;
                case 'reject': handleReject(signal); break;
                case 'busy': handleBusy(signal); break;
                case 'screen-share-start': enableRemoteScreenShare(); break;
                case 'screen-share-stop': disableRemoteScreenShare(); break;
                case 'group-offer': await handleGroupOffer(signal); break;
                case 'group-answer': await handleGroupAnswer(signal); break;
                case 'group-ice-candidate': await handleGroupIceCandidate(signal); break;
                default: addDebugLog('CALL unknown signal type: ' + signal.signal_type);
            }
        }

        async function handleOffer(signal) {
            if (callState !== 'idle') {
                addDebugLog('CALL busy, rejecting incoming offer');
                var fromStr = signal.from_user.username +
                    (signal.from_user.server ? '@' + signal.from_user.server : '');
                await sendCallSignal(fromStr, 'busy', '');
                return;
            }

            var callerUser = allUsers.find(function(u) {
                if (signal.from_user.server) {
                    return u.username === signal.from_user.username &&
                           u.server_name === signal.from_user.server;
                }
                return u.username === signal.from_user.username && !u.server_name;
            });

            callTarget = {
                username: signal.from_user.username,
                server_name: signal.from_user.server || null,
                id: callerUser ? callerUser.id : null
            };
            callTargetUserId = callerUser ? callerUser.id : null;
            iceCandidateQueue = [];
            callTarget._pendingOffer = JSON.parse(signal.payload);

            setCallState('ringing');
            showCallOverlay();
            document.getElementById('call-status').textContent = '';
            document.getElementById('incoming-call-text').textContent =
                'Incoming call from ' + signal.from_user.username;
            document.getElementById('incoming-call').style.display = 'block';
        }

        async function acceptCall() {
            if (callState !== 'ringing' || !callTarget || !callTarget._pendingOffer) return;

            addDebugLog('CALL accepting call from ' + callTarget.username);
            document.getElementById('incoming-call').style.display = 'none';

            localStream = await getCallMedia();
            if (!localStream) {
                endCall(true);
                return;
            }

            document.getElementById('local-video').srcObject = localStream;
            document.getElementById('call-status').textContent = 'Connecting...';

            peerConnection = createPeerConnection();
            localStream.getTracks().forEach(function(track) {
                peerConnection.addTrack(track, localStream);
            });

            try {
                await peerConnection.setRemoteDescription(
                    new RTCSessionDescription(callTarget._pendingOffer)
                );

                for (var i = 0; i < iceCandidateQueue.length; i++) {
                    await peerConnection.addIceCandidate(new RTCIceCandidate(iceCandidateQueue[i]));
                }
                iceCandidateQueue = [];

                var answer = await peerConnection.createAnswer();
                await peerConnection.setLocalDescription(answer);
                addDebugLog('CALL answer created, sending');
                var targetStr = callTargetString(callTarget);
                await sendCallSignal(targetStr, 'answer', JSON.stringify(answer));

                setCallState('connected');
                document.getElementById('call-status').textContent = 'Connected';
            } catch (e) {
                addDebugLog('CALL acceptCall error: ' + e.message);
                endCall(true);
            }

            delete callTarget._pendingOffer;
        }

        async function rejectCall() {
            if (callState !== 'ringing' || !callTarget) return;
            addDebugLog('CALL rejecting call from ' + callTarget.username);
            document.getElementById('incoming-call').style.display = 'none';
            var targetStr = callTargetString(callTarget);
            await sendCallSignal(targetStr, 'reject', '');
            resetCallState();
        }

        async function handleAnswer(signal) {
            if (callState !== 'calling' || !peerConnection) {
                addDebugLog('CALL answer received but not in calling state, ignoring');
                return;
            }
            addDebugLog('CALL answer received, setting remote description');
            try {
                var sdp = JSON.parse(signal.payload);
                await peerConnection.setRemoteDescription(new RTCSessionDescription(sdp));

                for (var i = 0; i < iceCandidateQueue.length; i++) {
                    await peerConnection.addIceCandidate(new RTCIceCandidate(iceCandidateQueue[i]));
                }
                iceCandidateQueue = [];

                setCallState('connected');
                document.getElementById('call-status').textContent = 'Connected';
            } catch (e) {
                addDebugLog('CALL handleAnswer error: ' + e.message);
                endCall(true);
            }
        }

        async function handleIceCandidate(signal) {
            var candidate = JSON.parse(signal.payload);
            if (peerConnection && peerConnection.remoteDescription) {
                try {
                    await peerConnection.addIceCandidate(new RTCIceCandidate(candidate));
                } catch (e) {
                    addDebugLog('CALL addIceCandidate error: ' + e.message);
                }
            } else {
                addDebugLog('CALL ICE candidate queued (no remote desc yet)');
                iceCandidateQueue.push(candidate);
            }
        }

        function handleHangup(signal) {
            addDebugLog('CALL remote hangup');
            document.getElementById('call-status').textContent = 'Call ended';
            setTimeout(function() { resetCallState(); }, 1500);
        }

        function handleReject(signal) {
            addDebugLog('CALL call rejected by remote');
            document.getElementById('call-status').textContent = 'Call rejected';
            setTimeout(function() { resetCallState(); }, 2000);
        }

        function handleBusy(signal) {
            addDebugLog('CALL remote is busy');
            document.getElementById('call-status').textContent = 'User is busy';
            setTimeout(function() { resetCallState(); }, 2000);
        }

        async function endCall(sendSignal) {
            addDebugLog('CALL endCall, sendSignal=' + sendSignal);
            if (sendSignal && callTarget && (callState === 'calling' || callState === 'connected' || callState === 'ringing')) {
                var targetStr = callTargetString(callTarget);
                try {
                    await sendCallSignal(targetStr, 'hangup', '');
                } catch (e) {
                    addDebugLog('CALL hangup signal failed: ' + e.message);
                }
            }
            document.getElementById('call-status').textContent = 'Call ended';
            setTimeout(function() { resetCallState(); }, 1000);
        }

        function toggleVideo() {
            if (!localStream) return;
            localStream.getVideoTracks().forEach(function(t) { t.enabled = !t.enabled; });
            videoMuted = !videoMuted;
            document.getElementById('call-toggle-video').classList.toggle('muted', videoMuted);
            addDebugLog('CALL video ' + (videoMuted ? 'muted' : 'unmuted'));
        }

        function toggleAudio() {
            if (!localStream) return;
            localStream.getAudioTracks().forEach(function(t) { t.enabled = !t.enabled; });
            audioMuted = !audioMuted;
            document.getElementById('call-toggle-audio').classList.toggle('muted', audioMuted);
            addDebugLog('CALL audio ' + (audioMuted ? 'muted' : 'unmuted'));
        }

        async function toggleScreenShare() {
            if (callState !== 'connected' || !peerConnection) {
                addDebugLog('CALL screen share ignored, not in connected state');
                return;
            }

            if (screenSharing) {
                stopScreenShare();
                return;
            }

            try {
                screenStream = await navigator.mediaDevices.getDisplayMedia({ video: true });
                var screenTrack = screenStream.getVideoTracks()[0];

                var videoSender = peerConnection.getSenders().find(function(s) {
                    return s.track && s.track.kind === 'video';
                });
                if (videoSender) {
                    await videoSender.replaceTrack(screenTrack);
                }

                screenSharing = true;
                document.getElementById('call-toggle-screen').classList.add('screen-active');
                addDebugLog('CALL screen sharing started');

                // Notify remote side
                var targetStr = callTargetString(callTarget);
                sendCallSignal(targetStr, 'screen-share-start', '');

                // Auto-revert when user stops sharing via browser UI
                screenTrack.onended = function() {
                    addDebugLog('CALL screen track ended by browser');
                    stopScreenShare();
                };
            } catch (e) {
                addDebugLog('CALL getDisplayMedia failed: ' + e.message);
                screenStream = null;
            }
        }

        async function stopScreenShare() {
            if (!screenSharing) return;

            // Revert to camera track
            if (localStream && peerConnection) {
                var cameraTrack = localStream.getVideoTracks()[0];
                if (cameraTrack) {
                    var videoSender = peerConnection.getSenders().find(function(s) {
                        return s.track && s.track.kind === 'video';
                    });
                    if (videoSender) {
                        await videoSender.replaceTrack(cameraTrack);
                    }
                }
            }

            if (screenStream) {
                screenStream.getTracks().forEach(function(t) { t.stop(); });
                screenStream = null;
            }

            screenSharing = false;
            document.getElementById('call-toggle-screen').classList.remove('screen-active');
            addDebugLog('CALL screen sharing stopped');

            // Notify remote side
            if (callTarget && callState === 'connected') {
                var targetStr = callTargetString(callTarget);
                sendCallSignal(targetStr, 'screen-share-stop', '');
            }
        }

        function setScreenZoom(level) {
            screenZoomLevel = Math.max(0.5, Math.min(3, level));
            var remoteVid = document.getElementById('remote-video');
            if (screenZoomLevel === 1) {
                remoteVid.style.transform = '';
            } else {
                remoteVid.style.transform = 'scale(' + screenZoomLevel + ')';
            }
            document.getElementById('zoom-level').textContent = Math.round(screenZoomLevel * 100) + '%';
        }

        function enableRemoteScreenShare() {
            remoteScreenShare = true;
            document.getElementById('call-videos').classList.add('screen-share-mode');
            document.getElementById('screen-share-controls').classList.add('active');
            setScreenZoom(1);
            addDebugLog('CALL remote screen share enabled');
        }

        function disableRemoteScreenShare() {
            remoteScreenShare = false;
            document.getElementById('call-videos').classList.remove('screen-share-mode');
            document.getElementById('screen-share-controls').classList.remove('active');
            setScreenZoom(1);
            addDebugLog('CALL remote screen share disabled');
        }

        // ===== Group Call Functions =====

        async function joinGroupCall(channelId, channelName) {
            if (groupCallActive) return;
            if (callState !== 'idle') {
                alert('End your current call first');
                return;
            }

            addDebugLog('GROUP joining call in #' + channelName);
            groupCallLocalStream = await getCallMedia();
            if (!groupCallLocalStream) return;

            groupCallActive = true;
            groupCallChannelId = channelId;
            groupCallChannelName = channelName;
            groupCallPeers = {};
            groupCallVideoMuted = false;
            groupCallAudioMuted = false;
            groupCallScreenSharing = false;

            document.getElementById('group-call-channel-name').textContent = '# ' + channelName;
            document.getElementById('group-call-overlay').classList.add('active');

            // Add local tile
            var localTile = createGroupPeerTile('local', 'You');
            localTile.classList.add('local');
            localTile.querySelector('video').srcObject = groupCallLocalStream;
            document.getElementById('group-call-grid').appendChild(localTile);

            // POST join
            var resp = await requestJson('/api/channels/' + channelId + '/call/join', 'POST');
            if (!resp || !resp.participants) {
                addDebugLog('GROUP join failed');
                leaveGroupCall();
                return;
            }

            // Find our own entry to get the local server name
            var selfEntry = resp.participants.find(function(p) { return p.user_id === currentUser.id; });
            if (selfEntry) groupCallLocalServerName = selfEntry.server_name;

            addDebugLog('GROUP joined, ' + resp.participants.length + ' participants, server=' + groupCallLocalServerName);
            updateGroupCallCount(resp.participants.length);

            // Connect to all existing participants (we send offers to them)
            for (var i = 0; i < resp.participants.length; i++) {
                var p = resp.participants[i];
                // Skip self
                if (p.user_id === currentUser.id) continue;
                var peerKey = p.username + '@' + p.server_name;
                addDebugLog('GROUP sending offer to ' + peerKey);
                await connectToGroupPeer(p, true);
            }
            updateGroupCallGrid();
            updateChannelCallButton();
        }

        async function leaveGroupCall() {
            if (!groupCallActive) return;
            addDebugLog('GROUP leaving call');

            // POST leave
            await requestJson('/api/channels/' + groupCallChannelId + '/call/leave', 'POST');

            // Close all peer connections
            Object.keys(groupCallPeers).forEach(function(key) {
                removeGroupPeer(key);
            });

            if (groupCallLocalStream) {
                groupCallLocalStream.getTracks().forEach(function(t) { t.stop(); });
                groupCallLocalStream = null;
            }
            if (groupCallScreenStream) {
                groupCallScreenStream.getTracks().forEach(function(t) { t.stop(); });
                groupCallScreenStream = null;
            }

            groupCallActive = false;
            groupCallChannelId = null;
            groupCallChannelName = null;
            groupCallPeers = {};
            groupCallScreenSharing = false;
            groupCallVideoMuted = false;
            groupCallAudioMuted = false;

            document.getElementById('group-call-grid').innerHTML = '';
            document.getElementById('group-call-overlay').classList.remove('active');
            document.getElementById('group-toggle-video').classList.remove('muted');
            document.getElementById('group-toggle-audio').classList.remove('muted');
            document.getElementById('group-toggle-screen').classList.remove('screen-active');
            updateChannelCallButton();
        }

        async function connectToGroupPeer(participant, sendOffer) {
            var peerKey = participant.username + '@' + participant.server_name;
            if (groupCallPeers[peerKey]) return; // already connected

            var pc = new RTCPeerConnection(STUN_CONFIG);
            var tile = createGroupPeerTile(peerKey, participant.username);
            var videoEl = tile.querySelector('video');
            document.getElementById('group-call-grid').appendChild(tile);

            groupCallPeers[peerKey] = {
                pc: pc,
                videoEl: videoEl,
                tile: tile,
                iceCandidateQueue: [],
                participant: participant
            };

            // Add local tracks
            if (groupCallLocalStream) {
                groupCallLocalStream.getTracks().forEach(function(track) {
                    pc.addTrack(track, groupCallLocalStream);
                });
            }

            pc.onicecandidate = function(event) {
                if (event.candidate) {
                    var targetStr = peerKey;
                    var payload = JSON.stringify({
                        candidate: event.candidate,
                        channel_id: groupCallChannelId,
                        from_peer: currentUser.username + '@' + groupCallLocalServerName
                    });
                    sendCallSignal(targetStr, 'group-ice-candidate', payload);
                }
            };

            pc.oniceconnectionstatechange = function() {
                addDebugLog('GROUP ICE ' + peerKey + ': ' + pc.iceConnectionState);
                if (pc.iceConnectionState === 'disconnected' || pc.iceConnectionState === 'failed') {
                    removeGroupPeer(peerKey);
                    updateGroupCallGrid();
                }
            };

            pc.ontrack = function(event) {
                addDebugLog('GROUP track from ' + peerKey + ': ' + event.track.kind);
                if (videoEl.srcObject !== event.streams[0]) {
                    videoEl.srcObject = event.streams[0];
                }
            };

            if (sendOffer) {
                try {
                    var offer = await pc.createOffer();
                    await pc.setLocalDescription(offer);
                    var payload = JSON.stringify({
                        sdp: offer,
                        channel_id: groupCallChannelId
                    });
                    sendCallSignal(peerKey, 'group-offer', payload);
                    addDebugLog('GROUP offer sent to ' + peerKey);
                } catch (e) {
                    addDebugLog('GROUP createOffer error for ' + peerKey + ': ' + e.message);
                }
            }

            updateGroupCallGrid();
            updateGroupCallCount(Object.keys(groupCallPeers).length + 1);
        }

        function removeGroupPeer(peerKey) {
            var peer = groupCallPeers[peerKey];
            if (!peer) return;
            addDebugLog('GROUP removing peer ' + peerKey);
            if (peer.pc) peer.pc.close();
            if (peer.tile && peer.tile.parentNode) {
                peer.tile.parentNode.removeChild(peer.tile);
            }
            delete groupCallPeers[peerKey];
            updateGroupCallGrid();
            updateGroupCallCount(Object.keys(groupCallPeers).length + 1);
        }

        async function handleGroupOffer(signal) {
            if (!groupCallActive) {
                addDebugLog('GROUP offer received but not in group call, ignoring');
                return;
            }
            var payloadData = JSON.parse(signal.payload);
            var fromKey = signal.from_user.username + '@' + signal.from_user.server;
            addDebugLog('GROUP offer from ' + fromKey);

            // Create peer if needed
            if (!groupCallPeers[fromKey]) {
                await connectToGroupPeer({
                    username: signal.from_user.username,
                    server_name: signal.from_user.server,
                    user_id: ''
                }, false);
            }

            var peer = groupCallPeers[fromKey];
            if (!peer) return;

            try {
                await peer.pc.setRemoteDescription(new RTCSessionDescription(payloadData.sdp));

                // Drain queued ICE candidates
                for (var i = 0; i < peer.iceCandidateQueue.length; i++) {
                    await peer.pc.addIceCandidate(new RTCIceCandidate(peer.iceCandidateQueue[i]));
                }
                peer.iceCandidateQueue = [];

                var answer = await peer.pc.createAnswer();
                await peer.pc.setLocalDescription(answer);
                var payload = JSON.stringify({
                    sdp: answer,
                    channel_id: groupCallChannelId
                });
                sendCallSignal(fromKey, 'group-answer', payload);
                addDebugLog('GROUP answer sent to ' + fromKey);
            } catch (e) {
                addDebugLog('GROUP handleGroupOffer error: ' + e.message);
            }
        }

        async function handleGroupAnswer(signal) {
            var payloadData = JSON.parse(signal.payload);
            var fromKey = signal.from_user.username + '@' + signal.from_user.server;
            var peer = groupCallPeers[fromKey];
            if (!peer) {
                addDebugLog('GROUP answer from unknown peer ' + fromKey);
                return;
            }

            try {
                await peer.pc.setRemoteDescription(new RTCSessionDescription(payloadData.sdp));
                for (var i = 0; i < peer.iceCandidateQueue.length; i++) {
                    await peer.pc.addIceCandidate(new RTCIceCandidate(peer.iceCandidateQueue[i]));
                }
                peer.iceCandidateQueue = [];
                addDebugLog('GROUP answer from ' + fromKey + ' applied');
            } catch (e) {
                addDebugLog('GROUP handleGroupAnswer error: ' + e.message);
            }
        }

        async function handleGroupIceCandidate(signal) {
            var payloadData = JSON.parse(signal.payload);
            var fromKey = signal.from_user.username + '@' + signal.from_user.server;
            var peer = groupCallPeers[fromKey];
            if (!peer) {
                addDebugLog('GROUP ICE from unknown peer ' + fromKey);
                return;
            }

            if (peer.pc && peer.pc.remoteDescription) {
                try {
                    await peer.pc.addIceCandidate(new RTCIceCandidate(payloadData.candidate));
                } catch (e) {
                    addDebugLog('GROUP addIceCandidate error: ' + e.message);
                }
            } else {
                peer.iceCandidateQueue.push(payloadData.candidate);
            }
        }

        function toggleGroupVideo() {
            if (!groupCallLocalStream) return;
            groupCallLocalStream.getVideoTracks().forEach(function(t) { t.enabled = !t.enabled; });
            groupCallVideoMuted = !groupCallVideoMuted;
            document.getElementById('group-toggle-video').classList.toggle('muted', groupCallVideoMuted);
        }

        function toggleGroupAudio() {
            if (!groupCallLocalStream) return;
            groupCallLocalStream.getAudioTracks().forEach(function(t) { t.enabled = !t.enabled; });
            groupCallAudioMuted = !groupCallAudioMuted;
            document.getElementById('group-toggle-audio').classList.toggle('muted', groupCallAudioMuted);
        }

        async function toggleGroupScreenShare() {
            if (!groupCallActive) return;

            if (groupCallScreenSharing) {
                // Revert to camera
                if (groupCallLocalStream) {
                    var cameraTrack = groupCallLocalStream.getVideoTracks()[0];
                    if (cameraTrack) {
                        Object.values(groupCallPeers).forEach(function(peer) {
                            var sender = peer.pc.getSenders().find(function(s) {
                                return s.track && s.track.kind === 'video';
                            });
                            if (sender) sender.replaceTrack(cameraTrack);
                        });
                    }
                }
                if (groupCallScreenStream) {
                    groupCallScreenStream.getTracks().forEach(function(t) { t.stop(); });
                    groupCallScreenStream = null;
                }
                groupCallScreenSharing = false;
                document.getElementById('group-toggle-screen').classList.remove('screen-active');
                return;
            }

            try {
                groupCallScreenStream = await navigator.mediaDevices.getDisplayMedia({ video: true });
                var screenTrack = groupCallScreenStream.getVideoTracks()[0];

                Object.values(groupCallPeers).forEach(function(peer) {
                    var sender = peer.pc.getSenders().find(function(s) {
                        return s.track && s.track.kind === 'video';
                    });
                    if (sender) sender.replaceTrack(screenTrack);
                });

                groupCallScreenSharing = true;
                document.getElementById('group-toggle-screen').classList.add('screen-active');

                screenTrack.onended = function() {
                    toggleGroupScreenShare(); // will revert
                };
            } catch (e) {
                addDebugLog('GROUP getDisplayMedia failed: ' + e.message);
            }
        }

        function updateGroupCallGrid() {
            var grid = document.getElementById('group-call-grid');
            var count = grid.children.length;
            grid.classList.remove('cols-2', 'cols-3', 'cols-4');
            if (count === 1) {
                // single participant, 1 column
            } else if (count <= 4) {
                grid.classList.add('cols-2');  // 2x2 square grid
            } else if (count <= 9) {
                grid.classList.add('cols-3');  // 3x3 square grid
            } else {
                grid.classList.add('cols-4');  // 4x4 square grid
            }
        }

        function updateGroupCallCount(count) {
            document.getElementById('group-call-count').textContent = count + ' participant' + (count !== 1 ? 's' : '');
        }

        function createGroupPeerTile(peerKey, label) {
            var tile = document.createElement('div');
            tile.className = 'group-call-tile';
            tile.setAttribute('data-peer', peerKey);
            var video = document.createElement('video');
            video.autoplay = true;
            video.playsInline = true;
            if (peerKey === 'local') video.muted = true;
            tile.appendChild(video);
            var labelEl = document.createElement('div');
            labelEl.className = 'tile-label';
            labelEl.textContent = label;
            tile.appendChild(labelEl);
            return tile;
        }

        function updateChannelCallButton() {
            var btn = document.getElementById('channel-call-btn');
            if (!btn) return;
            if (!currentTarget || currentTargetType !== 'channel') return;

            if (groupCallActive && groupCallChannelId === currentTarget) {
                btn.textContent = 'Leave Call';
                btn.className = 'channel-call-btn in-call';
            } else {
                var count = activeChannelCalls[currentTarget] || 0;
                if (count > 0) {
                    btn.textContent = 'Join Call (' + count + ')';
                } else {
                    btn.textContent = 'Start Call';
                }
                btn.className = 'channel-call-btn';
            }
        }

        async function refreshActiveChannelCalls() {
            var data = await requestJson('/api/channels/active-calls');
            if (data) {
                activeChannelCalls = data;
                updateChannelCallButton();
                loadChannels();
            }
        }

        function addDebugLog() {}

        function escapeHtml(text) {
            const textarea = document.createElement('textarea');
            textarea.textContent = text;
            return textarea.innerHTML;
        }

        function getUserToken() {
          const sessionToken = sessionStorage.getItem('user_token');
          if (sessionToken) return sessionToken;
          const legacyToken = localStorage.getItem('user_token');
          if (legacyToken) {
            sessionStorage.setItem('user_token', legacyToken);
            localStorage.removeItem('user_token');
          }
          return legacyToken;
        }

        function setCurrentUserFromToken(users) {
          const token = getUserToken();
          if (!token) return false;
          // Try stored user info first (session tokens won't match DB tokens)
          const storedInfo = sessionStorage.getItem('user_info');
          if (storedInfo) {
            try {
              const info = JSON.parse(storedInfo);
              if (info.id && info.username) {
                currentUser = { id: info.id, username: info.username, display_name: info.display_name };
                document.getElementById('current-user').textContent = info.display_name || info.username;
                return true;
              }
            } catch(e) {}
          }
          // Fallback: match by DB token (backwards compat)
          if (!users) return false;
          const match = users.find(function(u) {
            return u.token === token;
          });
          if (!match) return false;
          currentUser = { id: match.id, username: match.username, display_name: match.display_name };
          document.getElementById('current-user').textContent = match.display_name || match.username;
          return true;
        }

        async function requestJson(url, method, body) {
            const opts = {
                method: method || 'GET',
                headers: { 
                    'x-admin-token': getUserToken()
                }
            };
            if (body) {
                opts.body = JSON.stringify(body);
                opts.headers['Content-Type'] = 'application/json';
            }
            
            addDebugLog(`→ ${opts.method} ${url}`);
            if (body) addDebugLog(`  Body: ${JSON.stringify(body)}`);
            
            const resp = await fetch(url, opts);
            let data = null;
            try {
                data = await resp.json();
            } catch (e) {
                data = `[JSON parse error: ${e.message}]`;
            }
            
            addDebugLog(`← Status: ${resp.status} from ${url}`);
            if (data) {
                const dataStr = typeof data === 'string' ? data : JSON.stringify(data).substring(0, 100);
                addDebugLog(`  Response: ${dataStr}`);
            }
            
            if (resp.status === 401) {
                addDebugLog(`⚠ 401 Unauthorized - logging out`);
                logout();
                return null;
            }
            return data;
        }

        async function loadUsers(seedUsers) {
          const users = seedUsers || await requestJson('/api/users');
          if (!users) return;
          allUsers = users;

          if (!currentUser) {
            setCurrentUserFromToken(allUsers);
          }

            // Debug: show received user list with online status
            if (allUsers.length > 0) {
                const onlineInfo = allUsers.map(u => 
                    (`${u.is_online ? '🟢' : '⚪'} ${u.username}${u.server_name ? '@' + u.server_name : ''}`
                )).join(', ');
                addDebugLog(`📋 User list received (${allUsers.length} users): ${onlineInfo}`);
            }
            
            const list = document.getElementById('users-list');
            list.innerHTML = '';
            
            // Filter out duplicates and current user
            const seen = new Set();
            const filtered = [];
            allUsers.forEach(function(u) {
              if ((!currentUser || u.id !== currentUser.id) && !seen.has(u.id)) {
                    seen.add(u.id);
                    filtered.push(u);
                }
            });
            
            filtered.forEach(function(u) {
                const div = document.createElement('div');
                div.className = 'sidebar-item';
                if (currentTarget === u.id && currentTargetType === 'user') {
                    div.classList.add('active');
                }
                // Add 'unread' class if this user has unread messages
                if (unreadDMs.has(u.id)) {
                    div.classList.add('unread');
                }
              const serverSuffix = u.server_name ? ('@' + u.server_name) : '';
              const onlineDot = u.is_online ? '🟢' : '⚪';
              const displayName = u.display_name || (u.username + serverSuffix);

              const content = document.createElement('span');
              content.className = 'sidebar-item-content';

              const nameSpan = document.createElement('span');
              nameSpan.className = 'sidebar-item-name';
              nameSpan.textContent = onlineDot + ' 👤 ' + displayName;
              content.appendChild(nameSpan);

              const camBtn = document.createElement('button');
              camBtn.className = 'video-call-btn' + (u.is_online ? ' online' : '');
              camBtn.textContent = '📹';
              camBtn.title = 'Video call ' + (u.display_name || u.username);
              camBtn.onclick = function(e) {
                  e.stopPropagation();
                  startCall(u);
              };
              content.appendChild(camBtn);

              div.appendChild(content);
              div.onclick = function() { selectUser(u); };
              list.appendChild(div);
            });
        }

        async function loadChannels() {
            const channels = await requestJson('/api/channels');
            const list = document.getElementById('channels-list');
            list.innerHTML = '';
            if (!channels) return;
            
            // Filter out duplicates
            const seen = new Set();
            const filtered = [];
            channels.forEach(function(c) {
                if (!seen.has(c.id)) {
                    seen.add(c.id);
                    filtered.push(c);
                }
            });
            
            filtered.forEach(function(c) {
                const div = document.createElement('div');
                div.className = 'sidebar-item';
                if (currentTarget === c.id && currentTargetType === 'channel') {
                    div.classList.add('active');
                }
                div.textContent = '# ' + c.name;
                var callCount = activeChannelCalls[c.id];
                if (callCount && callCount > 0) {
                    var badge = document.createElement('span');
                    badge.className = 'call-badge';
                    badge.textContent = callCount + ' in call';
                    div.appendChild(badge);
                }
                div.onclick = function() { selectChannel(c); };
                list.appendChild(div);
            });
        }

        function selectUser(user) {
            currentTarget = user.id;
            currentTargetType = 'user';
            document.getElementById('current-title').textContent = 'DM with ' + (user.display_name || user.username);
            document.getElementById('input-area').classList.add('active');
            // Mark DM as read when viewing it
            unreadDMs.delete(user.id);
            loadMessages('dm', user.id);
            loadUsers();
            loadChannels();
        }

        function selectChannel(channel) {
            currentTarget = channel.id;
            currentTargetType = 'channel';
            var titleEl = document.getElementById('current-title');
            titleEl.textContent = '# ' + channel.name;

            // Add call button
            var existingBtn = document.getElementById('channel-call-btn');
            if (existingBtn) existingBtn.remove();
            var callBtn = document.createElement('button');
            callBtn.id = 'channel-call-btn';
            callBtn.className = 'channel-call-btn';
            callBtn.textContent = 'Start Call';
            callBtn.onclick = function() {
                if (groupCallActive && groupCallChannelId === channel.id) {
                    leaveGroupCall();
                } else {
                    joinGroupCall(channel.id, channel.name);
                }
            };
            titleEl.appendChild(callBtn);
            updateChannelCallButton();

            document.getElementById('input-area').classList.add('active');
            loadMessages('channel', channel.id);
            loadUsers();
            loadChannels();
        }

        function startEventSource() {
            if (eventSource) {
                eventSource.close();
            }
            
            const token = getUserToken();
            addDebugLog('🔌 SSE connecting...');
            eventSource = new EventSource('/api/events?token=' + encodeURIComponent(token));

            eventSource.onopen = function() {
                addDebugLog('✅ SSE connection established');
                loadUsers();
            };
            
            eventSource.onmessage = function(event) {
                try {
                    const notification = JSON.parse(event.data);
                addDebugLog(`📨 SSE: ${notification.event} received (user_id: ${notification.user_id || 'none'}, channel_id: ${notification.channel_id || 'none'})`);
                    
                    // Handle WebRTC signals
                    if (notification.event === 'webrtc_signal') {
                        if (currentUser && notification.target_user_id === currentUser.id) {
                            try {
                                var signal = JSON.parse(notification.payload);
                                handleWebRtcSignal(signal);
                            } catch (parseErr) {
                                addDebugLog('CALL SSE signal parse error: ' + parseErr.message);
                            }
                        }
                        return;
                    }

                    // Handle channel call events
                    if (notification.event === 'channel_call_join' || notification.event === 'channel_call_leave') {
                        addDebugLog('GROUP SSE: ' + notification.event + ' ch=' + notification.channel_id);
                        refreshActiveChannelCalls();
                        // If someone left and we're in that call, remove their peer
                        if (notification.event === 'channel_call_leave' && groupCallActive && notification.channel_id === groupCallChannelId) {
                            try {
                                var evtData = JSON.parse(notification.payload);
                                var peerKey = evtData.username + '@' + evtData.server_name;
                                removeGroupPeer(peerKey);
                            } catch (e) {}
                        }
                        return;
                    }

                    // Handle presence changes - reload user list
                    if (notification.event === 'presence_changed') {
                        addDebugLog('👥 SSE -> presence changed, refreshing user list');
                        loadUsers();
                        return;
                    }
                    
                    // If viewing a DM and got notification for this user
                    if (currentTargetType === 'user' && notification.user_id === currentTarget) {
                        addDebugLog('🔄 SSE -> refreshing current DM view');
                        loadMessages('dm', currentTarget, true);
                    }
                    // If viewing a channel and got notification for this channel
                    else if (currentTargetType === 'channel' && notification.channel_id === currentTarget) {
                        addDebugLog('🔄 SSE -> refreshing current channel view');
                        loadMessages('channel', currentTarget, true);
                    }
                    // If logged in as this user (received a new message)
                    else if (currentUser && notification.user_id === currentUser.id) {
                        addDebugLog('🔄 SSE -> refreshing active view for current user');
                        loadMessages(currentTargetType === 'user' ? 'dm' : 'channel', currentTarget, true);
                    }
                    
                    // Mark message as unread if it's for a user we're not currently viewing
                    if (notification.event === 'new_message' && notification.user_id) {
                        if (currentTargetType !== 'user' || currentTarget !== notification.user_id) {
                            addDebugLog(`🔔 Unread DM from user: ${notification.user_id}`);
                            unreadDMs.add(notification.user_id);
                            loadUsers(); // Refresh to show orange background
                        }
                    }
                } catch (e) {
                    addDebugLog(`⚠ SSE parse error: ${e.message}`);
                }
            };
            
            eventSource.onerror = function(event) {
                    const state = eventSource ? eventSource.readyState : 'unknown';
                    addDebugLog(`⚠ SSE connection error (readyState: ${state}), will reconnect...`);
                // Browser will automatically reconnect
            };
        }

        async function loadMessages(type, id, silent) {
            let messages = [];
            if (!silent) addDebugLog(`Loading ${type} messages for ID: ${id}`);
            if (type === 'dm') {
                messages = await requestJson('/api/messages/dm/' + id) || [];
            } else if (type === 'channel') {
                messages = await requestJson('/api/messages/channel/' + id) || [];
            }
            if (!silent) addDebugLog(`Got ${messages.length} messages`);
            const container = document.getElementById('messages');
            container.innerHTML = '';
            messages.forEach(function(m) {
                const div = document.createElement('div');
                div.className = 'message';
                if (m.author_user_id === currentUser.id) {
                    div.classList.add('own');
                }
                const content = document.createElement('div');
                content.className = 'message-content';
                const author = document.createElement('div');
                author.className = 'message-author';
                author.textContent = m.author_display_name || m.author_username;
                const text = document.createElement('div');
                text.className = 'message-text';
                var gifMatch = m.body.match(/^\[gif:(https?:\/\/[^\]]+)\]$/);
                if (gifMatch) {
                    var img = document.createElement('img');
                    img.src = gifMatch[1];
                    img.alt = 'GIF';
                    img.className = 'gif-msg-img';
                    text.appendChild(img);
                } else {
                    text.textContent = m.body;
                }
                const time = document.createElement('div');
                time.className = 'message-time';
                const d = new Date(m.sent_at);
                time.textContent = d.toLocaleTimeString();
                content.appendChild(author);
                content.appendChild(text);
                content.appendChild(time);
                div.appendChild(content);
                container.appendChild(div);
            });
            container.scrollTop = container.scrollHeight;
        }

        async function sendMessage() {
            const input = document.getElementById('msg-input');
            const body = input.value.trim();
            if (!body || !currentTarget) return;
            // Intercept /gif command
            if (body === '/gif' || body.startsWith('/gif ')) {
                var gifQuery = body.slice(4).trim();
                input.value = '';
                openGifPicker(gifQuery);
                return;
            }
            addDebugLog(`Sending message: "${body}" to target: ${currentTarget} (${currentTargetType})`);
            input.value = '';
            if (currentTargetType === 'user') {
                const user = allUsers.find(function(u) {
                    return u.id === currentTarget;
                });
                if (user) {
                    let recipient = user.username;
                    // If user is remote, add server name from API response
                    if (user.server_name) {
                        recipient = user.username + '@' + user.server_name;
                        addDebugLog(`  Remote user detected: ${user.username} (server: ${user.server_name}) → sending as ${recipient}`);
                    } else {
                        addDebugLog(`  Local user: ${user.username}`);
                    }
                    addDebugLog(`  POSTing to /api/messages/dm with recipient: ${recipient}`);
                    const result = await requestJson('/api/messages/dm', 'POST', {
                        recipient: recipient,
                        body: body
                    });
                    addDebugLog(`  Send result: ${result ? 'success' : 'null/error'}`);
                    loadMessages('dm', currentTarget);
                } else {
                    addDebugLog(`⚠ User not found in allUsers for ID: ${currentTarget}`);
                }
            } else if (currentTargetType === 'channel') {
                const channel = (await requestJson('/api/channels') || []).find(function(c) {
                    return c.id === currentTarget;
                });
                if (channel) {
                    addDebugLog(`  POSTing to /api/messages/channel with channel: ${channel.name}`);
                    const result = await requestJson('/api/messages/channel', 'POST', {
                        channel: channel.name,
                        origin_server: channel.origin_server,
                        body: body
                    });
                    addDebugLog(`  Send result: ${result ? 'success' : 'null/error'}`);
                    loadMessages('channel', currentTarget);
                } else {
                    addDebugLog(`⚠ Channel not found in channels list for ID: ${currentTarget}`);
                }
            }
        }

        function showNewChannelModal() {
            document.getElementById('new-channel-modal').classList.add('active');
        }

        function closeNewChannelModal() {
            document.getElementById('new-channel-modal').classList.remove('active');
            document.getElementById('channel-name-input').value = '';
        }

        async function createChannel() {
            const name = document.getElementById('channel-name-input').value.trim();
            if (!name) return;
            await requestJson('/api/channels', 'POST', { name: name });
            closeNewChannelModal();
            loadChannels();
        }
        function logout() {
            if (eventSource) {
                eventSource.close();
                eventSource = null;
            }
            sessionStorage.removeItem('user_token');
            sessionStorage.removeItem('user_info');
            localStorage.removeItem('user_token');
          currentUser = null;
          currentTarget = null;
          currentTargetType = null;
          unreadDMs.clear();
            document.getElementById('app').classList.remove('active');
            document.getElementById('login-screen').style.display = 'flex';
            document.getElementById('login-username').value = '';
            document.getElementById('login-password').value = '';
            document.getElementById('login-username').focus();
        }

        async function handleLogin() {
            const username = document.getElementById('login-username').value.trim();
            const password = document.getElementById('login-password').value;
            const errorEl = document.getElementById('login-error');
            errorEl.textContent = '';
            if (!username) { errorEl.textContent = 'Enter a username'; return; }

            const resp = await fetch('/api/login', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ username: username, password: password })
            });

            if (resp.ok) {
                const data = await resp.json();
              sessionStorage.setItem('user_token', data.token);
              sessionStorage.setItem('user_info', JSON.stringify({ id: data.user_id, username: data.username, display_name: data.display_name }));
                currentUser = { id: data.user_id, username: data.username, display_name: data.display_name };
                document.getElementById('current-user').textContent = data.display_name || data.username;
                document.getElementById('login-screen').style.display = 'none';
                document.getElementById('app').classList.add('active');
                loadUsers();
                refreshActiveChannelCalls().then(function() { loadChannels(); });
                startEventSource();
            } else {
                errorEl.textContent = 'Login failed. Check username/password.';
            }
        }

        async function initChat() {
            const token = getUserToken();
            if (!token) {
                document.getElementById('login-screen').style.display = 'flex';
                document.getElementById('app').classList.remove('active');
                return;
            }
            document.getElementById('app').classList.add('active');
            document.getElementById('login-screen').style.display = 'none';

          const users = await requestJson('/api/users');
          if (!users) return;
          if (!setCurrentUserFromToken(users)) {
            addDebugLog('⚠ Stored token does not match a user. Logging out.');
            logout();
            return;
          }

          loadUsers(users);
            refreshActiveChannelCalls().then(function() { loadChannels(); });
            startEventSource();
            setInterval(refreshActiveChannelCalls, 10000);
        }

        document.getElementById('login-btn').addEventListener('click', handleLogin);
        document.getElementById('login-username').addEventListener('keypress', function(e) {
            if (e.key === 'Enter') document.getElementById('login-password').focus();
        });
        document.getElementById('login-password').addEventListener('keypress', function(e) {
            if (e.key === 'Enter') handleLogin();
        });
        document.getElementById('new-channel-btn').addEventListener('click', showNewChannelModal);
        document.getElementById('modal-cancel').addEventListener('click', closeNewChannelModal);
        document.getElementById('modal-create').addEventListener('click', createChannel);
        document.getElementById('logout-btn').addEventListener('click', logout);

        // User dropdown menu toggle
        document.getElementById('user-menu-toggle').addEventListener('click', function(e) {
            e.stopPropagation();
            document.getElementById('user-menu-dropdown').classList.toggle('open');
        });
        // Close dropdown when clicking outside
        document.addEventListener('click', function(e) {
            const dropdown = document.getElementById('user-menu-dropdown');
            if (dropdown && !dropdown.contains(e.target)) {
                dropdown.classList.remove('open');
            }
        });

        document.getElementById('send-btn').addEventListener('click', sendMessage);
        document.getElementById('msg-input').addEventListener('keypress', function(e) {
            if (e.key === 'Enter') sendMessage();
        });

        // Wire call overlay buttons
        document.getElementById('call-hangup').addEventListener('click', function() { endCall(true); });
        document.getElementById('call-toggle-video').addEventListener('click', toggleVideo);
        document.getElementById('call-toggle-audio').addEventListener('click', toggleAudio);
        document.getElementById('call-accept').addEventListener('click', acceptCall);
        document.getElementById('call-reject').addEventListener('click', rejectCall);
        document.getElementById('call-toggle-screen').addEventListener('click', toggleScreenShare);
        document.getElementById('zoom-in-btn').addEventListener('click', function() {
            setScreenZoom(screenZoomLevel + 0.25);
        });
        document.getElementById('zoom-out-btn').addEventListener('click', function() {
            setScreenZoom(screenZoomLevel - 0.25);
        });
        document.getElementById('zoom-fit-btn').addEventListener('click', function() {
            setScreenZoom(1);
        });
        document.getElementById('call-videos').addEventListener('wheel', function(e) {
            if (!remoteScreenShare) return;
            e.preventDefault();
            var delta = e.deltaY > 0 ? -0.25 : 0.25;
            setScreenZoom(screenZoomLevel + delta);
        }, { passive: false });

        // Wire group call buttons
        document.getElementById('group-call-leave').addEventListener('click', leaveGroupCall);
        document.getElementById('group-toggle-video').addEventListener('click', toggleGroupVideo);
        document.getElementById('group-toggle-audio').addEventListener('click', toggleGroupAudio);
        document.getElementById('group-toggle-screen').addEventListener('click', toggleGroupScreenShare);

        // --- GIF Picker ---
        var gifSearchTimeout = null;

        function openGifPicker(query) {
            document.getElementById('gif-picker').classList.add('active');
            var searchInput = document.getElementById('gif-search-input');
            searchInput.value = query || '';
            searchInput.focus();
            if (query) {
                searchGifs(query);
            } else {
                document.getElementById('gif-grid').innerHTML = '<div class="gif-grid-empty">Type to search for GIFs</div>';
            }
        }

        function closeGifPicker() {
            document.getElementById('gif-picker').classList.remove('active');
            document.getElementById('gif-search-input').value = '';
            document.getElementById('gif-grid').innerHTML = '<div class="gif-grid-empty">Type to search for GIFs</div>';
        }

        async function searchGifs(query) {
            if (!query.trim()) {
                document.getElementById('gif-grid').innerHTML = '<div class="gif-grid-empty">Type to search for GIFs</div>';
                return;
            }
            var grid = document.getElementById('gif-grid');
            grid.innerHTML = '<div class="gif-grid-empty">Searching...</div>';
            try {
                var resp = await fetch('/api/gif/search?q=' + encodeURIComponent(query.trim()) + '&limit=20', {
                    headers: { 'x-admin-token': getUserToken() }
                });
                if (!resp.ok) {
                    var errText = await resp.text();
                    grid.innerHTML = '<div class="gif-grid-empty">' + (errText || 'Search failed') + '</div>';
                    return;
                }
                var gifs = await resp.json();
                if (!gifs.length) {
                    grid.innerHTML = '<div class="gif-grid-empty">No GIFs found</div>';
                    return;
                }
                grid.innerHTML = '';
                gifs.forEach(function(g) {
                    var img = document.createElement('img');
                    img.src = g.preview_url;
                    img.alt = 'GIF';
                    img.loading = 'lazy';
                    img.addEventListener('click', function() {
                        selectGif(g.url);
                    });
                    grid.appendChild(img);
                });
            } catch (e) {
                grid.innerHTML = '<div class="gif-grid-empty">Error: ' + e.message + '</div>';
            }
        }

        async function selectGif(url) {
            closeGifPicker();
            var body = '[gif:' + url + ']';
            addDebugLog('Sending GIF: ' + url);
            if (currentTargetType === 'user') {
                var user = allUsers.find(function(u) { return u.id === currentTarget; });
                if (user) {
                    var recipient = user.username;
                    if (user.server_name) recipient = user.username + '@' + user.server_name;
                    await requestJson('/api/messages/dm', 'POST', { recipient: recipient, body: body });
                    loadMessages('dm', currentTarget);
                }
            } else if (currentTargetType === 'channel') {
                var channel = (await requestJson('/api/channels') || []).find(function(c) { return c.id === currentTarget; });
                if (channel) {
                    await requestJson('/api/messages/channel', 'POST', { channel: channel.name, origin_server: channel.origin_server, body: body });
                    loadMessages('channel', currentTarget);
                }
            }
        }

        document.getElementById('gif-search-input').addEventListener('input', function(e) {
            clearTimeout(gifSearchTimeout);
            var val = e.target.value;
            gifSearchTimeout = setTimeout(function() { searchGifs(val); }, 300);
        });

        document.getElementById('gif-search-input').addEventListener('keypress', function(e) {
            if (e.key === 'Enter') {
                clearTimeout(gifSearchTimeout);
                searchGifs(e.target.value);
            }
        });

        document.getElementById('gif-picker').addEventListener('click', function(e) {
            if (e.target === this) closeGifPicker();
        });

        initChat();
    </script>
</body>
</html>"#;

const SETTINGS_UI_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8"/>
    <meta name="viewport" content="width=device-width, initial-scale=1.0"/>
    <title>BeringShare Settings</title>
    <style>
        @import url('https://fonts.googleapis.com/css2?family=Space+Grotesk:wght@400;500;600;700&display=swap');
        :root {
            --bg: #1a1a2e;
            --bg-card: #16213e;
            --text: #e7f2f4;
            --muted: #9ab2b7;
            --accent: #49d3b0;
            --border: #20373c;
        }
        * { box-sizing: border-box; }
        body {
            margin: 0;
            font-family: 'Space Grotesk', system-ui, sans-serif;
            color: var(--text);
            background: linear-gradient(180deg, #0f0f23 0%, var(--bg) 100%);
            min-height: 100vh;
            display: flex;
            align-items: center;
            justify-content: center;
            padding: 20px;
        }
        .card {
            background: var(--bg-card);
            border: 1px solid var(--border);
            border-radius: 18px;
            padding: 40px;
            max-width: 560px;
            width: 100%;
        }
        .card h1 {
            margin: 0 0 8px;
            font-size: 28px;
        }
        .card .subtitle {
            color: var(--muted);
            margin: 0 0 32px;
            font-size: 14px;
        }
        .form-group {
            margin-bottom: 24px;
        }
        .form-group label {
            display: block;
            font-size: 13px;
            font-weight: 600;
            text-transform: uppercase;
            color: var(--muted);
            margin-bottom: 8px;
        }
        .form-group select {
            width: 100%;
            padding: 10px 12px;
            background: rgba(255,255,255,0.06);
            border: 1px solid var(--border);
            border-radius: 8px;
            color: var(--text);
            font-family: inherit;
            font-size: 14px;
            outline: none;
            transition: border-color 0.2s;
            appearance: none;
            -webkit-appearance: none;
            background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 12 12'%3E%3Cpath fill='%239ab2b7' d='M6 8L1 3h10z'/%3E%3C/svg%3E");
            background-repeat: no-repeat;
            background-position: right 12px center;
        }
        .form-group select:focus {
            border-color: var(--accent);
        }
        .video-preview {
            margin-top: 12px;
            border-radius: 12px;
            overflow: hidden;
            background: #000;
            aspect-ratio: 16/9;
            display: flex;
            align-items: center;
            justify-content: center;
        }
        .video-preview video {
            width: 100%;
            height: 100%;
            object-fit: cover;
            display: block;
            transform: scaleX(-1);
        }
        .video-preview .placeholder {
            color: var(--muted);
            font-size: 14px;
        }
        .audio-meter {
            margin-top: 12px;
            height: 12px;
            background: rgba(255,255,255,0.06);
            border-radius: 6px;
            overflow: hidden;
        }
        .audio-meter-bar {
            height: 100%;
            width: 0%;
            border-radius: 6px;
            background: var(--accent);
            transition: width 0.05s linear;
        }
        .permission-error {
            display: none;
            background: rgba(255, 82, 82, 0.15);
            border: 1px solid rgba(255, 82, 82, 0.4);
            border-radius: 10px;
            padding: 16px;
            margin-bottom: 24px;
            color: #ff8a80;
            font-size: 14px;
            line-height: 1.5;
        }
        .back-link {
            display: inline-block;
            color: var(--accent);
            text-decoration: none;
            font-size: 14px;
            font-weight: 500;
            padding: 8px 16px;
            border: 1px solid var(--border);
            border-radius: 8px;
            transition: all 0.2s;
            margin-top: 8px;
        }
        .back-link:hover {
            border-color: var(--accent);
            background: rgba(73, 211, 176, 0.08);
        }
        .tabs {
            display: flex;
            gap: 4px;
            margin-bottom: 28px;
            border-bottom: 1px solid var(--border);
            padding-bottom: 0;
        }
        .tab-btn {
            padding: 10px 20px;
            border: none;
            border-bottom: 2px solid transparent;
            background: transparent;
            color: var(--muted);
            font-family: inherit;
            font-size: 14px;
            font-weight: 600;
            cursor: pointer;
            transition: all 0.2s;
        }
        .tab-btn:hover { color: var(--text); }
        .tab-btn.active {
            color: var(--accent);
            border-bottom-color: var(--accent);
        }
        .tab-panel { display: none; }
        .tab-panel.active { display: block; }
    </style>
</head>
<body>
    <div class="card">
        <h1>Settings</h1>
        <p class="subtitle">Configure your profile and devices.</p>

        <div class="tabs">
            <button class="tab-btn active" data-tab="profile">Profile</button>
            <button class="tab-btn" data-tab="security">Security</button>
            <button class="tab-btn" data-tab="camera">Camera</button>
            <button class="tab-btn" data-tab="microphone">Microphone</button>
        </div>

        <div id="tab-profile" class="tab-panel active">
            <div class="form-group">
                <label for="display-name-input">Display Name</label>
                <input type="text" id="display-name-input" placeholder="Enter display name"
                       style="width:100%;padding:10px 12px;background:rgba(255,255,255,0.06);border:1px solid var(--border);border-radius:8px;color:var(--text);font-family:inherit;font-size:14px;outline:none;" />
            </div>
            <button id="save-display-name" style="padding:10px 24px;border-radius:8px;border:none;background:var(--accent);color:#08211c;font-weight:600;cursor:pointer;font-family:inherit;font-size:14px;">Save Display Name</button>
            <div id="profile-status" style="font-size:12px;color:var(--accent);margin-top:8px;"></div>
        </div>

        <div id="tab-security" class="tab-panel">
            <div class="form-group">
                <label for="current-password">Current Password</label>
                <input type="password" id="current-password" placeholder="Current password (leave blank if none set)"
                       style="width:100%;padding:10px 12px;background:rgba(255,255,255,0.06);border:1px solid var(--border);border-radius:8px;color:var(--text);font-family:inherit;font-size:14px;outline:none;" />
            </div>
            <div class="form-group">
                <label for="new-password">New Password</label>
                <input type="password" id="new-password" placeholder="New password"
                       style="width:100%;padding:10px 12px;background:rgba(255,255,255,0.06);border:1px solid var(--border);border-radius:8px;color:var(--text);font-family:inherit;font-size:14px;outline:none;" />
            </div>
            <div class="form-group">
                <label for="confirm-password">Confirm New Password</label>
                <input type="password" id="confirm-password" placeholder="Confirm new password"
                       style="width:100%;padding:10px 12px;background:rgba(255,255,255,0.06);border:1px solid var(--border);border-radius:8px;color:var(--text);font-family:inherit;font-size:14px;outline:none;" />
            </div>
            <button id="change-password-btn" style="padding:10px 24px;border-radius:8px;border:none;background:var(--accent);color:#08211c;font-weight:600;cursor:pointer;font-family:inherit;font-size:14px;">Change Password</button>
            <div id="password-status" style="font-size:12px;margin-top:8px;"></div>
        </div>

        <div id="tab-camera" class="tab-panel">
            <div id="permission-error" class="permission-error">
                <strong>Permission denied.</strong> Please allow camera and microphone access in your browser settings and reload this page.
            </div>
            <div class="form-group">
                <label for="video-select">Video Device</label>
                <select id="video-select"><option value="">Loading...</option></select>
            </div>
            <div class="video-preview" id="video-preview">
                <span class="placeholder">No preview</span>
            </div>
        </div>

        <div id="tab-microphone" class="tab-panel">
            <div class="form-group">
                <label for="audio-select">Audio Device</label>
                <select id="audio-select"><option value="">Loading...</option></select>
            </div>
            <div class="audio-meter">
                <div class="audio-meter-bar" id="audio-meter-bar"></div>
            </div>
        </div>

        <a href="/chat/ui" class="back-link" id="back-link">&larr; Back to Chat</a>
    </div>

    <script>
        const LS_VIDEO_KEY = 'beringshare_videoDeviceId';
        const LS_AUDIO_KEY = 'beringshare_audioDeviceId';

        const videoSelect = document.getElementById('video-select');
        const audioSelect = document.getElementById('audio-select');
        const videoPreview = document.getElementById('video-preview');
        const meterBar = document.getElementById('audio-meter-bar');
        const permError = document.getElementById('permission-error');

        let currentStream = null;
        let audioCtx = null;
        let analyser = null;
        let animFrameId = null;
        let mediaInitialized = false;

        async function initMedia() {
            if (mediaInitialized) {
                await startPreview();
                return;
            }
            try {
                // Step 1: unlock device labels
                const tempStream = await navigator.mediaDevices.getUserMedia({ video: true, audio: true });
                tempStream.getTracks().forEach(function(t) { t.stop(); });
            } catch (err) {
                permError.style.display = 'block';
                return;
            }

            mediaInitialized = true;

            // Step 2: enumerate and populate
            await enumerateDevices();

            // Step 3: start preview
            await startPreview();

            // Listen for device changes (USB plug/unplug)
            navigator.mediaDevices.addEventListener('devicechange', enumerateDevices);
        }

        async function enumerateDevices() {
            const devices = await navigator.mediaDevices.enumerateDevices();
            const savedVideo = localStorage.getItem(LS_VIDEO_KEY);
            const savedAudio = localStorage.getItem(LS_AUDIO_KEY);

            // Populate video dropdown
            videoSelect.innerHTML = '';
            const videoDev = devices.filter(function(d) { return d.kind === 'videoinput'; });
            videoDev.forEach(function(d, i) {
                const opt = document.createElement('option');
                opt.value = d.deviceId;
                opt.textContent = d.label || ('Camera ' + (i + 1));
                if (d.deviceId === savedVideo) opt.selected = true;
                videoSelect.appendChild(opt);
            });
            if (videoDev.length === 0) {
                const opt = document.createElement('option');
                opt.value = '';
                opt.textContent = 'No cameras found';
                videoSelect.appendChild(opt);
            }

            // Populate audio dropdown
            audioSelect.innerHTML = '';
            const audioDev = devices.filter(function(d) { return d.kind === 'audioinput'; });
            audioDev.forEach(function(d, i) {
                const opt = document.createElement('option');
                opt.value = d.deviceId;
                opt.textContent = d.label || ('Microphone ' + (i + 1));
                if (d.deviceId === savedAudio) opt.selected = true;
                audioSelect.appendChild(opt);
            });
            if (audioDev.length === 0) {
                const opt = document.createElement('option');
                opt.value = '';
                opt.textContent = 'No microphones found';
                audioSelect.appendChild(opt);
            }
        }

        async function startPreview() {
            const videoId = videoSelect.value;
            const audioId = audioSelect.value;
            if (!videoId && !audioId) return;

            const constraints = {};
            if (videoId) constraints.video = { deviceId: { exact: videoId } };
            if (audioId) constraints.audio = { deviceId: { exact: audioId } };

            try {
                currentStream = await navigator.mediaDevices.getUserMedia(constraints);
            } catch (err) {
                permError.style.display = 'block';
                return;
            }

            // Attach video
            if (currentStream.getVideoTracks().length > 0) {
                videoPreview.innerHTML = '';
                const vid = document.createElement('video');
                vid.srcObject = currentStream;
                vid.muted = true;
                vid.playsInline = true;
                vid.autoplay = true;
                videoPreview.appendChild(vid);
            }

            // Attach audio analyser
            if (currentStream.getAudioTracks().length > 0) {
                audioCtx = new (window.AudioContext || window.webkitAudioContext)();
                const source = audioCtx.createMediaStreamSource(currentStream);
                analyser = audioCtx.createAnalyser();
                analyser.fftSize = 256;
                source.connect(analyser);
                // NOT connected to destination — no feedback
                updateMeter();
            }
        }

        function updateMeter() {
            if (!analyser) return;
            const data = new Uint8Array(analyser.frequencyBinCount);
            analyser.getByteFrequencyData(data);
            let sum = 0;
            for (let i = 0; i < data.length; i++) sum += data[i];
            const avg = sum / data.length;
            const pct = Math.min(100, (avg / 128) * 100);
            meterBar.style.width = pct + '%';
            if (pct < 40) {
                meterBar.style.background = '#49d3b0';
            } else if (pct < 70) {
                meterBar.style.background = '#ffd740';
            } else {
                meterBar.style.background = '#ff5252';
            }
            animFrameId = requestAnimationFrame(updateMeter);
        }

        function stopPreview() {
            if (animFrameId) { cancelAnimationFrame(animFrameId); animFrameId = null; }
            if (audioCtx) { audioCtx.close(); audioCtx = null; analyser = null; }
            if (currentStream) {
                currentStream.getTracks().forEach(function(t) { t.stop(); });
                currentStream = null;
            }
        }

        videoSelect.addEventListener('change', function() {
            localStorage.setItem(LS_VIDEO_KEY, videoSelect.value);
            stopPreview();
            startPreview();
        });

        audioSelect.addEventListener('change', function() {
            localStorage.setItem(LS_AUDIO_KEY, audioSelect.value);
            stopPreview();
            startPreview();
        });

        window.addEventListener('beforeunload', function() {
            stopPreview();
        });

        document.getElementById('back-link').addEventListener('click', function() {
            stopPreview();
        });

        // Profile: load current display name and save
        async function loadProfile() {
            const token = sessionStorage.getItem('user_token') || localStorage.getItem('user_token');
            if (!token) return;
            try {
                const resp = await fetch('/api/users', {
                    headers: { 'x-admin-token': token }
                });
                if (!resp.ok) return;
                const users = await resp.json();
                const me = users.find(function(u) { return u.token === token; });
                if (me && me.display_name) {
                    document.getElementById('display-name-input').value = me.display_name;
                }
            } catch (e) {}
        }

        document.getElementById('save-display-name').addEventListener('click', async function() {
            const token = sessionStorage.getItem('user_token') || localStorage.getItem('user_token');
            if (!token) return;
            const displayName = document.getElementById('display-name-input').value.trim();
            try {
                const resp = await fetch('/api/profile', {
                    method: 'PUT',
                    headers: {
                        'Content-Type': 'application/json',
                        'x-admin-token': token
                    },
                    body: JSON.stringify({ display_name: displayName || null })
                });
                if (resp.ok) {
                    document.getElementById('profile-status').textContent = 'Display name saved!';
                    setTimeout(function() {
                        document.getElementById('profile-status').textContent = '';
                    }, 3000);
                } else {
                    document.getElementById('profile-status').textContent = 'Failed to save.';
                    document.getElementById('profile-status').style.color = '#ff6b6b';
                }
            } catch (e) {
                document.getElementById('profile-status').textContent = 'Error: ' + e.message;
                document.getElementById('profile-status').style.color = '#ff6b6b';
            }
        });

        document.getElementById('change-password-btn').addEventListener('click', async function() {
            const token = sessionStorage.getItem('user_token') || localStorage.getItem('user_token');
            if (!token) return;
            const statusEl = document.getElementById('password-status');
            const currentPw = document.getElementById('current-password').value;
            const newPw = document.getElementById('new-password').value;
            const confirmPw = document.getElementById('confirm-password').value;
            statusEl.textContent = '';
            statusEl.style.color = 'var(--accent)';
            if (!newPw) { statusEl.textContent = 'New password is required.'; statusEl.style.color = '#ff6b6b'; return; }
            if (newPw !== confirmPw) { statusEl.textContent = 'Passwords do not match.'; statusEl.style.color = '#ff6b6b'; return; }
            try {
                const resp = await fetch('/api/profile/password', {
                    method: 'PUT',
                    headers: { 'Content-Type': 'application/json', 'x-admin-token': token },
                    body: JSON.stringify({ current_password: currentPw || null, new_password: newPw })
                });
                if (resp.ok) {
                    statusEl.textContent = 'Password changed successfully!';
                    document.getElementById('current-password').value = '';
                    document.getElementById('new-password').value = '';
                    document.getElementById('confirm-password').value = '';
                    setTimeout(function() { statusEl.textContent = ''; }, 3000);
                } else if (resp.status === 401) {
                    statusEl.textContent = 'Current password is incorrect.';
                    statusEl.style.color = '#ff6b6b';
                } else {
                    statusEl.textContent = 'Failed to change password.';
                    statusEl.style.color = '#ff6b6b';
                }
            } catch(e) {
                statusEl.textContent = 'Error: ' + e.message;
                statusEl.style.color = '#ff6b6b';
            }
        });

        // Tab switching
        document.querySelectorAll('.tab-btn').forEach(function(btn) {
            btn.addEventListener('click', function() {
                document.querySelectorAll('.tab-btn').forEach(function(b) { b.classList.remove('active'); });
                document.querySelectorAll('.tab-panel').forEach(function(p) { p.classList.remove('active'); });
                btn.classList.add('active');
                document.getElementById('tab-' + btn.dataset.tab).classList.add('active');
                stopPreview();
                if (btn.dataset.tab === 'camera' || btn.dataset.tab === 'microphone') {
                    initMedia();
                }
            });
        });

        loadProfile();
    </script>
</body>
</html>"#;