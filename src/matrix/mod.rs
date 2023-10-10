pub mod e2ee;
use std::{fs, path::Path, sync::OnceLock};

use anyhow::Result;
use matrix_sdk::{
    self, config::SyncSettings, room::Joined, ruma::events::room::message::RoomMessageEventContent,
    Client,
};
use tokio::runtime::Runtime;
use url::Url;

use crate::args;

pub static ROOM: OnceLock<Joined> = OnceLock::new();
pub static CLIENT: OnceLock<Client> = OnceLock::new();

pub async fn send_msg(msg: &str) -> Result<()> {
    let msg = RoomMessageEventContent::text_markdown(msg);
    ROOM.get_or_init(room_init).send(msg, None).await?;
    Ok(())
}

async fn login(homeserver_url: &str, username: &str, password: &str) -> Result<Client> {
    let homeserver_url = Url::parse(homeserver_url).expect("Couldn't parse the homeserver URL");

    let db_path = std::path::PathBuf::from(&args().data_dir).join("db");
    let mut client = Client::builder()
        .homeserver_url(&homeserver_url)
        .sled_store(&db_path, None)?
        .build()
        .await
        .map_err(|e| {
            log::error!("client build error: {}", e);
            e
        })?;

    if !client.logged_in() {
        let session_file = std::path::PathBuf::from(&args().data_dir).join("session");

        if session_file.exists()
            && restore_login(&client, &session_file).await.is_ok()
            && client.logged_in()
            && client.sync_once(SyncSettings::new()).await.is_ok()
        {
            log::info!("Restored login from session file");
        } else {
            drop(client);
            // 清理数据库
            fs::remove_dir_all(&db_path)?;
            client = Client::builder()
                .homeserver_url(&homeserver_url)
                .sled_store(db_path, None)?
                .build()
                .await
                .map_err(|e| {
                    log::error!("client build error: {}", e);
                    e
                })?;
            login_username(&client, &session_file, username, password).await?;
        };

        log::info!("Logged in as {}", username);
    }

    Ok(client)
}

async fn get_room(client: &Client, room_id: &str) -> Result<Joined> {
    client.sync_once(SyncSettings::new()).await?;
    let room = client
        .get_joined_room(room_id.try_into()?)
        .ok_or(anyhow::Error::msg("No room with that alias exists"))?;

    Ok(room)
}

async fn restore_login(client: &Client, session_file: impl AsRef<Path>) -> Result<()> {
    let session = fs::read_to_string(session_file)?;
    let session = serde_json::from_str(&session)?;
    client.restore_login(session).await.map_err(|e| {
        log::error!("restore login error: {}", e);
        e
    })?;
    Ok(())
}

async fn login_username(
    client: &Client,
    session_file: impl AsRef<Path>,
    username: &str,
    password: &str,
) -> Result<()> {
    client
        .login_username(username, password)
        .initial_device_display_name("matrix_webhook_bot")
        .send()
        .await?;
    let session = client.session();
    if let Some(session) = session {
        let session = serde_json::to_string(&session)?;
        fs::write(session_file, session)?;
    };
    Ok(())
}

pub fn room_init() -> Joined {
    std::thread::spawn(|| {
        let rt = Runtime::new().unwrap();
        rt.block_on(async move {
            let args = args();
            let client = CLIENT.get_or_init(client_init);
            get_room(client, &args.room_id).await.unwrap()
        })
    })
    .join()
    .unwrap()
}

pub fn client_init() -> Client {
    std::thread::spawn(|| {
        let rt = Runtime::new().unwrap();
        rt.block_on(async move {
            let args = args();

            login(&args.home_server_url, &args.user, &args.password)
                .await
                .unwrap()
        })
    })
    .join()
    .unwrap()
}
