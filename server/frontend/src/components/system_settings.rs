use common::*;
use crate::util::*;
use yew::format::Json;
use yew::services::fetch::{ FetchService, FetchTask, };
use yew::{ Component, ComponentLink, Html, Renderable, ShouldRender, html, Properties, };
use stdweb::web;
use super::user_message::UserMessage;
use std::net::Ipv4Addr;

pub enum Msg {
    InputIp(String),

    RequestRestart(SystemCommand),
    RequestSetIp,

    ResponseRestart(JsonResponse<()>),
    ResponseSetIp(JsonResponse<()>),
}

pub struct SystemSettings {
    user_msg: UserMessage<Self>,
    fetch_service: FetchService,
    self_link: ComponentLink<Self>,
    user_type: WebUserType,
    ip_raw: String,

    fetch_task: Option<FetchTask>,
    fetch_task_command: Option<FetchTask>,
}

impl JsonResponseHandler for SystemSettings {}

#[derive(Properties)]
pub struct SystemSettingsProps {
    #[props(required)]
    pub user_type: WebUserType,
}

impl Component for SystemSettings {
    type Message = Msg;
    type Properties = SystemSettingsProps;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let result = SystemSettings {
            user_msg: UserMessage::new(),
            fetch_service: FetchService::new(),
            self_link: link,
            user_type: props.user_type,
            ip_raw: String::new(),

            fetch_task: None,
            fetch_task_command: None,
        };
        result
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::RequestRestart(command) => {
                let confirmed = match command {
                    SystemCommand::StartNormal => web::window().confirm("Are you sure you wish to restart?"),
                    SystemCommand::RebuildDB => web::window().confirm("Are you sure you wish to rebuild the database? This action will cause the server to restart and cannot be undone."),
                };

                if confirmed {
                    self.fetch_task = post_request! (
                        self.fetch_service,
                        &system_restart_url(),
                        command,
                        self.self_link,
                        Msg::ResponseRestart
                    );
                }
            },
            Msg::InputIp(ip) => {
                self.ip_raw = ip;
            }
            Msg::RequestSetIp => {
                self.user_msg.reset();
                let ret_ip: Result<Ipv4Addr, _> = self.ip_raw.parse();

                match ret_ip {
                    Ok(ip) => {
                        self.fetch_task_command = post_request! (
                            self.fetch_service,
                            &beacon_command_url(),
                            BeaconRequest::SetIp(ip),
                            self.self_link,
                            Msg::ResponseSetIp
                        );
                    },
                    Err(e) => {
                        self.user_msg.error_messages.push(format!("failed to send setip command, reason: {}", e));
                    },
                }

            },
            Msg::ResponseRestart(response) => {
                let (meta, Json(_body)) = response.into_parts();
                if meta.status.is_success() {
                    self.user_msg.success_message = Some("successfully sent restart command".to_owned());
                } else {
                    self.user_msg.error_messages.push("failed to send restart command".to_string());
                }
            },
            Msg::ResponseSetIp(response) => {
                self.handle_response(
                    response,
                    |s, _| {
                        s.user_msg.success_message = Some("Successfully sent setip command".to_owned());
                    },
                    |s, e| {
                        s.user_msg.error_messages.push(format!("failed to send setip, reason: {}", e));
                    },
                );
            },
        }
        true
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.user_type = props.user_type;
        true
    }
}

// The front-end layout in HTML
impl Renderable<SystemSettings> for SystemSettings {
    fn view(&self) -> Html<Self> {
        let space = {" "};

        html! {
            <>
                { self.user_msg.view() }
                <div class="content-wrapper">
                    <div/>
                    <div class="boxedForm">
                        <h2>{ "System Settings" }</h2>
                        <div class="d-flex">
                            <button
                                class="btn btn-lg btn-info mr-3 my-auto",
                                onclick=|_| Msg::RequestRestart(SystemCommand::StartNormal),
                            >
                                <i class="fa fa-refresh" aria-hidden="true"></i>
                                { space }
                                { "Restart Server" }
                            </button>
                            
                            <button
                                class="btn btn-lg btn-info ml-3 my-auto",
                                onclick=|_| Msg::RequestRestart(SystemCommand::RebuildDB),
                            >
                                <i class="fa fa-power-off" aria-hidden="true"></i>
                                { space }
                                { " Reset Database" }
                            </button>
                        </div>

                        <div class="d-flex justify-content-start">
                            <button
                                class="btn btn-lg btn-secondary my-auto",
                                onclick=|_| Msg::RequestSetIp,
                            >
                                { space }
                                {"Set IP Address"}
                            </button>
                            <input
                                type="text",
                                class="fixedLength",
                                placeholder="IP address",
                                oninput=|event| Msg::InputIp(event.value),
                            />
                        </div>
                    </div>
                </div>
            </>
        }
    }
}
