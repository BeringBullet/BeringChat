use crate::domain::{Channel, FederationToken, Message, MessageKind, Server, User};
use crate::error::AppError;
use rusqlite::{params, Connection, OptionalExtension};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Clone)]
pub struct SqliteStore {
    conn: Arc<Mutex<Connection>>,
}

impl SqliteStore {
    pub fn new(path: &str) -> Result<Self, AppError> {
        let conn = Connection::open(path)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn init(&self) -> Result<(), AppError> {
        let conn = self.conn.lock().expect("db mutex");
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS servers (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                base_url TEXT NOT NULL,
                token TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY,
                username TEXT NOT NULL,
                token TEXT NOT NULL,
                server_id TEXT,
                is_local INTEGER NOT NULL,
                UNIQUE(username, server_id)
            );
            CREATE TABLE IF NOT EXISTS channels (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                origin_server TEXT NOT NULL,
                UNIQUE(name, origin_server)
            );
            CREATE TABLE IF NOT EXISTS channel_members (
                channel_id TEXT NOT NULL,
                user_id TEXT NOT NULL,
                PRIMARY KEY(channel_id, user_id)
            );
            CREATE TABLE IF NOT EXISTS messages (
                id TEXT PRIMARY KEY,
                kind TEXT NOT NULL,
                body TEXT NOT NULL,
                author_user_id TEXT NOT NULL,
                recipient_user_id TEXT,
                channel_id TEXT,
                sent_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS federation_tokens (
                id TEXT PRIMARY KEY,
                token TEXT NOT NULL UNIQUE,
                label TEXT NOT NULL,
                created_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS server_hidden_users (
                server_id TEXT NOT NULL,
                user_id TEXT NOT NULL,
                PRIMARY KEY(server_id, user_id)
            );
            CREATE TABLE IF NOT EXISTS server_hidden_channels (
                server_id TEXT NOT NULL,
                channel_id TEXT NOT NULL,
                PRIMARY KEY(server_id, channel_id)
            );
            ",
        )?;
        // Migration: add display_name column if not present
        let _ = conn.execute_batch("ALTER TABLE users ADD COLUMN display_name TEXT;");
        // Migration: add password_hash column if not present
        let _ = conn.execute_batch("ALTER TABLE users ADD COLUMN password_hash TEXT;");
        Ok(())
    }

    pub fn ensure_server(&self, name: &str, base_url: &str, token: &str) -> Result<Server, AppError> {
        if let Some(existing) = self.get_server_by_name(name)? {
            return Ok(existing);
        }
        self.create_server(name, base_url, token)
    }

    pub fn create_server(&self, name: &str, base_url: &str, token: &str) -> Result<Server, AppError> {
        let id = Uuid::new_v4();
        let conn = self.conn.lock().expect("db mutex");
        conn.execute(
            "INSERT INTO servers (id, name, base_url, token) VALUES (?1, ?2, ?3, ?4)",
            params![id.to_string(), name, base_url, token],
        )?;
        Ok(Server {
            id,
            name: name.to_string(),
            base_url: base_url.to_string(),
            token: token.to_string(),
        })
    }

    pub fn seed_initial_data(&self, server_name: &str) -> Result<(), AppError> {
        tracing::info!("Seeding federation references for server '{}'", server_name);
        
        // Create federation references to other servers if not present
        let other_servers = match server_name {
            "a" => vec![
                ("b", "http://server_b:8080", "token-b"),
                ("c", "http://server_c:8080", "token-c"),
            ],
            "b" => vec![
                ("a", "http://server_a:8080", "token-a"),
                ("c", "http://server_c:8080", "token-c"),
            ],
            "c" => vec![
                ("a", "http://server_a:8080", "token-a"),
                ("b", "http://server_b:8080", "token-b"),
            ],
            _ => vec![],
        };

        for (name, url, token) in other_servers {
            self.ensure_server(name, url, token)?;
            tracing::debug!("Federation reference to server '{}' ensured", name);
        }

        Ok(())
    }

    pub fn get_server_by_name(&self, name: &str) -> Result<Option<Server>, AppError> {
        let conn = self.conn.lock().expect("db mutex");
        conn.query_row(
            "SELECT id, name, base_url, token FROM servers WHERE name = ?1",
            params![name],
            |row| row_to_server(row),
        )
        .optional()
        .map_err(AppError::from)
    }

    pub fn get_server_by_id(&self, id: &Uuid) -> Result<Option<Server>, AppError> {
        let conn = self.conn.lock().expect("db mutex");
        conn.query_row(
            "SELECT id, name, base_url, token FROM servers WHERE id = ?1",
            params![id.to_string()],
            |row| row_to_server(row),
        )
        .optional()
        .map_err(AppError::from)
    }

    pub fn get_server_by_token(&self, token: &str) -> Result<Option<Server>, AppError> {
        let conn = self.conn.lock().expect("db mutex");
        conn.query_row(
            "SELECT id, name, base_url, token FROM servers WHERE token = ?1",
            params![token],
            |row| row_to_server(row),
        )
        .optional()
        .map_err(AppError::from)
    }

    pub fn list_servers(&self) -> Result<Vec<Server>, AppError> {
        let conn = self.conn.lock().expect("db mutex");
        let mut stmt = conn.prepare(
            "SELECT id, name, base_url, token FROM servers ORDER BY name",
        )?;
        let rows = stmt.query_map([], |row| row_to_server(row))?;
        let mut servers = Vec::new();
        for row in rows {
            servers.push(row?);
        }
        Ok(servers)
    }

    pub fn create_user(
        &self,
        username: &str,
        is_local: bool,
        server_id: Option<Uuid>,
    ) -> Result<User, AppError> {
        self.create_user_with_password(username, is_local, server_id, None)
    }

    pub fn create_user_with_password(
        &self,
        username: &str,
        is_local: bool,
        server_id: Option<Uuid>,
        password_hash: Option<&str>,
    ) -> Result<User, AppError> {
        let id = Uuid::new_v4();
        let token = if is_local {
            Uuid::new_v4().to_string()
        } else {
            "".to_string()
        };
        let conn = self.conn.lock().expect("db mutex");
        conn.execute(
            "INSERT INTO users (id, username, token, server_id, is_local, password_hash) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![id.to_string(), username, token, server_id.map(|s| s.to_string()), is_local as i32, password_hash],
        )?;
        Ok(User {
            id,
            username: username.to_string(),
            token,
            server_id,
            is_local,
            display_name: None,
        })
    }

    pub fn list_users(&self) -> Result<Vec<User>, AppError> {
        let conn = self.conn.lock().expect("db mutex");
        let mut stmt = conn.prepare(
            "SELECT id, username, token, server_id, is_local, display_name FROM users ORDER BY username",
        )?;
        let rows = stmt.query_map([], |row| row_to_user(row))?;
        let mut users = Vec::new();
        for row in rows {
            users.push(row?);
        }
        Ok(users)
    }

    pub fn get_user_by_token(&self, token: &str) -> Result<Option<User>, AppError> {
        let conn = self.conn.lock().expect("db mutex");
        conn.query_row(
            "SELECT id, username, token, server_id, is_local, display_name FROM users WHERE token = ?1",
            params![token],
            |row| row_to_user(row),
        )
        .optional()
        .map_err(AppError::from)
    }

    pub fn get_user_by_name_and_server(
        &self,
        username: &str,
        server_id: Option<Uuid>,
    ) -> Result<Option<User>, AppError> {
        let conn = self.conn.lock().expect("db mutex");
        conn.query_row(
            "SELECT id, username, token, server_id, is_local, display_name FROM users WHERE username = ?1 AND COALESCE(server_id, '') = COALESCE(?2, '')",
            params![username, server_id.map(|s| s.to_string())],
            |row| row_to_user(row),
        )
        .optional()
        .map_err(AppError::from)
    }

    pub fn get_user_by_id(&self, user_id: Uuid) -> Result<Option<User>, AppError> {
        let conn = self.conn.lock().expect("db mutex");
        conn.query_row(
            "SELECT id, username, token, server_id, is_local, display_name FROM users WHERE id = ?1",
            params![user_id.to_string()],
            |row| row_to_user(row),
        )
        .optional()
        .map_err(AppError::from)
    }

    pub fn create_channel(&self, name: &str, origin_server: &str) -> Result<Channel, AppError> {
        let id = Uuid::new_v4();
        let conn = self.conn.lock().expect("db mutex");
        conn.execute(
            "INSERT INTO channels (id, name, origin_server) VALUES (?1, ?2, ?3)",
            params![id.to_string(), name, origin_server],
        )?;
        Ok(Channel {
            id,
            name: name.to_string(),
            origin_server: origin_server.to_string(),
        })
    }

    pub fn list_channels(&self) -> Result<Vec<Channel>, AppError> {
        let conn = self.conn.lock().expect("db mutex");
        let mut stmt = conn.prepare(
            "SELECT id, name, origin_server FROM channels ORDER BY name",
        )?;
        let rows = stmt.query_map([], |row| row_to_channel(row))?;
        let mut channels = Vec::new();
        for row in rows {
            channels.push(row?);
        }
        Ok(channels)
    }

    pub fn get_channel_by_name_origin(
        &self,
        name: &str,
        origin_server: &str,
    ) -> Result<Option<Channel>, AppError> {
        let conn = self.conn.lock().expect("db mutex");
        conn.query_row(
            "SELECT id, name, origin_server FROM channels WHERE name = ?1 AND origin_server = ?2",
            params![name, origin_server],
            |row| row_to_channel(row),
        )
        .optional()
        .map_err(AppError::from)
    }

    pub fn get_channel_by_id(&self, id: Uuid) -> Result<Option<Channel>, AppError> {
        let conn = self.conn.lock().expect("db mutex");
        conn.query_row(
            "SELECT id, name, origin_server FROM channels WHERE id = ?1",
            params![id.to_string()],
            |row| row_to_channel(row),
        )
        .optional()
        .map_err(AppError::from)
    }

    pub fn add_channel_member(&self, channel_id: Uuid, user_id: Uuid) -> Result<(), AppError> {
        let conn = self.conn.lock().expect("db mutex");
        conn.execute(
            "INSERT OR IGNORE INTO channel_members (channel_id, user_id) VALUES (?1, ?2)",
            params![channel_id.to_string(), user_id.to_string()],
        )?;
        Ok(())
    }

    pub fn list_channel_member_servers(&self, channel_id: Uuid) -> Result<Vec<Server>, AppError> {
        let conn = self.conn.lock().expect("db mutex");
        let mut stmt = conn.prepare(
            "
            SELECT DISTINCT s.id, s.name, s.base_url, s.token
            FROM channel_members cm
            JOIN users u ON cm.user_id = u.id
            JOIN servers s ON u.server_id = s.id
            WHERE cm.channel_id = ?1 AND u.server_id IS NOT NULL
            ",
        )?;
        let rows = stmt.query_map(params![channel_id.to_string()], |row| row_to_server(row))?;

        let mut servers = Vec::new();
        for row in rows {
            servers.push(row?);
        }
        Ok(servers)
    }

    pub fn create_message(
        &self,
        kind: MessageKind,
        body: &str,
        author_user_id: Uuid,
        recipient_user_id: Option<Uuid>,
        channel_id: Option<Uuid>,
        sent_at: &str,
    ) -> Result<Message, AppError> {
        let id = Uuid::new_v4();
        let conn = self.conn.lock().expect("db mutex");
        conn.execute(
            "INSERT INTO messages (id, kind, body, author_user_id, recipient_user_id, channel_id, sent_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                id.to_string(),
                match kind {
                    MessageKind::Dm => "dm",
                    MessageKind::Channel => "channel",
                },
                body,
                author_user_id.to_string(),
                recipient_user_id.map(|id| id.to_string()),
                channel_id.map(|id| id.to_string()),
                sent_at,
            ],
        )?;
        Ok(Message {
            id,
            kind,
            body: body.to_string(),
            author_user_id,
            recipient_user_id,
            channel_id,
            sent_at: sent_at.to_string(),
        })
    }

    pub fn create_message_with_id(
        &self,
        id_str: &str,
        kind: MessageKind,
        body: &str,
        author_user_id: Uuid,
        recipient_user_id: Option<Uuid>,
        channel_id: Option<Uuid>,
        sent_at: &str,
    ) -> Result<Option<Message>, AppError> {
        let id = Uuid::parse_str(id_str)
            .map_err(|_| AppError::BadRequest("invalid message_id".to_string()))?;

        let conn = self.conn.lock().expect("db mutex");

        let inserted = conn.execute(
            "INSERT OR IGNORE INTO messages (id, kind, body, author_user_id, recipient_user_id, channel_id, sent_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                id.to_string(),
                match kind {
                    MessageKind::Dm => "dm",
                    MessageKind::Channel => "channel",
                },
                body,
                author_user_id.to_string(),
                recipient_user_id.map(|id| id.to_string()),
                channel_id.map(|id| id.to_string()),
                sent_at,
            ],
        )?;

        if inserted == 0 {
            return Ok(None);
        }

        Ok(Some(Message {
            id,
            kind,
            body: body.to_string(),
            author_user_id,
            recipient_user_id,
            channel_id,
            sent_at: sent_at.to_string(),
        }))
    }

