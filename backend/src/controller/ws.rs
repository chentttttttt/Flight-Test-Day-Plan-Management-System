use actix::{Actor, Addr, AsyncContext, Context, Handler};
use actix_rt::spawn;
use actix_web::{web, HttpRequest, HttpResponse};
use actix_ws::{self, Message, ProtocolError, Session};
use futures::StreamExt;
use serde_json::json;

use crate::error::AppError;
use crate::middleware::auth::AuthUser;
use crate::service::ServiceContainer;
use crate::utils::ws_manager::{NotificationMessage, WsManager};

/// WebSocket Actor
pub struct ApprovalWs {
    pub user_id: i64,
    pub ws_manager: WsManager,
    session: Option<Session>,
}

impl ApprovalWs {
    pub fn new(user_id: i64, ws_manager: WsManager) -> Self {
        Self {
            user_id,
            ws_manager,
            session: None,
        }
    }

    pub fn set_session(&mut self, session: Session) {
        self.session = Some(session);
    }
}

impl Actor for ApprovalWs {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.ws_manager.add(self.user_id, ctx.address());

        if let Some(session) = &mut self.session {
            let welcome = json!({
            "type": "welcome",
            "user_id": self.user_id
        })
                .to_string();
            let _ = session.text(welcome);
        }
    }

    fn stopping(&mut self, _ctx: &mut Self::Context) -> actix::Running {
        self.ws_manager.remove(self.user_id);
        actix::Running::Stop
    }
}

impl Handler<NotificationMessage> for ApprovalWs {
    type Result = ();

    fn handle(&mut self, msg: NotificationMessage, _ctx: &mut Self::Context) {
        if let Some(session) = &mut self.session {
            let _ = session.text(msg.0);
        }
    }
}

/// WebSocket 消息循环（处理 ping/pong/close）
pub async fn run_ws_task(
    mut session: Session,
    mut stream: actix_ws::MessageStream,
    _addr: Addr<ApprovalWs>,
) {
    while let Some(Ok(msg)) = stream.next().await {
        match msg {
            Message::Ping(data) => {
                let _ = session.pong(&data);
            }
            Message::Close(_) => {
                let _ = session.close(None);
                break;
            }
            _ => {}
        }
    }
}

/// WebSocket 路由处理
pub async fn websocket_handler(
    req: HttpRequest,
    stream: web::Payload,
    auth: AuthUser,
    services: web::Data<ServiceContainer>,
) -> Result<HttpResponse, AppError> {
    let user_id = auth.id;
    let ws_manager = services.ws_manager.clone();

    let (response, session, msg_stream) = actix_ws::handle(&req, stream)
        .map_err(|e| AppError::InternalError(format!("WebSocket upgrade failed: {}", e)))?;

    let mut ws_actor = ApprovalWs::new(user_id, ws_manager);
    ws_actor.set_session(session.clone());
    let addr = ws_actor.start();

    spawn(async move {
        // 显式声明可变，确保所有权正确转移
        let mut session = session;
        let mut msg_stream = msg_stream;
        run_ws_task(session, msg_stream, addr).await;
    });

    Ok(response)
}