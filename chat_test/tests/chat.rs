use std::{net::SocketAddr, time::Duration};

use anyhow::Result;

use chat_core::{Chat, ChatType, Message};
use chat_server::{AppState, CreateChat, CreateMessage, SigninUser};
use futures::StreamExt;
use reqwest::{
    multipart::{Form, Part},
    StatusCode,
};
use reqwest_eventsource::{Event, EventSource};
use serde::Deserialize;
use tokio::net::TcpListener;

#[derive(Debug, Deserialize)]
struct AuthToken {
    token: String,
}

struct ChatServer {
    addr: SocketAddr,
    token: String,
    client: reqwest::Client,
}

struct NotifyServer;

const WILD_ADDR: &str = "0.0.0.0:0";

#[tokio::test]
async fn chat_server_should_work() -> Result<()> {
    let (tdb, state) = chat_server::AppState::new_for_test().await?;
    let db_url = tdb.url();
    let chat_server = ChatServer::new(state).await?;
    NotifyServer::new(&db_url, &chat_server.token).await?;
    let chat = chat_server.create_chat().await?;
    let _msg = chat_server.create_message(chat.id as _).await?;
    tokio::time::sleep(Duration::from_secs(1)).await;
    Ok(())
}

impl NotifyServer {
    pub async fn new(db_url: &str, token: &str) -> Result<Self> {
        let mut config = notify_server::AppConfig::load()?;
        config.server.db_url = db_url.to_string();
        let state = notify_server::AppState::new(config);
        let app = notify_server::get_router(state).await?;
        let listener = TcpListener::bind(WILD_ADDR).await?;
        let addr = listener.local_addr()?;
        tokio::spawn(async move {
            axum::serve(listener, app.into_make_service())
                .await
                .unwrap();
        });

        let mut es = EventSource::get(format!("http://{}/events?access_token={}", addr, token));
        tokio::spawn(async move {
            while let Some(event) = es.next().await {
                match event {
                    Ok(Event::Open) => {
                        println!("Connection Open!");
                    }
                    Ok(Event::Message(msg)) => {
                        println!("Message: {:?}", msg);
                        match msg.event.as_str() {
                            "NewChat" => {
                                let chat: Chat = serde_json::from_str(&msg.data).unwrap();
                                assert_eq!(chat.name.as_ref().unwrap(), "test");
                                assert_eq!(chat.members, vec![1, 2]);
                                assert_eq!(chat.r#type, ChatType::PrivateChannel);
                            }
                            "NewMessage" => {
                                let msg: Message = serde_json::from_str(&msg.data).unwrap();
                                assert_eq!(msg.content, "hello world");
                                assert_eq!(msg.files.len(), 1);
                                assert_eq!(msg.sender_id, 1);
                            }
                            _ => panic!("unexpected event: {:?}", msg),
                        }
                    }
                    Err(e) => {
                        println!("Error: {}", e);
                        es.close()
                    }
                }
            }
        });
        Ok(Self)
    }
}

impl ChatServer {
    async fn new(state: AppState) -> Result<Self> {
        let app = chat_server::get_router(state).await?;
        let listener = TcpListener::bind(WILD_ADDR).await?;
        let addr = listener.local_addr()?;
        tokio::spawn(async move {
            axum::serve(listener, app.into_make_service())
                .await
                .unwrap();
        });

        let mut server = Self {
            addr,
            token: "".to_string(),
            client: reqwest::Client::new(),
        };
        server.signin().await?;

        Ok(server)
    }

    async fn signin(&mut self) -> Result<()> {
        let body = serde_json::to_string(&SigninUser {
            email: "startdusk@acme.org".to_string(),
            password: "password".to_string(),
        })?;

        let res = self
            .client
            .post(&format!("http://{}/api/signin", self.addr))
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await?;
        assert_eq!(res.status(), StatusCode::OK);
        let ret: AuthToken = res.json().await?;
        self.token = ret.token;
        Ok(())
    }

    async fn create_chat(&self) -> Result<Chat> {
        let body = serde_json::to_string(&CreateChat {
            name: Some("test".to_string()),
            members: vec![1, 2],
            public: false,
        })?;

        let res = self
            .client
            .post(&format!("http://{}/api/chats", self.addr))
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await?;
        assert_eq!(res.status(), StatusCode::CREATED);
        let chat: Chat = res.json().await?;
        assert_eq!(chat.name.as_ref().unwrap(), "test");
        assert_eq!(chat.members, vec![1, 2]);
        assert_eq!(chat.r#type, ChatType::PrivateChannel);
        Ok(chat)
    }

    async fn create_message(&self, chat_id: u64) -> Result<Message> {
        // upload file
        let data = include_bytes!("../Cargo.toml");
        let files = Part::bytes(data)
            .file_name("Cargo.toml")
            .mime_str("text/plain")?;
        let form = Form::new().part("file", files);

        let res = self
            .client
            .post(&format!("http://{}/api/upload", self.addr))
            .header("Authorization", format!("Bearer {}", self.token))
            .multipart(form)
            .send()
            .await?;
        assert_eq!(res.status(), StatusCode::OK);
        let files: Vec<String> = res.json().await?;

        let body = serde_json::to_string(&CreateMessage {
            content: "hello world".to_string(),
            files: files.clone(),
        })?;

        let res = self
            .client
            .post(&format!("http://{}/api/chats/{}", self.addr, chat_id))
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await?;

        assert_eq!(res.status(), StatusCode::CREATED);
        let msg: Message = res.json().await?;
        assert_eq!(msg.content, "hello world");
        assert_eq!(msg.files, files);
        assert_eq!(msg.sender_id, 1);
        assert_eq!(msg.chat_id, chat_id as i64);
        Ok(msg)
    }
}