    pub fn list_messages_for_user(&self, user_id: Uuid, limit: usize) -> Result<Vec<Message>, AppError> {
        let conn = self.conn.lock().expect("db mutex");
        let mut messages = Vec::new();

        let mut stmt = conn.prepare(
            "SELECT id, kind, body, author_user_id, recipient_user_id, channel_id, sent_at
             FROM messages
             WHERE recipient_user_id = ?1
             ORDER BY sent_at DESC
             LIMIT ?2",
        )?;
        let rows = stmt.query_map(params![user_id.to_string(), limit as i64], row_to_message)?;
        for row in rows {
            messages.push(row?);
        }

        let channel_ids = self.list_channel_ids_for_user(user_id)?;
        for channel_id in channel_ids {
            let mut stmt = conn.prepare(
                "SELECT id, kind, body, author_user_id, recipient_user_id, channel_id, sent_at
                 FROM messages
                 WHERE channel_id = ?1
                 ORDER BY sent_at DESC
                 LIMIT ?2",
            )?;
            let rows = stmt.query_map(params![channel_id.to_string(), limit as i64], row_to_message)?;
            for row in rows {
                messages.push(row?);
            }
        }

        messages.sort_by(|a, b| b.sent_at.cmp(&a.sent_at));
        messages.truncate(limit);
        Ok(messages)
    }

