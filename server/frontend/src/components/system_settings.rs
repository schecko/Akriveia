use common::*;
use crate::util::*;
use yew::format::Json;
use yew::services::fetch::{ FetchService, FetchTask, };
use yew::{ Component, ComponentLink, Html, Renderable, ShouldRender, html, Properties, };
use stdweb::web;
use super::user_message::UserMessage;

pub enum Msg {
    RequestRestart(SystemCommand),
    ResponseRestart(JsonResponse<()>),
}

pub struct SystemSettings {
    user_msg: UserMessage<Self>,
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
            user_msg: UserMessage::new(),
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
                    self.user_msg.success_message = Some("successfully sent restart command".to_owned());
                } else {
                    self.user_msg.error_messages.push("failed to send restart command".to_string());
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
        html! {
            <>
                { self.user_msg.view() }
                <div/>
                <div class="boxedForm">
                    <h2>{ "System Settings" }</h2>

                    <div class="d-flex">
                        <button
                            class="btn btn-lg btn-info mr-3 my-auto",
                            onclick=|_| Msg::RequestRestart(SystemCommand::StartNormal),
                        >
                            <i class="fa fa-refresh" aria-hidden="true"></i>
                            { " Restart Server" }
                        </button>
                        
                        <button
                            class="btn btn-lg btn-info ml-3 my-auto",
                            onclick=|_| Msg::RequestRestart(SystemCommand::RebuildDB),
                        >
                            <i class="fa fa-recycle fa-fw" aria-hidden="true"></i>
                            { " Reset Database" }
                        </button>
                    </div>

                    <div class="d-flex justify-content-start">
                        <button
                            class="btn btn-lg btn-secondary my-auto",
                        ><i class="fa fa-laptop" aria-hidden="true"></i>
                            {" Set IP Address"}
                        </button>
                        <input
                            type="text",
                            class="fixedLength",
                            placeholder="IP address",
                        />
                    </div>
                </div>
            </>
        }
    }
}
