use common::*;
use crate::util::{ self, WebUserType, };
use yew::services::fetch::{ FetchService, FetchTask, StatusCode, };
use yew::prelude::*;
use super::root;

pub enum State {
    Switch,
    Form,
}

pub enum Msg {
    ChangeRootPage(root::Page),

    ChangeState(State),
    InputName(String),
    InputPassword(String),

    RequestLogin,
    RequestLoginAnon,
    RequestLogout,

    ResponseLogin(util::Response<()>),
    ResponseLogout(util::Response<()>),
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
    state: State,
    change_page: Callback<root::Page>,
    change_user_type: Callback<WebUserType>,
    data: Data,
    fetch_service: FetchService,
    fetch_task: Option<FetchTask>,
    self_link: ComponentLink<Self>,
}

#[derive(Properties)]
pub struct LoginProps {
    #[props(required)]
    pub change_page: Callback<root::Page>,
    #[props(required)]
    pub change_user_type: Callback<WebUserType>,
    pub logout: bool,
}

impl Component for Login {
    type Message = Msg;
    type Properties = LoginProps;

    fn create(props: Self::Properties, mut link: ComponentLink<Self>) -> Self {
        if props.logout {
            link.send_self(Msg::RequestLogout);
        }
        let result = Login {
            state: State::Switch,
            change_page: props.change_page,
            change_user_type: props.change_user_type,
            data: Data::new(),
            fetch_service: FetchService::new(),
            fetch_task: None,
            self_link: link,
        };
        result
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::ChangeRootPage(page) => {
                self.change_page.emit(page);
            }
            Msg::ChangeState(state) => {
                self.state = state;
            }
            Msg::InputName(name) => {
                self.data.login.name = name;
            },
            Msg::InputPassword(pw) => {
                self.data.login.pw = pw;
            },
            Msg::RequestLoginAnon => {
                self.data.error_messages = Vec::new();
                self.data.success_message = None;
                let mut info = LoginInfo::new();
                info.name = String::from("responder");
                self.fetch_task = post_request!(
                    self.fetch_service,
                    &session_login_url(),
                    info,
                    self.self_link,
                    Msg::ResponseLogin
                );
                self.change_user_type.emit(WebUserType::Responder);
            },
            Msg::RequestLogin => {
                self.data.error_messages = Vec::new();
                self.data.success_message = None;
                self.fetch_task = post_request!(
                    self.fetch_service,
                    &session_login_url(),
                    self.data.login,
                    self.self_link,
                    Msg::ResponseLogin
                );
                self.data.login.reset_pw(); // ensure the password is deleted asap
                self.change_user_type.emit(WebUserType::Admin);
            },
            Msg::RequestLogout => {
                self.data.error_messages = Vec::new();
                self.data.success_message = None;
                self.fetch_task = post_request!(
                    self.fetch_service,
                    &session_logout_url(),
                    (),
                    self.self_link,
                    Msg::ResponseLogout
                );
                self.data.login.reset_pw(); // ensure the password is deleted asap
            },
            Msg::ResponseLogin(response) => {
                let (meta, _body) = response.into_parts();
                match meta.status {
                    StatusCode::OK => {
                        self.data.success_message = Some("Successfully logged in.".to_string());
                        self.self_link.send_self(Msg::ChangeRootPage(root::Page::MapView(None)));
                    },
                    StatusCode::UNAUTHORIZED => {
                        self.data.error_messages.push("Failed to login, username or password is incorrect.".to_string());
                    },
                    _ => {
                        self.data.error_messages.push("Failed to loginerror.".to_string());
                    }
                }
            },
            Msg::ResponseLogout(response) => {
                let (meta, _body) = response.into_parts();
                match meta.status {
                    StatusCode::OK | StatusCode::UNAUTHORIZED => {
                        self.data.success_message = Some("Successfully logged out.".to_string());
                    },
                    _ => {
                        self.data.error_messages.push("Failed to logout.".to_string());
                    }
                }
            },
        }
        true
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        if props.logout {
            self.self_link.send_self(Msg::RequestLogout);
        }
        true
    }
}

impl Login {
    fn render_switch(&self) -> Html<Self> {
        html! {
            <>
                //TODO Make a more intuintive layout for selecting admin or First Responder
                <h1>{"Select Login"}</h1>
                    <button
                        type="button",
                        class="btn btn-secondary btn-lg",
                        onclick=|_| Msg::ChangeState(State::Form),
                    >
                        { "Admin" }
                    </button>
                    
                    <button
                        type="button",
                        class="btn btn-primary btn-lg",
                        onclick=|_| Msg::RequestLoginAnon,
                    >
                        { "First Responder" }
                    </button>
            </>
        }
    }

    fn render_form(&self) -> Html<Self> {
        html! {
            <>
                <div class="wrapper fadeInDown">
                    <div id = "formContent">
                        
                        <div class="fadeIn first">
                            <img src="/images/company_name.PNG" id="company_name" width="480" height="270"/>
                        </div>

                        <div class="justify-content-center">
                            <input
                                type="text",
                                id="login",
                                name="login",
                                class="fadeIn first",
                                placeholder="Username",
                                value=&self.data.login.name,
                                oninput=|e| Msg::InputName(e.value),
                            />
                            <input
                                type="password",
                                id="password",
                                name="login",
                                class="fadeIn first",
                                placeholder="Password",
                                value=&self.data.login.pw,
                                oninput=|e| Msg::InputPassword(e.value),
                            />
                            <input
                                type="submit",
                                class="fadein first",
                                value="Login",
                                onclick=|_| Msg::RequestLogin,
                            />
                        </div>                           
                    </div>
                </div>
            </>
        }
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
                {
                    match &self.data.success_message {
                        Some(msg) => { format!("Success: {}", msg) },
                        None => { "".to_string() },
                    }
                }
                { if self.data.error_messages.len() > 0 { "Failure: " } else { "" } }
                { for errors }
                <div/>
                {
                    match self.state {
                        State::Switch => self.render_switch(),
                        State::Form => self.render_form(),
                    }
                }
            </>
        }
    }
}