    fn list_channel_ids_for_user(&self, user_id: Uuid) -> Result<Vec<Uuid>, AppError> {
        let conn = self.conn.lock().expect("db mutex");
        let mut stmt = conn.prepare(
            "SELECT channel_id FROM channel_members WHERE user_id = ?1",
        )?;
        let rows = stmt.query_map(params![user_id.to_string()], |row| {
            let id: String = row.get(0)?;
            let uuid = Uuid::parse_str(&id).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
            })?;
            Ok(uuid)
        })?;
        let mut ids = Vec::new();
        for row in rows {
            ids.push(row?);
        }
        Ok(ids)
    }

    pub fn set_user_password(&self, user_id: &Uuid, hash: &str) -> Result<(), AppError> {
        let conn = self.conn.lock().expect("db mutex");
        conn.execute(
            "UPDATE users SET password_hash = ?1 WHERE id = ?2",
            params![hash, user_id.to_string()],
        )?;
        Ok(())
    }

    pub fn get_user_password_hash(&self, user_id: &Uuid) -> Result<Option<String>, AppError> {
        let conn = self.conn.lock().expect("db mutex");
        conn.query_row(
            "SELECT password_hash FROM users WHERE id = ?1",
            params![user_id.to_string()],
            |row| row.get::<_, Option<String>>(0),
        )
        .optional()
        .map(|opt| opt.flatten())
        .map_err(AppError::from)
    }

    pub fn delete_user(&self, id: &Uuid) -> Result<(), AppError> {
        let conn = self.conn.lock().expect("db mutex");
        conn.execute("DELETE FROM users WHERE id = ?1", params![id.to_string()])?;
        Ok(())
    }

    pub fn update_user(&self, id: &Uuid, username: &str, display_name: Option<&str>) -> Result<User, AppError> {
        let conn = self.conn.lock().expect("db mutex");
        conn.execute(
            "UPDATE users SET username = ?1, display_name = ?2 WHERE id = ?3",
            params![username, display_name, id.to_string()],
        )?;
        conn.query_row(
            "SELECT id, username, token, server_id, is_local, display_name FROM users WHERE id = ?1",
            params![id.to_string()],
            |row| row_to_user(row),
        )
        .map_err(AppError::from)
    }

    pub fn update_user_display_name(&self, id: &Uuid, display_name: Option<&str>) -> Result<(), AppError> {
        let conn = self.conn.lock().expect("db mutex");
        conn.execute(
            "UPDATE users SET display_name = ?1 WHERE id = ?2",
            params![display_name, id.to_string()],
        )?;
        Ok(())
    }

    pub fn delete_server(&self, id: &Uuid) -> Result<(), AppError> {
        let conn = self.conn.lock().expect("db mutex");
        conn.execute("DELETE FROM server_hidden_users WHERE server_id = ?1", params![id.to_string()])?;
        conn.execute("DELETE FROM server_hidden_channels WHERE server_id = ?1", params![id.to_string()])?;
        conn.execute("DELETE FROM servers WHERE id = ?1", params![id.to_string()])?;
        Ok(())
    }

    pub fn update_server(&self, id: &Uuid, name: &str, base_url: &str, token: &str) -> Result<Server, AppError> {
        let conn = self.conn.lock().expect("db mutex");
        conn.execute(
            "UPDATE servers SET name = ?1, base_url = ?2, token = ?3 WHERE id = ?4",
            params![name, base_url, token, id.to_string()],
        )?;
        let server = conn.query_row(
            "SELECT id, name, base_url, token FROM servers WHERE id = ?1",
            params![id.to_string()],
            |row| row_to_server(row),
        )?;
        Ok(server)
    }

    pub fn delete_channel(&self, id: &Uuid) -> Result<(), AppError> {
        let conn = self.conn.lock().expect("db mutex");
        conn.execute("DELETE FROM channels WHERE id = ?1", params![id.to_string()])?;
        Ok(())
    }

    pub fn update_channel(&self, id: &Uuid, name: &str) -> Result<Channel, AppError> {
        let conn = self.conn.lock().expect("db mutex");
        conn.execute("UPDATE channels SET name = ?1 WHERE id = ?2", params![name, id.to_string()])?;
        let channel = conn.query_row(
            "SELECT id, name, origin_server FROM channels WHERE id = ?1",
            params![id.to_string()],
            |row| row_to_channel(row),
        )?;
        Ok(channel)
    }

    pub fn list_channel_messages(&self, channel_id: Uuid) -> Result<Vec<Message>, AppError> {
        let conn = self.conn.lock().expect("db mutex");
        let mut stmt = conn.prepare(
            "SELECT id, kind, body, author_user_id, recipient_user_id, channel_id, sent_at
             FROM messages
             WHERE channel_id = ?1
             ORDER BY sent_at ASC",
        )?;
        let rows = stmt.query_map(params![channel_id.to_string()], row_to_message)?;
        let mut messages = Vec::new();
        for row in rows {
            messages.push(row?);
        }
        Ok(messages)
    }

    pub fn list_dm_messages(&self, user_id: Uuid, other_user_id: Uuid) -> Result<Vec<Message>, AppError> {
        let conn = self.conn.lock().expect("db mutex");
        let mut stmt = conn.prepare(
            "SELECT id, kind, body, author_user_id, recipient_user_id, channel_id, sent_at
             FROM messages
             WHERE (author_user_id = ?1 AND recipient_user_id = ?2)
                OR (author_user_id = ?2 AND recipient_user_id = ?1)
             ORDER BY sent_at ASC",
        )?;
        let rows = stmt.query_map(params![user_id.to_string(), other_user_id.to_string()], row_to_message)?;
        let mut messages = Vec::new();
        for row in rows {
            messages.push(row?);
        }
        Ok(messages)
    }

    pub fn create_federation_token(&self, label: &str) -> Result<FederationToken, AppError> {
        let id = Uuid::new_v4();
        let token = Uuid::new_v4().to_string();
        let created_at = time::OffsetDateTime::now_utc()
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_default();
        let conn = self.conn.lock().expect("db mutex");
        conn.execute(
            "INSERT INTO federation_tokens (id, token, label, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![id.to_string(), token, label, created_at],
        )?;
        Ok(FederationToken {
            id,
            token,
            label: label.to_string(),
            created_at,
        })
    }

    pub fn list_federation_tokens(&self) -> Result<Vec<FederationToken>, AppError> {
        let conn = self.conn.lock().expect("db mutex");
        let mut stmt = conn.prepare(
            "SELECT id, token, label, created_at FROM federation_tokens ORDER BY created_at DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(FederationToken {
                id: Uuid::parse_str(row.get::<_, String>(0)?.as_str()).map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
                })?,
                token: row.get(1)?,
                label: row.get(2)?,
                created_at: row.get(3)?,
            })
        })?;
        let mut tokens = Vec::new();
        for row in rows {
            tokens.push(row?);
        }
        Ok(tokens)
    }

    pub fn delete_federation_token(&self, id: &Uuid) -> Result<(), AppError> {
        let conn = self.conn.lock().expect("db mutex");
        conn.execute(
            "DELETE FROM federation_tokens WHERE id = ?1",
            params![id.to_string()],
        )?;
        Ok(())
    }

    pub fn is_valid_federation_token(&self, token: &str) -> Result<bool, AppError> {
        let conn = self.conn.lock().expect("db mutex");
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM federation_tokens WHERE token = ?1",
            params![token],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    pub fn get_hidden_user_ids(&self, server_id: Uuid) -> Result<Vec<String>, AppError> {
        let conn = self.conn.lock().expect("db mutex");
        let mut stmt = conn.prepare(
            "SELECT user_id FROM server_hidden_users WHERE server_id = ?1",
        )?;
        let rows = stmt.query_map(params![server_id.to_string()], |row| row.get::<_, String>(0))?;
        let mut ids = Vec::new();
        for row in rows {
            ids.push(row?);
        }
        Ok(ids)
    }

    pub fn get_hidden_channel_ids(&self, server_id: Uuid) -> Result<Vec<String>, AppError> {
        let conn = self.conn.lock().expect("db mutex");
        let mut stmt = conn.prepare(
            "SELECT channel_id FROM server_hidden_channels WHERE server_id = ?1",
        )?;
        let rows = stmt.query_map(params![server_id.to_string()], |row| row.get::<_, String>(0))?;
        let mut ids = Vec::new();
        for row in rows {
            ids.push(row?);
        }
        Ok(ids)
    }

    pub fn set_hidden_users(&self, server_id: Uuid, user_ids: &[String]) -> Result<(), AppError> {
        let conn = self.conn.lock().expect("db mutex");
        conn.execute("DELETE FROM server_hidden_users WHERE server_id = ?1", params![server_id.to_string()])?;
        for uid in user_ids {
            conn.execute(
                "INSERT INTO server_hidden_users (server_id, user_id) VALUES (?1, ?2)",
                params![server_id.to_string(), uid],
            )?;
        }
        Ok(())
    }

    pub fn set_hidden_channels(&self, server_id: Uuid, channel_ids: &[String]) -> Result<(), AppError> {
        let conn = self.conn.lock().expect("db mutex");
        conn.execute("DELETE FROM server_hidden_channels WHERE server_id = ?1", params![server_id.to_string()])?;
        for cid in channel_ids {
            conn.execute(
                "INSERT INTO server_hidden_channels (server_id, channel_id) VALUES (?1, ?2)",
                params![server_id.to_string(), cid],
            )?;
        }
        Ok(())
    }

    pub fn remove_channel_member(&self, channel_id: Uuid, user_id: Uuid) -> Result<(), AppError> {
        let conn = self.conn.lock().expect("db mutex");
        conn.execute(
            "DELETE FROM channel_members WHERE channel_id = ?1 AND user_id = ?2",
            params![channel_id.to_string(), user_id.to_string()],
        )?;
        Ok(())
    }
}

