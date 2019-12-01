

use common::*;
use crate::util::*;
use super::beacon_addupdate::BeaconAddUpdate;
use super::beacon_list::BeaconList;
use super::diagnostics::Diagnostics;
use super::emergency_buttons::EmergencyButtons;
use super::login::{ self, Login, };
use super::map_addupdate::MapAddUpdate;
use super::map_list::MapList;
use super::map_view::MapViewComponent;
use super::status::{ self, Status, };
use super::system_settings::SystemSettings;
use super::user_addupdate::UserAddUpdate;
use super::user_list::UserList;
use yew::prelude::*;
use yew::services::fetch::{ FetchService, FetchTask, };


#[derive(PartialEq)]
pub enum Page {
    BeaconAddUpdate(Option<i32>),
    BeaconList,
    Diagnostics,
    Login(login::AutoAction),
    MapAddUpdate(Option<i32>),
    MapList,
    MapView(Option<i32>),
    Status(status::PageState),
    SystemSettings,
    UserAddUpdate(Option<i32>),
    UserList,
}

pub struct RootComponent {
    user_type: WebUserType,
    current_page: Page,
    emergency: bool,
    fetch_service: FetchService,
    fetch_task: Option<FetchTask>,
    link: ComponentLink<RootComponent>,
}

impl JsonResponseHandler for RootComponent {}

pub enum Msg {
    // page changes
    ChangePage(Page),
    ChangeWebUserType(WebUserType),

    // requests
    RequestPostEmergency(bool),
    RequestGetEmergency,

    // responses
    ResponsePostEmergency(JsonResponse<bool>),
    ResponseGetEmergency(JsonResponse<bool>),
}

impl Component for RootComponent {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, mut link: ComponentLink<Self>) -> Self {
        link.send_self(Msg::RequestGetEmergency);
        let root = RootComponent {
            user_type: WebUserType::Responder,
            current_page: Page::Login(login::AutoAction::Login),
            emergency: false,
            fetch_service: FetchService::new(),
            fetch_task: None,
            link: link,
        };
        root
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::ChangePage(page) => {
                self.current_page = page;
            },
            Msg::ChangeWebUserType(user_type) => {
                self.user_type = user_type;
            },

            // requests
            Msg::RequestPostEmergency(is_emergency) => {
                self.fetch_task = post_request!(
                    self.fetch_service,
                    &system_emergency_url(),
                    SystemCommandResponse::new(is_emergency),
                    self.link,
                    Msg::ResponsePostEmergency
                );
            },
            Msg::RequestGetEmergency => {
                self.fetch_task = get_request!(
                    self.fetch_service,
                    &system_emergency_url(),
                    self.link,
                    Msg::ResponseGetEmergency
                );
            },
            // responses
            Msg::ResponsePostEmergency(response) => {
                self.handle_response(
                    response,
                    |s, resp| {
                        s.emergency = resp;
                    },
                    |_s, e| {
                        Log!("response - failed to post start emergency, {}", e);
                    },
                );
            },
            Msg::ResponseGetEmergency(response) => {
                self.handle_response(
                    response,
                    |s, resp| {
                        s.emergency = resp;
                    },
                    |_s, e| {
                        Log!("response - failed to request emergency status, {}", e);
                    },
                );
            },
        }
        true
    }
}

