use federated_server::{api, config::Config, storage::SqliteStore};
use std::net::SocketAddr;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let config = Config::from_env();
    let store = SqliteStore::new(&config.database_path)?;
    store.init()?;
    // Disabled: auto-seeding of federation peers. Use Admin UI to manually add servers.
    // store.seed_initial_data(&config.server_name)?;

    // Ensure this server is present in the servers table and update its
    // base_url/token if they differ from the current environment. This
    // helps reconcile tokens when DB was reused across runs or when
    // docker-compose seeds differ from the runtime env.
    let self_server = store.ensure_server(&config.server_name, &config.base_url, &config.server_token)?;
    if self_server.token != config.server_token || self_server.base_url != config.base_url {
        let _ = store.update_server(&self_server.id, &config.server_name, &config.base_url, &config.server_token)?;
        tracing::info!(target: "startup", "Updated local server record '{}' with current base_url/token (masked)", config.server_name);
    }

    // Ensure admin user exists with the configured password
    ensure_admin_user(&store, &config);

    // Log discovered/seeded federation peers at startup (mask tokens for safety).
    if let Ok(servers) = store.list_servers() {
        for s in servers {
            let masked = if s.token.len() > 4 {
                format!("***{}", &s.token[s.token.len() - 4..])
            } else {
                s.token.clone()
            };
            tracing::info!(target: "startup", "federation peer: {} -> {} token: {}", s.name, s.base_url, masked);
        }
    }

    let app = api::router(store, config.clone());

    let addr: SocketAddr = "0.0.0.0:8080".parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("server running on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;

    Ok(())
}

fn ensure_admin_user(store: &SqliteStore, config: &Config) {
    let password_hash = match bcrypt::hash(&config.admin_password, bcrypt::DEFAULT_COST) {
        Ok(h) => h,
        Err(e) => {
            tracing::error!(target: "startup", "Failed to hash admin password: {}", e);
            return;
        }
    };

    match store.get_user_by_name_and_server(&config.admin_username, None) {
        Ok(Some(user)) => {
            // Update password to match config on every startup
            if let Err(e) = store.set_user_password(&user.id, &password_hash) {
                tracing::error!(target: "startup", "Failed to update admin user password: {}", e);
            } else {
                tracing::info!(target: "startup", "Admin user '{}' password synced from environment", config.admin_username);
            }
        }
        Ok(None) => {
            // Create the admin user
            match store.create_user_with_password(
                &config.admin_username,
                true,
                None,
                Some(&password_hash),
            ) {
                Ok(user) => {
                    tracing::info!(target: "startup", "Created admin user '{}' (id: {})", config.admin_username, user.id);
                }
                Err(e) => {
                    tracing::error!(target: "startup", "Failed to create admin user: {}", e);
                }
            }
        }
        Err(e) => {
            tracing::error!(target: "startup", "Failed to look up admin user: {}", e);
        }
    }
}
