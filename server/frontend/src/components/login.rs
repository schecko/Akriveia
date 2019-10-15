use common::*;
use crate::util;
use yew::services::fetch::{ FetchService, FetchTask, StatusCode, };
use yew::prelude::*;
use yew::services::storage:: { StorageService, Area, };

pub enum Msg {
    InputName(String),
    InputPassword(String),

    RequestLogin,

    ResponseLogin(util::Response<()>),
}

// keep all of the transient data together, since its not easy to create
// a "new" method for a component.
struct Data {
    pub login: LoginInfo,
    pub error_messages: Vec<String>,
    pub success_message: Option<String>,
}

impl Data {
    fn new() -> Data {
        Data {
            login: LoginInfo::new(),
            error_messages: Vec::new(),
            success_message: None,
        }
    }
}

pub struct Login {
    data: Data,
    fetch_service: FetchService,
    fetch_task: Option<FetchTask>,
    self_link: ComponentLink<Self>,
}

#[derive(Properties)]
pub struct LoginProps { }

impl Component for Login {
    type Message = Msg;
    type Properties = LoginProps;

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let result = Login {
            data: Data::new(),
            fetch_service: FetchService::new(),
            fetch_task: None,
            self_link: link,
        };
        result
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::InputName(name) => {
                self.data.login.name = name;
            },
            Msg::InputPassword(pw) => {
                self.data.login.pw = pw;
            },
            Msg::RequestLogin => {
                self.data.error_messages = Vec::new();
                self.data.success_message = None;
                self.fetch_task = post_request!(
                    self.fetch_service,
                    &login_url(),
                    self.data.login,
                    self.self_link,
                    Msg::ResponseLogin
                );
                self.data.login.reset_pw(); // ensure the password is deleted asap
            },
            Msg::ResponseLogin(response) => {
                let (meta, _body) = response.into_parts();
                match meta.status {
                    StatusCode::OK => {
                        self.data.success_message = Some("Successfully logged in.".to_string());
                    },
                    StatusCode::UNAUTHORIZED => {
                        self.data.error_messages.push("Failed to login, username or password is incorrect.".to_string());
                    },
                    _ => {
                        self.data.error_messages.push("Failed to login, internal server error.".to_string());
                    }
                }
            },
        }
        true
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        true
    }
}

impl Renderable<Login> for Login {
    fn view(&self) -> Html<Self> {
        let mut errors = self.data.error_messages.iter().cloned().map(|msg| {
            html! {
                <p>{msg}</p>
            }
        });

        html! {
            <>
                <p>{ "Login" }</p>
                {
                    match &self.data.success_message {
                        Some(msg) => { format!("Success: {}", msg) },
                        None => { "".to_string() },
                    }
                }
                { if self.data.error_messages.len() > 0 { "Failure: " } else { "" } }
                { for errors }
                <div/>
                <table>
                    <tr>
                        <td>{ "Name: " }</td>
                        <td>
                            <input
                                type="text",
                                value=&self.data.login.name,
                                oninput=|e| Msg::InputName(e.value),
                            />
                        </td>
                    </tr>
                    <tr>
                        <td>{ "Password" }</td>
                        <input
                            type="password",
                            value=&self.data.login.pw,
                            oninput=|e| Msg::InputPassword(e.value),
                        />
                    </tr>
                </table>
                <button onclick=|_| Msg::RequestLogin,>{ "Login" }</button>
            </>
        }
    }
}