impl Renderable<RootComponent> for RootComponent {
    fn view(&self) -> Html<Self> {
        match self.current_page {
            Page::Diagnostics => {
                html! {
                    <div class="page-content-wrapper">
                        { self.navigation() }
                        <div class="d">
                            <EmergencyButtons
                                is_emergency={self.emergency},
                                on_emergency=|_| Msg::RequestPostEmergency(true),
                                on_end_emergency=|_| Msg::RequestPostEmergency(false),
                            />
                            <Diagnostics
                                emergency={self.emergency}
                            />
                        </div>
                    </div>
                }
            },
            Page::Status(state) => {
                html! {
                    <div class="page-content-wrapper">
                        { self.navigation() }
                        <div class="d">
                            <EmergencyButtons
                                is_emergency={self.emergency},
                                on_emergency=|_| Msg::RequestPostEmergency(true),
                                on_end_emergency=|_| Msg::RequestPostEmergency(false),
                            />
                            <Status
                                change_page=|page| Msg::ChangePage(page),
                                state=state,
                                user_type=self.user_type,
                            />
                        </div>
                    </div>
                }
            },
            Page::Login(auto_action) => {
                html! {
                    <div class="container">
                        <Login
                            change_page=|page| Msg::ChangePage(page),
                            change_user_type=|user_type| Msg::ChangeWebUserType(user_type),
                            auto_action=auto_action,
                        />
                    </div>
                }
            },
            Page::MapView(opt_id) => {
                html! {
                    <div class="page-content-wrapper">
                        { self.navigation() }
                        <div class="container">
                            <EmergencyButtons
                                is_emergency={self.emergency},
                                on_emergency=|_| Msg::RequestPostEmergency(true),
                                on_end_emergency=|_| Msg::RequestPostEmergency(false),
                            />
                            <MapViewComponent
                                emergency={self.emergency},
                                opt_id=opt_id,
                                user_type=self.user_type,
                            />
                        </div>
                    </div>
                }
            },
            Page::BeaconList => {
               html! {
                    <div class="page-content-wrapper">
                        { self.navigation() }
                        <div class="container">
                            <BeaconList
                                change_page=|page| Msg::ChangePage(page),
                            />
                        </div>
                    </div>
                }
            },
            Page::BeaconAddUpdate(id) => {
               html! {
                    <div class="page-content-wrapper">
                        { self.navigation() }
                        <div class="container">
                            <BeaconAddUpdate
                                id=id,
                                user_type=self.user_type,
                                change_page=|page| Msg::ChangePage(page),
                            />
                        </div>
                    </div>
                }
            },
            Page::UserList => {
                html! {
                    <div class="page-content-wrapper">
                        { self.navigation() }
                        <div class="container">
                            <UserList
                                change_page=|page| Msg::ChangePage(page),
                            />
                        </div>
                    </div>
                }
            },
            Page::UserAddUpdate(id) => {
                html! {
                    <div class="page-content-wrapper">
                        { self.navigation() }
                        <div class="container">
                            <UserAddUpdate
                                id=id,
                                user_type=self.user_type,
                                change_page=|page| Msg::ChangePage(page),
                            />
                        </div>
                    </div>
                }
            },
            Page::MapList => {
               html! {
                    <div class="page-content-wrapper">
                        { self.navigation() }
                        <div class="container">
                            <MapList
                                change_page=|page| Msg::ChangePage(page),
                            />
                        </div>
                    </div>
                }
            },
            Page::MapAddUpdate(opt_id) => {
               html! {
                    <div class="page-content-wrapper">
                        { self.navigation() }
                        <div class="container">
                            <MapAddUpdate
                                opt_id=opt_id,
                                user_type=self.user_type,
                                change_page=|page| Msg::ChangePage(page),
                            />
                        </div>
                    </div>
                }
            },
            Page::SystemSettings => {
                html! {
                    <div class="page-content-wrapper">
                        { self.navigation() }
                        <div class="container">
                            <SystemSettings
                                user_type=self.user_type,
                            />
                        </div>
                    </div>
                }
            },
        }
    }
}