fn row_to_server(row: &rusqlite::Row) -> Result<Server, rusqlite::Error> {
    Ok(Server {
        id: Uuid::parse_str(row.get::<_, String>(0)?.as_str()).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
        })?,
        name: row.get(1)?,
        base_url: row.get(2)?,
        token: row.get(3)?,
    })
}

fn row_to_channel(row: &rusqlite::Row) -> Result<Channel, rusqlite::Error> {
    Ok(Channel {
        id: Uuid::parse_str(row.get::<_, String>(0)?.as_str()).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
        })?,
        name: row.get(1)?,
        origin_server: row.get(2)?,
    })
}

fn row_to_user(row: &rusqlite::Row) -> Result<User, rusqlite::Error> {
    let id_string: String = row.get(0)?;
    let server_id_string: Option<String> = row.get(3)?;
    Ok(User {
        id: Uuid::parse_str(id_string.as_str()).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
        })?,
        username: row.get(1)?,
        token: row.get(2)?,
        server_id: server_id_string
            .as_deref()
            .map(|value| {
                Uuid::parse_str(value).map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
                })
            })
            .transpose()?,
        is_local: row.get::<_, i32>(4)? == 1,
        display_name: row.get(5)?,
    })
}

fn row_to_message(row: &rusqlite::Row) -> Result<Message, rusqlite::Error> {
    let id_string: String = row.get(0)?;
    let kind_string: String = row.get(1)?;
    let author_string: String = row.get(3)?;
    let recipient_string: Option<String> = row.get(4)?;
    let channel_string: Option<String> = row.get(5)?;

    let kind = match kind_string.as_str() {
        "dm" => MessageKind::Dm,
        "channel" => MessageKind::Channel,
        _ => {
            return Err(rusqlite::Error::FromSqlConversionFailure(
                1,
                rusqlite::types::Type::Text,
                Box::new(std::fmt::Error),
            ))
        }
    };

    Ok(Message {
        id: Uuid::parse_str(id_string.as_str()).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
        })?,
        kind,
        body: row.get(2)?,
        author_user_id: Uuid::parse_str(author_string.as_str()).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(3, rusqlite::types::Type::Text, Box::new(e))
        })?,
        recipient_user_id: recipient_string
            .as_deref()
            .map(|value| {
                Uuid::parse_str(value).map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(4, rusqlite::types::Type::Text, Box::new(e))
                })
            })
            .transpose()?,
        channel_id: channel_string
            .as_deref()
            .map(|value| {
                Uuid::parse_str(value).map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(5, rusqlite::types::Type::Text, Box::new(e))
                })
            })
            .transpose()?,
        sent_at: row.get(6)?,
    })
}


#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn can_create_local_user() {
        let file = NamedTempFile::new().expect("tempfile");
        let store = SqliteStore::new(file.path().to_str().unwrap()).expect("store");
        store.init().expect("init");
        let user = store.create_user("alice", true, None).expect("user");
        assert_eq!(user.username, "alice");
        assert!(user.is_local);
        assert!(!user.token.is_empty());
    }
}
