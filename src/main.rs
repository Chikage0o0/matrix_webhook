mod matrix;

use axum::{
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use clap::Parser;
use std::{
    net::{IpAddr, SocketAddr},
    sync::OnceLock,
};

static ARGS: OnceLock<Args> = OnceLock::new();
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Home Server URL
    #[arg(long, env = "HOME_SERVER_URL")]
    home_server_url: String,

    /// 发送到的房间ID
    #[arg(long, env = "ROOM_ID")]
    room_id: String,

    /// 机器人用户名
    #[arg(long, env = "USER")]
    user: String,

    /// 机器人密码
    #[arg(long, env = "PASSWORD")]
    password: String,

    /// web服务端口
    /// 默认值: 3000
    #[arg(short, long, default_value = "3000", env = "PORT")]
    port: u16,

    /// 日志等级
    /// 默认值: warn
    /// 可选值: trace, debug, info, warn, error
    #[arg(short, long, default_value = "warn")]
    log_level: log::LevelFilter,

    /// Token
    /// 用于接口认证
    #[arg(long, default_value = None, env = "TOKEN")]
    token: Option<String>,

    /// listen address
    /// 默认值: 127.0.0.1
    #[arg(long, default_value = "127.0.0.1", env = "LISTEN")]
    listen: String,
}

#[tokio::main]
async fn main() {
    let args = args();
    env_logger::Builder::new()
        .filter_level(args.log_level)
        .init();
    log::info!("args: {:?}", args);
    log::info!("start web server");
    let mut app: Router = Router::new()
        .route("/ping", get(ping))
        .route("/send", post(send))
        .fallback(not_found);

    if let Some(token) = &args.token {
        app = app.layer(tower_http::validate_request::ValidateRequestHeaderLayer::bearer(token));
    }

    log::info!("listen on {}", args.port);
    let ip: IpAddr = args.listen.parse().unwrap();
    let addr = SocketAddr::from((ip, args.port));
    let server = axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(async move {
            tokio::signal::ctrl_c()
                .await
                .expect("failed to install CTRL+C handler")
        });

    server.await.unwrap();
}

pub(crate) fn args() -> &'static Args {
    ARGS.get_or_init(Args::parse)
}

async fn ping() -> &'static str {
    "pong"
}

#[derive(Debug, serde::Deserialize)]
struct Msg {
    msg: String,
}
async fn send(Json(msg): Json<Msg>) -> StatusCode {
    match matrix::send_msg(&msg.msg).await {
        Ok(_) => {
            log::info!("send msg success");
            StatusCode::OK
        }
        Err(e) => {
            log::error!("send msg failed: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

async fn not_found() -> StatusCode {
    StatusCode::NOT_FOUND
}