impl RootComponent {
    fn navigation(&self) -> Html<Self> {
        let view_map = html! {
            <>
                <a
                    class = match self.current_page {
                        Page::MapView {..} => {"nav-link navBarText active"},
                        _ => {"nav-link navBarText"},
                    }
                    onclick=|_| Msg::ChangePage(Page::MapView(None)),
                    disabled={
                        match self.current_page {
                            Page::MapView { .. } => true,
                            _ => false,
                        }
                    },
                >
                    { "View Map" }
                </a>
            </>
        };

        let show_status = html! {
            <>
                <a
                    class = match self.current_page {
                        Page::Status {..} => {"nav-link navBarText active"},
                        _ => {"nav-link navBarText"},
                    }
                    id="navbarDropdown",
                    role="button",
                    onclick=|_| Msg::ChangePage(Page::Status(status::PageState::UserStatus)),
                >
                    { "Status" }
                </a>
                <div class="dropdown-content">
                    <a
                        class="dropdown-item navBarText",
                        onclick=|_| Msg::ChangePage(Page::Status(status::PageState::UserStatus)),
                        disabled={
                            match self.current_page {
                                Page::Status(status::PageState::UserStatus) => true,
                                _ => false,
                            }
                        },
                    >
                        { "User Status" }
                    </a>
                    <a
                        class="dropdown-item navBarText",
                        onclick=|_| Msg::ChangePage(Page::Status(status::PageState::BeaconStatus)),
                        disabled={
                            match self.current_page {
                                Page::Status(status::PageState::BeaconStatus) => true,
                                _ => false,
                            }
                        },
                    >
                        { "Beacon Status" }
                    </a>
                </div>
            </>
        };

        let select_user = match self.user_type {
            WebUserType::Admin => html! {
                <>
                    <a
                        class = match self.current_page {
                            Page::UserList => {"nav-link dropdown navBarText active"},
                            Page::UserAddUpdate{..} => {"nav-link dropdown navBarText active"},
                            _ => {"nav-link dropdown navBarText"},
                        }
                        role="button",
                        data-toggle="dropdown",
                        onclick=|_| Msg::ChangePage(Page::UserList)
                    >
                            { "User" }
                    </a>
                    <div class="dropdown-content">
                        <a
                            class="dropdown-item navBarText",
                            onclick=|_| Msg::ChangePage(Page::UserList),
                            disabled={self.current_page == Page::UserList},>
                                { "User List" }
                        </a>
                        <a
                            class="dropdown-item navBarText",
                            onclick=|_| Msg::ChangePage(Page::UserAddUpdate(None)),
                            disabled={
                                match self.current_page {
                                    // match ignoring the fields
                                    Page::UserAddUpdate {..} => true,
                                    _=> false,
                                }
                            },
                        >
                            { "Add User" }
                        </a>
                    </div>
                </>
            },
            WebUserType::Responder => html! { },
        };

        let select_beacon = match self.user_type {
            WebUserType::Admin => html! {
                <>
                    <a
                        class = match self.current_page {
                            Page::BeaconList => {"nav-link dropdown navBarText active"},
                            Page::BeaconAddUpdate{..} => {"nav-link dropdown navBarText active"},
                            _ => {"nav-link dropdown navBarText"},
                        }
                        role="button",
                        data-toggle="dropdown",
                        onclick=|_| Msg::ChangePage(Page::BeaconList),
                    >
                        { "Beacons" }
                    </a>
                    <div class="dropdown-content">
                            <a
                                class="dropdown-item navBarText",
                                onclick=|_| Msg::ChangePage(Page::BeaconList),
                                disabled={self.current_page == Page::BeaconList},
                            >
                                { "Beacon List" }
                            </a>
                            <a
                                class="dropdown-item navBarText",
                                onclick=|_| Msg::ChangePage(Page::BeaconAddUpdate(None)),
                                disabled={
                                    match self.current_page {
                                        // match ignoring the fields
                                        Page::BeaconAddUpdate {..} => true,
                                        _ => false,
                                    }
                                },
                            >
                                { "Add Beacon" }
                            </a>
                    </div>
                </>
            },
            WebUserType::Responder => html! { }
        };

        let select_map = match self.user_type {
            WebUserType::Admin => html! {
                <>
                    <a
                        class = match self.current_page {
                            Page::MapList => {"nav-link dropdown navBarText active"},
                            Page::MapAddUpdate{..} => {"nav-link dropdown navBarText active"},
                            _ => {"nav-link dropdown navBarText"},
                        }
                        role="button",
                        data-toggle="dropdown",
                        onclick=|_| Msg::ChangePage(Page::MapList),
                    >
                            { "Maps" }
                    </a>
                    <div class="dropdown-content">
                        <a
                            class = "dropdown-item navBarText",
                            onclick=|_| Msg::ChangePage(Page::MapList),
                            disabled={self.current_page == Page::MapList},
                        >
                                { "Map List" }
                        </a>
                        <a
                            class = "dropdown-item navBarText",
                            onclick=|_| Msg::ChangePage(Page::MapAddUpdate(None)),
                            disabled={
                                match self.current_page {
                                    // match ignoring the fields
                                    Page::MapAddUpdate {..} => true,
                                    _ => false,
                                }
                            },
                        >
                            { "Add Map" }
                        </a>
                    </div>
                </>
            },
            WebUserType::Responder => html! { }
        };

        let select_system = match self.user_type {
            WebUserType::Admin => html! {
                <>
                    <a
                        class = match self.current_page {
                            Page::SystemSettings => {"nav-link navBarText active"}
                            Page::Diagnostics {..} => {"nav-link navBarText active"},
                            _ => {"nav-link navBarText"},
                        },
                        role="button"
                        onclick=|_| Msg::ChangePage(Page::SystemSettings),
                    >
                        { " System" }
                    </a>
                    <div class="dropdown-content">
                        <a
                            class="dropdown-item navBarText",
                            onclick=|_| Msg::ChangePage(Page::SystemSettings),
                            disabled={self.current_page == Page::SystemSettings},>
                                { "System Settings" }
                        </a>
                        <a
                            class="dropdown-item navBarText",
                            onclick=|_| Msg::ChangePage(Page::Diagnostics),
                            disabled={self.current_page == Page::Diagnostics},>
                                { "Diagnostics" }
                        </a>
                    </div>
                </>
            },
            WebUserType::Responder => html! { }
        };

        let login_type = match self.user_type {
            WebUserType::Admin => html! {
                <>
                    <button
                        class="btn btn-danger btn-sm nav-link logoutPlacement ml-auto",
                        onclick=|_| Msg::ChangePage(Page::Login(login::AutoAction::Logout)),
                        disabled={
                            match self.current_page {
                                Page::Login{..} => true,
                                _ => false,
                            }
                        },
                    >
                        { "Logout" }
                    </button>
                    <a class="loginTypeHeader">{ "ADMIN" }</a>
                </>
            },
            WebUserType::Responder => html! {
                <>
                    <button
                        class="btn btn-success btn-sm nav-link logoutPlacement ml-auto",
                        onclick=|_| Msg::ChangePage(Page::Login(login::AutoAction::Nothing)),
                        disabled={
                            match self.current_page {
                                Page::Login{..} => true,
                                _ => false,
                            }
                        },
                    >
                        { "Login" }
                    </button>
                    <a class="loginTypeHeader">{ "FIRST RESPONDER" }</a>
                </>
            }
        };
        html! {
            <nav class="navbar navbar-expand-sm navbarColour">
                <a class="navbar-brand">
                    <img src="/images/icon.PNG" width="52" height="48" class="d-inline-block align-top" alt=""/>
                </a>
                <div class="navbarJustify">
                    <ul class="nav navbarText">
                        <li class="my-auto">
                            {view_map}
                        </li>
                        <li class="dropdown my-auto">
                            { show_status }
                        </li>
                        <li class="dropdown my-auto">
                            { select_map }
                        </li>
                        <li class="dropdown my-auto">
                            { select_beacon }
                        </li>
                        <li class="dropdown my-auto">
                            { select_user }
                        </li>
                        <li class="dropdown my-auto">
                            { select_system }
                        </li>
                    </ul>
                </div>
                {login_type}
            </nav>
        }
    }
}
