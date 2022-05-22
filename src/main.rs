use std::io;
use std::net::SocketAddr;

use axum::{
    extract::{
        ConnectInfo,
        ws::{Message, WebSocket, WebSocketUpgrade},
        TypedHeader,
    },
    routing::get,
    routing::get_service,
    http::StatusCode,
    response::IntoResponse,
    Router,
};
use tower_http::{
    services::ServeDir,
    trace::{DefaultMakeSpan, TraceLayer},
};

#[tokio::main]
async fn main() {
    // Set the RUST_LOG, if it hasn't been explicitly defined
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "ws=debug,tower_http=debug")
    }
    tracing_subscriber::fmt::init();

    // build our application with some routes
    let app: _ = Router::new()
            .route("/foo", get(|| async {"Hi from /foo"}))
            .route("/ip", get(handler_ip))
            .route("/ws", get(handler_ws))
            .fallback(get_service(ServeDir::new("html")).handle_error(handle_error))
            .layer(TraceLayer::new_for_http()
                    .make_span_with(DefaultMakeSpan::default().include_headers(true)));
            //get_service(ServeDir::new(".").append_index_html_on_directories(true)).handle_error(|error: std::io::Error| async move {

        // routes are matched from bottom to top, so we have to put `nest` at the
        // top since it matches all routes
        // logging so we can see whats going on

    // run it with hyper
    tracing::debug!("listening on 8888");
    axum::Server::bind(&"0.0.0.0:8888".parse().unwrap())
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
}

async fn handle_error(_err: io::Error) -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong...")
}

async fn handler_ip(ConnectInfo(addr): ConnectInfo<SocketAddr>) -> impl IntoResponse {
    format!("Your address is {}.", addr)
}

async fn handler_ws(
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
) -> impl IntoResponse {
    if let Some(TypedHeader(user_agent)) = user_agent {
        println!("`{}` connected", user_agent.as_str());
    }

    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    if let Some(msg) = socket.recv().await {
        if let Ok(msg) = msg {
            println!("Client says: {:?}", msg);
        } else {
            println!("client disconnected");
            return;
        }
    }

    loop {
        if socket
            .send(Message::Text(String::from("Hi!")))
            .await
            .is_err()
        {
            println!("client disconnected");
            return;
        }
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    }
}
