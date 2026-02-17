# BeringShare

A self-hosted, federated chat platform built in Rust. Multiple BeringShare servers can connect to each other, allowing users on different servers to exchange direct messages, participate in shared channels, and join peer-to-peer video calls — all without a central authority.

BeringShare ships with a built-in web UI for chat, administration, and settings, plus an optional Tauri-based desktop client.

---

## Table of Contents

- [Features](#features)
- [Architecture Overview](#architecture-overview)
- [Quick Start](#quick-start)
  - [Run Locally with Cargo](#run-locally-with-cargo)
  - [Run with Docker Compose (Federation)](#run-with-docker-compose-federation)
- [Configuration](#configuration)
- [Setting Up Federation](#setting-up-federation)
- [Authentication](#authentication)
- [API Reference](#api-reference)
  - [Admin API](#admin-api)
  - [User API](#user-api)
  - [Real-Time Events](#real-time-events-sse--websocket)
  - [Federation API](#federation-api)
- [Database Schema](#database-schema)
- [Desktop Client](#desktop-client)
- [Project Structure](#project-structure)
- [Building from Source](#building-from-source)
- [Deployment](#deployment)
- [Contributing](#contributing)
- [License](#license)

---

## Features

### Chat
- **Direct Messages** — Send private messages to any user, local or on a federated server.
- **Channels** — Create group channels with multiple members. Channel messages are replicated to all federated servers that have members in the channel.
- **GIF Search** — Built-in Tenor GIF search (requires API key).
- **Message History** — All messages are persisted in SQLite and available through the API.

### Federation
- **Server-to-Server Replication** — Messages, user presence, and channel membership are synchronized across federated servers in real time.
- **Presence Sync** — Every 2 seconds, each server polls its federation peers for online user status and broadcasts changes to connected clients.
- **Channel & User Discovery** — Federated servers automatically discover each other's channels and users during the presence sync cycle.
- **Message Deduplication** — Messages carry a unique ID to prevent duplicates when relayed across multiple servers.
- **Visibility Controls** — Admins can hide specific users or channels from individual federated servers.

### Video Calling
- **Peer-to-Peer WebRTC** — Video and audio calls are established directly between users via WebRTC. The server only relays signaling (offer/answer/ICE candidates).
- **Channel Group Calls** — Any channel can host a group call. Participants are tracked server-side and synchronized across federated servers.
- **Cross-Server Calls** — Users on different federated servers can call each other. Signaling messages are routed through the federation layer.

### Real-Time Communication
- **Server-Sent Events (SSE)** — Primary push channel for new messages, presence changes, WebRTC signals, and call events.
- **WebSocket Bridge** — Bidirectional WebSocket that receives the same events as SSE and also accepts client-to-server messages (used by the desktop client).

### Authentication & Security
- **Bcrypt Password Hashing** — User passwords are hashed with bcrypt. No plaintext passwords are stored.
- **Session Tokens** — Login produces a 24-hour session token. Expired sessions return 401, and the UI redirects to the login screen.
- **Admin Sessions** — Admin login produces a 1-hour session token.
- **Admin-Managed Users** — Only admins can create user accounts (with or without an initial password). Users can change their own password in Settings.
- **Federation Tokens** — Server-to-server requests are authenticated with per-server tokens or shared federation tokens.

### Built-In Web UI
- **Chat UI** (`/chat/ui`) — Full-featured chat interface with channels, DMs, presence indicators, video calling, GIF picker, and a debug console.
- **Admin Panel** (`/admin/ui`) — Manage users, servers, channels, federation tokens, and visibility settings.
- **Settings** (`/chat/settings`) — Profile (display name), password change, camera/microphone device selection with live preview.

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────┐
│                    BeringShare Server                    │
│                                                         │
│  ┌──────────┐  ┌──────────┐  ┌───────────────────────┐  │
│  │ Admin API │  │ User API │  │   Federation API      │  │
│  │ /admin/* │  │ /api/*   │  │   /federation/*       │  │
│  └────┬─────┘  └────┬─────┘  └───────────┬───────────┘  │
│       │              │                    │              │
│  ┌────┴──────────────┴────────────────────┴───────────┐  │
│  │              Axum Router + AppState                 │  │
│  ├────────────┬──────────┬────────────┬───────────────┤  │
│  │  SQLite    │ Sessions │  Presence  │ Channel Calls │  │
│  │  Store     │ (Admin + │  Store     │ Store         │  │
│  │            │  User)   │            │               │  │
│  └────────────┴──────────┴────────────┴───────────────┘  │
│                         │                                │
│              ┌──────────┴──────────┐                     │
│              │  SSE / WebSocket    │                     │
│              │  Broadcaster        │                     │
│              └─────────────────────┘                     │
└─────────────────────────────────────────────────────────┘
          │                                    │
          ▼                                    ▼
   ┌─────────────┐                   ┌──────────────────┐
   │  Web Browser │                   │ Federated Server │
   │  (Chat UI)   │                   │ (HTTP + Tokens)  │
   └─────────────┘                   └──────────────────┘
```

Each server is a single Rust binary that embeds its own web UI, stores data in SQLite, and communicates with federation peers over HTTP. There is no external dependency beyond the binary itself.

---

## Quick Start

### Run Locally with Cargo

**Prerequisites:** Rust 1.78+ installed ([rustup.rs](https://rustup.rs/))

```bash
git clone https://github.com/yourusername/beringshare.git
cd beringshare
cargo run -p federated-server
```

The server starts on `http://localhost:8080`. Open the following in your browser:

| URL | Description |
|-----|-------------|
| `http://localhost:8080/admin/ui` | Admin panel — create users, channels, manage federation |
| `http://localhost:8080/chat/ui` | Chat interface — login as a user to send messages |
| `http://localhost:8080/chat/settings` | User settings — profile, password, camera/microphone |

**First-time setup:**
1. Open the Admin panel and log in (default: `admin` / `admin`).
2. Create a few users (e.g., `alice`, `bob`). Optionally set passwords.
3. Create a channel (e.g., `general`).
4. Add users as channel members.
5. Open the Chat UI and log in as one of the users.

### Run with Docker Compose (Federation)

The included `docker-compose.yml` launches three federated servers:

```bash
docker compose up --build
```

| Server | URL | Admin Token | Server Token |
|--------|-----|-------------|--------------|
| Server A | `http://localhost:8081` | `admin-a` | `token-a` |
| Server B | `http://localhost:8082` | `admin-b` | `token-b` |
| Server C | `http://localhost:8083` | `admin-c` | `token-c` |

Each server has its own isolated SQLite database in a Docker volume. The default admin credentials for all three are `admin` / `admin` (configurable via `ADMIN_USERNAME` and `ADMIN_PASSWORD` environment variables).

To set up federation between them, see [Setting Up Federation](#setting-up-federation) below.

---

## Configuration

All configuration is done through environment variables. Every variable has a sensible default for local development.

| Variable | Default | Description |
|----------|---------|-------------|
| `SERVER_NAME` | `local` | Unique name for this server in the federation. Used in user addresses (e.g., `alice@server_a`). |
| `BASE_URL` | `http://localhost:8080` | The URL other servers use to reach this server. Must be routable from federation peers. |
| `DATABASE_PATH` | `./data.sqlite` | Path to the SQLite database file. Created automatically on first run. |
| `ADMIN_TOKEN` | `admin-token` | Static token for admin API access (used as fallback alongside session tokens). |
| `SERVER_TOKEN` | `server-token` | Token this server uses to authenticate outgoing federation requests. |
| `ADMIN_USERNAME` | `admin` | Username for the admin user account. Created/updated on startup. |
| `ADMIN_PASSWORD` | `admin` | Password for the admin user account. Hashed with bcrypt and synced on every startup. |
| `TENOR_API_KEY` | *(none)* | Optional. Enables GIF search in the chat UI via the Tenor API. |
| `RUST_LOG` | *(none)* | Logging level. Examples: `info`, `debug`, `warn`, `federated_server=debug`. |

**Important:** In production, change `ADMIN_TOKEN`, `SERVER_TOKEN`, `ADMIN_PASSWORD`, and any federation tokens to strong, unique values.

---

## Setting Up Federation

Federation allows users on different BeringShare servers to communicate. Here's how to connect two servers:

### 1. Register Servers as Peers

On **Server A** (`http://localhost:8081/admin/ui`):
1. Log in as admin.
2. In the **Federated Servers** section, register Server B:
   - **Name:** `b`
   - **Base URL:** `http://server_b:8080` (Docker internal DNS) or `http://localhost:8082` (if running locally)
   - **Token:** The `SERVER_TOKEN` of Server B (e.g., `token-b`)

On **Server B** (`http://localhost:8082/admin/ui`):
1. Register Server A with its name, URL, and token.

### 2. Create Users

On each server, create local users via the admin panel. For example:
- Server A: `alice`, `bob`
- Server B: `charlie`, `dave`

### 3. Sync Users and Channels

Federation sync happens automatically every 2 seconds (presence, channels, display names). You can also manually trigger a full user sync from the admin panel by clicking **"Fetch Users From Federated Servers"**.

After sync, remote users appear in each server's user list. In the chat UI, users from other servers are displayed alongside local users.

### 4. Communicate

- **DMs:** Click on a remote user in the Chat UI to send a direct message. The message is routed through the federation layer to the remote server.
- **Channels:** Create a channel on any server and add members from multiple servers. Messages are replicated to all servers with members.
- **Calls:** Start a video call from any channel. Participants from different servers can join — WebRTC signaling is routed through federation.

### Federation Token Authentication

Federation requests use the `x-federation-token` HTTP header. A request is accepted if the token matches any of:
1. A known server's token in the servers table.
2. A custom federation token created in the admin panel.
3. The server's own `SERVER_TOKEN`.

---

## Authentication

### Admin Authentication

Admins authenticate via one of:
- **Session token**: `POST /admin/login` with username and password returns a 1-hour session token.
- **Static token**: The `ADMIN_TOKEN` environment variable (for scripts and automation).

Send the token in the `x-admin-token` header or as `Authorization: Bearer <token>`.

### User Authentication

Users authenticate via:
- **Session token**: `POST /api/login` with username and password returns a 24-hour session token.
- **Legacy DB token**: Each user has a permanent token stored in the database (for backwards compatibility).

Send the token as `Authorization: Bearer <token>` or in the `x-admin-token` header.

When a session expires, the API returns `401 Unauthorized`. The chat UI automatically detects this and redirects to the login screen.

### Admin User Bootstrap

On every startup, the server ensures the admin user exists with the credentials from `ADMIN_USERNAME` and `ADMIN_PASSWORD`. If the admin user already exists, their password is updated to match the environment. This means you can change the admin password by updating the environment variable and restarting.

---

## API Reference

### Admin API

All admin endpoints are under `/admin` and require admin authentication.

| Method | Endpoint | Description |
|--------|----------|-------------|
| `POST` | `/admin/login` | Admin login. Body: `{ "username", "password" }`. Returns `{ "token" }`. |
| `POST` | `/admin/users` | Create user. Body: `{ "username", "password"? }`. |
| `GET` | `/admin/users` | List all users. |
| `PUT` | `/admin/users/:id` | Update user. Body: `{ "username", "display_name"?, "password"? }`. |
| `DELETE` | `/admin/users/:id` | Delete user. |
| `POST` | `/admin/servers` | Register federated server. Body: `{ "name", "base_url", "token"? }`. |
| `GET` | `/admin/servers` | List federated servers. |
| `PUT` | `/admin/servers/:id` | Update server. Body: `{ "name", "base_url", "token"? }`. |
| `DELETE` | `/admin/servers/:id` | Delete server. |
| `GET` | `/admin/servers/:id/visibility` | Get hidden users/channels for a server. |
| `PUT` | `/admin/servers/:id/visibility` | Set hidden users/channels. Body: `{ "hidden_user_ids", "hidden_channel_ids" }`. |
| `POST` | `/admin/channels` | Create channel. Body: `{ "name" }`. |
| `GET` | `/admin/channels` | List channels. |
| `PUT` | `/admin/channels/:id` | Update channel. Body: `{ "name" }`. |
| `DELETE` | `/admin/channels/:id` | Delete channel. |
| `POST` | `/admin/channels/:id/members` | Add channel member. Body: `{ "username" }` (supports `user@server`). |
| `GET` | `/admin/server-info` | Get this server's name and token. |
| `GET` | `/admin/federation-tokens` | List federation tokens. |
| `POST` | `/admin/federation-tokens` | Create federation token. Body: `{ "label" }`. |
| `DELETE` | `/admin/federation-tokens/:id` | Delete federation token. |
| `POST` | `/admin/users/sync-federated` | Manually sync users from all federated servers. |
| `POST` | `/admin/channels/sync-federated` | Manually sync channels from all federated servers. |

### User API

All user endpoints are under `/api` and require user authentication.

| Method | Endpoint | Description |
|--------|----------|-------------|
| `POST` | `/api/login` | User login. Body: `{ "username", "password" }`. Returns `{ "user_id", "username", "token", "display_name" }`. |
| `GET` | `/api/users` | List all users (local and remote) with online status. |
| `GET` | `/api/channels` | List all channels. |
| `POST` | `/api/channels` | Create a channel. Body: `{ "name" }`. |
| `POST` | `/api/channels/:id/members` | Add member. Body: `{ "user_id" }`. |
| `DELETE` | `/api/channels/:id/members/:user_id` | Remove member. |
| `POST` | `/api/messages/dm` | Send DM. Body: `{ "recipient", "body" }`. Recipient can be `"alice"` or `"alice@server_b"`. |
| `POST` | `/api/messages/channel` | Send channel message. Body: `{ "channel", "body", "origin_server"? }`. |
| `GET` | `/api/messages/inbox` | Get recent DMs and channel messages (limit 50). |
| `GET` | `/api/messages/channel/:id` | Get all messages in a channel. |
| `GET` | `/api/messages/dm/:user_id` | Get DM conversation with a user. |
| `PUT` | `/api/profile` | Update profile. Body: `{ "display_name"? }`. |
| `PUT` | `/api/profile/password` | Change password. Body: `{ "current_password"?, "new_password" }`. |
| `POST` | `/api/channels/:id/call/join` | Join channel group call. Returns current participants. |
| `POST` | `/api/channels/:id/call/leave` | Leave channel group call. |
| `GET` | `/api/channels/:id/call/participants` | Get call participants. |
| `GET` | `/api/channels/active-calls` | Get all active calls. Returns `{ "channel_id": participant_count }`. |
| `POST` | `/api/call/signal` | Send WebRTC signal. Body: `{ "target", "signal_type", "payload" }`. |
| `GET` | `/api/gif/search?q=&limit=` | Search for GIFs via Tenor. Requires `TENOR_API_KEY`. |

### Real-Time Events (SSE / WebSocket)

**SSE:** `GET /api/events?token=<token>`

Connects to the server's event stream. The server sends JSON events:

| Event | Description | Fields |
|-------|-------------|--------|
| `new_message` | A new DM or channel message arrived. | `user_id`, `channel_id` |
| `presence_changed` | A user came online or went offline. | *(none — clients should re-fetch user list)* |
| `webrtc_signal` | WebRTC offer/answer/ICE candidate for a call. | `target_user_id`, `payload` |
| `channel_call_join` | A user joined a channel call. | `channel_id`, `payload` (JSON with username, server, user_id) |
| `channel_call_leave` | A user left a channel call. | `channel_id`, `payload` |

The SSE connection also registers the user as "online" for presence tracking. When the connection closes, the user is marked offline and any active calls are cleaned up.

**WebSocket:** `GET /api/ws` (token via header or query)

Bidirectional WebSocket that receives the same events as SSE. Additionally accepts messages from the client:

```json
{ "kind": "send_channel_message", "channel_id": "...", "body": "..." }
```

### Federation API

These endpoints are called by other BeringShare servers. Authentication is via the `x-federation-token` header.

| Method | Endpoint | Description |
|--------|----------|-------------|
| `POST` | `/federation/messages` | Receive a federated message (DM or channel). |
| `POST` | `/federation/channel-memberships` | Add a user to a channel (cross-server). |
| `GET` | `/federation/presence` | Get list of online local users. |
| `GET` | `/federation/users` | Get list of local users with display names. |
| `GET` | `/federation/channels` | Get list of locally-originated channels. |
| `POST` | `/federation/webrtc-signal` | Relay a WebRTC signaling message. |
| `POST` | `/federation/channel-call-event` | Relay a channel call join/leave event. |

---

## Database Schema

BeringShare uses SQLite with the following tables:

```sql
-- Federated server registry
servers (id, name UNIQUE, base_url, token)

-- User accounts (local and remote references)
users (id, username, token, server_id?, is_local, display_name?, password_hash?)
  UNIQUE(username, server_id)

-- Chat channels
channels (id, name, origin_server)
  UNIQUE(name, origin_server)

-- Channel membership (many-to-many)
channel_members (channel_id, user_id)

-- Messages (DMs and channel messages)
messages (id, kind, body, author_user_id, recipient_user_id?, channel_id?, sent_at)

-- Custom federation tokens
federation_tokens (id, token UNIQUE, label, created_at)

-- Per-server visibility controls
server_hidden_users (server_id, user_id)
server_hidden_channels (server_id, channel_id)
```

Migrations run automatically on startup. The schema is extended with `ALTER TABLE` for new columns (e.g., `display_name`, `password_hash`), wrapped in idempotent checks.

---

## Desktop Client

BeringShare includes an optional desktop client built with [Tauri](https://tauri.app/) (Rust backend) and React (TypeScript frontend).

```
client-desktop/
├── src/                    # React frontend
│   ├── App.tsx             # Main app shell
│   ├── components/
│   │   ├── Login.tsx       # Login form
│   │   ├── ChannelList.tsx # Channel sidebar
│   │   ├── ChatView.tsx    # Message view
│   │   └── Settings.tsx    # Settings panel
│   └── services/
│       ├── api.ts          # HTTP API client (axios)
│       ├── auth.ts         # Token management (secure keyring storage)
│       └── ws.ts           # WebSocket client
└── src-tauri/              # Tauri backend (Rust)
    └── src/main.rs         # Native WebSocket bridge, secure token storage
```

### Running the Desktop Client

```bash
cd client-desktop
npm install
npm run dev          # Development mode (web)
npx tauri dev        # Development mode (native window)
npx tauri build      # Production build
```

The desktop client connects to a configurable server URL and uses the Tauri keyring API for secure token storage.

---

## Project Structure

```
beringshare/
├── Cargo.toml                    # Workspace root
├── Dockerfile                    # Multi-stage Docker build
├── docker-compose.yml            # 3-server federation setup
│
├── crates/server/                # Server binary
│   ├── Cargo.toml                # Dependencies (axum, rusqlite, bcrypt, etc.)
│   └── src/
│       ├── main.rs               # Entry point, admin user bootstrap
│       ├── lib.rs                # Module declarations
│       ├── config.rs             # Environment variable configuration
│       ├── error.rs              # AppError enum → HTTP status codes
│       ├── domain/mod.rs         # Data models (User, Server, Channel, Message)
│       ├── storage/sqlite.rs     # SQLite storage layer
│       ├── auth/
│       │   ├── mod.rs            # AdminGuard, UserGuard (axum extractors)
│       │   └── sessions.rs       # Admin + User session management
│       ├── api/
│       │   ├── mod.rs            # Router assembly, AppState, presence sync task
│       │   ├── admin.rs          # Admin CRUD endpoints
│       │   ├── messages.rs       # User messaging, login, password change
│       │   └── web.rs            # Embedded HTML/JS for all three web UIs
│       ├── federation/
│       │   ├── mod.rs            # Federation router
│       │   ├── protocol.rs       # Federation message types
│       │   ├── handlers.rs       # Inbound federation message handlers
│       │   └── outbox.rs         # Outbound federation message sending
│       ├── websocket.rs          # SSE handler, event broadcaster
│       ├── ws_bridge.rs          # WebSocket bridge handler
│       ├── presence.rs           # Online/offline tracking (local + remote)
│       └── channel_call.rs       # Channel group call participant tracking
│
└── client-desktop/               # Optional Tauri desktop client
    ├── package.json
    ├── src/                      # React + TypeScript
    └── src-tauri/                # Tauri Rust backend
```

---

## Building from Source

### Prerequisites

- **Rust** 1.78 or later — [Install Rust](https://rustup.rs/)
- **Docker** and **Docker Compose** (optional, for federation testing)
- **Node.js** 18+ and **npm** (only if building the desktop client)

### Build the Server

```bash
cargo build -p federated-server --release
```

The binary is output to `target/release/federated-server`.

### Run Tests

```bash
cargo test -p federated-server
```

### Build the Docker Image

```bash
docker build -t beringshare .
```

### Build the Desktop Client

```bash
cd client-desktop
npm install
npx tauri build
```

---

## Deployment

### Single Server

Run the binary directly or via Docker:

```bash
# Direct
DATABASE_PATH=/data/db.sqlite \
SERVER_NAME=myserver \
ADMIN_PASSWORD=changeme \
ADMIN_TOKEN=changeme \
SERVER_TOKEN=changeme \
./federated-server

# Docker
docker run -d \
  -p 8080:8080 \
  -v beringshare-data:/data \
  -e DATABASE_PATH=/data/db.sqlite \
  -e SERVER_NAME=myserver \
  -e ADMIN_PASSWORD=changeme \
  -e ADMIN_TOKEN=changeme \
  -e SERVER_TOKEN=changeme \
  beringshare
```

### Federated Cluster

Use `docker-compose.yml` as a starting point. For each server:

1. Set a unique `SERVER_NAME`.
2. Set `BASE_URL` to the URL that other servers can reach this server at.
3. Set unique, strong values for `ADMIN_TOKEN`, `SERVER_TOKEN`, and `ADMIN_PASSWORD`.
4. After all servers are running, register each server as a peer on the others via the admin panel.

### Reverse Proxy

BeringShare uses standard HTTP, SSE, and WebSocket connections on a single port. Any reverse proxy (nginx, Caddy, Traefik) that supports WebSocket upgrades will work. Example nginx config:

```nginx
location / {
    proxy_pass http://127.0.0.1:8080;
    proxy_http_version 1.1;
    proxy_set_header Upgrade $http_upgrade;
    proxy_set_header Connection "upgrade";
    proxy_set_header Host $host;
    proxy_set_header X-Real-IP $remote_addr;
    proxy_read_timeout 86400s;  # Keep SSE connections alive
}
```

---

## Contributing

Contributions are welcome. Please open an issue to discuss significant changes before submitting a pull request.

### Development Setup

1. Clone the repository.
2. Run `cargo run -p federated-server` for a quick dev server.
3. The web UIs are embedded in `crates/server/src/api/web.rs` — edit the HTML/JS inline and restart the server to see changes.
4. For federation testing, use `docker compose up --build` to spin up three connected servers.

### Key Dependencies

| Crate | Purpose |
|-------|---------|
| `axum` 0.7 | HTTP framework with WebSocket support |
| `tokio` 1.36 | Async runtime |
| `rusqlite` 0.31 | SQLite database (bundled, no external dependency) |
| `bcrypt` 0.15 | Password hashing |
| `reqwest` 0.12 | HTTP client for federation |
| `serde` / `serde_json` | Serialization |
| `uuid` 1.6 | ID generation |
| `time` 0.3 | Timestamp formatting (RFC3339) |
| `tracing` 0.1 | Structured logging |

---

## License

This project is open source. See the [LICENSE](LICENSE) file for details.
