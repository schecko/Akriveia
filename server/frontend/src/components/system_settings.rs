use common::*;
use crate::util::{ self, WebUserType, };
use yew::format::Json;
use yew::services::fetch::{ FetchService, FetchTask, };
use yew::{ Component, ComponentLink, Html, Renderable, ShouldRender, html, Properties, };
use stdweb::web;

pub enum Msg {
    RequestRestart(SystemCommand),
    ResponseRestart(util::Response<()>),
}

struct Data {
    pub error_messages: Vec<String>,
    pub success_message: Option<String>,
}

impl Data {
    fn new() -> Data {
        Data {
            error_messages: Vec::new(),
            success_message: None,
        }
    }
}

pub struct SystemSettings {
    data: Data,
    fetch_service: FetchService,
    fetch_task: Option<FetchTask>,
    self_link: ComponentLink<Self>,
    user_type: WebUserType,
}

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
            data: Data::new(),
            fetch_service: FetchService::new(),
            fetch_task: None,
            self_link: link,
            user_type: props.user_type,
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
            Msg::ResponseRestart(response) => {
                let (meta, Json(_body)) = response.into_parts();
                if meta.status.is_success() {
                    self.data.success_message = Some("successfully sent restart command".to_owned());
                } else {
                    self.data.error_messages.push("failed to send restart command".to_string());
                }
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
        let mut errors = self.data.error_messages.iter().map(|msg| {
            html! {
                <p class="alert alert-error">{msg}</p>
            }
        });

        html! {
            <>
                {
                    match &self.data.success_message {
                        Some(msg) => { format!("Success: {}", msg) },
                        None => { String::new() },
                    }
                }
                { if self.data.error_messages.len() > 0 { "Failure: " } else { "" } }
                { for errors }
                <div/>
                <table>
                    <tr>
                        <td>
                            <button
                                onclick=|_| Msg::RequestRestart(SystemCommand::StartNormal),
                            >
                                { "Restart Server" }
                            </button>
                        </td>
                    </tr>
                    <tr>
                        <td>
                            <button
                                onclick=|_| Msg::RequestRestart(SystemCommand::RebuildDB),
                            >
                                { "Reset Database" }
                            </button>
                        </td>
                    </tr>
                </table>
            </>
        }
    }
}
