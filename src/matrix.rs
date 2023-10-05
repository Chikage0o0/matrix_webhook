use std::sync::OnceLock;

use tokio::runtime::Runtime;
use url::Url;

use anyhow::Result;
use matrix_sdk::{
    config::SyncSettings, room::Joined, ruma::events::room::message::RoomMessageEventContent,
    Client,
};

use crate::args;

static ROOM: OnceLock<Joined> = OnceLock::new();

pub async fn send_msg(msg: &str) -> Result<()> {
    let msg = RoomMessageEventContent::text_markdown(msg);
    ROOM.get_or_init(init).send(msg, None).await?;
    Ok(())
}

async fn login(
    homeserver_url: &str,
    username: &str,
    password: &str,
    room_id: &str,
) -> Result<Joined> {
    let homeserver_url = Url::parse(homeserver_url).expect("Couldn't parse the homeserver URL");
    let client = Client::new(homeserver_url).await.unwrap();

    client
        .login_username(username, password)
        .initial_device_display_name("rust-sdk")
        .send()
        .await?;

    client.sync_once(SyncSettings::new()).await?;

    let room = client
        .get_joined_room(room_id.try_into()?)
        .ok_or(anyhow::Error::msg("No room with that alias exists"))?;

    Ok(room)
}

fn init() -> Joined {
    std::thread::spawn(|| {
        let rt = Runtime::new().unwrap();
        rt.block_on(async move {
            let args = args();

            login(
                &args.home_server_url,
                &args.user,
                &args.password,
                &args.room_id,
            )
            .await
            .unwrap()
        })
    })
    .join()
    .unwrap()
}
