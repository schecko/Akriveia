
use common::*;
use crate::util::{ self, WebUserType, };
use yew::format::Json;
use yew::services::fetch::{ FetchService, FetchTask, };
use yew::prelude::*;
use super::map_view::MapViewComponent;
use super::emergency_buttons::EmergencyButtons;
use super::diagnostics::Diagnostics;
use super::beacon_list::BeaconList;
use super::beacon_addupdate::BeaconAddUpdate;
use super::user_list::UserList;
use super::user_addupdate::UserAddUpdate;
use super::map_list::MapList;
use super::map_addupdate::MapAddUpdate;
use super::status::Status;
use super::login::Login;

#[derive(PartialEq)]
pub enum Page {
    BeaconAddUpdate(Option<i32>),
    BeaconList,
    UserAddUpdate(Option<i32>),
    UserList,
    Diagnostics,
    Status,
    Login,
    MapView(Option<i32>),
    MapList,
    MapAddUpdate(Option<i32>),
}

pub struct RootComponent {
    user_type: WebUserType,
    current_page: Page,
    emergency: bool,
    fetch_service: FetchService,
    fetch_task: Option<FetchTask>,
    link: ComponentLink<RootComponent>,
}

pub enum Msg {
    // local functionality

    // page changes
    ChangePage(Page),
    ChangeWebUserType(WebUserType),

    // requests
    RequestPostEmergency(bool),
    RequestGetEmergency,

    // responses
    ResponsePostEmergency(util::Response<common::SystemCommandResponse>),
    ResponseGetEmergency(util::Response<common::SystemCommandResponse>),
}

impl Component for RootComponent {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, mut link: ComponentLink<Self>) -> Self {
        link.send_self(Msg::RequestGetEmergency);
        let root = RootComponent {
            user_type: WebUserType::Responder,
            current_page: Page::Login,
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
                let (meta, Json(body)) = response.into_parts();
                if meta.status.is_success() {
                    match body {
                        Ok(common::SystemCommandResponse { emergency }) => {
                            self.emergency = emergency;
                        }
                        _ => { }
                    }
                } else {
                    Log!("response - failed to post start emergency");
                }
            },
            Msg::ResponseGetEmergency(response) => {
                let (meta, Json(body)) = response.into_parts();
                if meta.status.is_success() {
                    match body {
                        Ok(common::SystemCommandResponse { emergency }) => {
                            self.emergency = emergency;
                        }
                        _ => { }
                    }
                } else {
                    Log!("response - failed to request emergency status");
                }
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
                        <div class="container-fluid">
                            <h1>{ "Diagnostics" }</h1>
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
            Page::Status => {
                html! {
                    <div class="page-content-wrapper">
                        { self.navigation() }
                        <div class="container-fluid">
                            <h1>{ "Status" }</h1>
                            <EmergencyButtons
                                is_emergency={self.emergency},
                                on_emergency=|_| Msg::RequestPostEmergency(true),
                                on_end_emergency=|_| Msg::RequestPostEmergency(false),
                            />
                            <Status
                                change_page=|page| Msg::ChangePage(page),
                            />
                        </div>
                    </div>
                }
            },
            Page::Login => {
                html! {
                    <div class="container-fluid">
                        //<h1>{ "Login Page" }</h1>
                        <Login
                            change_page=|page| Msg::ChangePage(page),
                            change_user_type=|user_type| Msg::ChangeWebUserType(user_type),
                        />
                    </div>
                }
            },
            Page::MapView(opt_id) => {
                html! {
                    <div class="page-content-wrapper">
                        { self.navigation() }
                        <div class="container-fluid">
                            <h1>{ "MapView" }</h1>
                            <EmergencyButtons
                                is_emergency={self.emergency},
                                on_emergency=|_| Msg::RequestPostEmergency(true),
                                on_end_emergency=|_| Msg::RequestPostEmergency(false),
                            />
                            <MapViewComponent
                                emergency={self.emergency},
                                opt_id=opt_id,
                            />
                        </div>
                    </div>
                }
            },
            Page::BeaconList => {
               html! {
                    <div class="page-content-wrapper">
                        { self.navigation() }
                        <div class="container-fluid">
                            <h1>{ "Beacon" }</h1>
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
                        <div class="container-fluid">
                            <h1>{ "Beacon" }</h1>
                            <BeaconAddUpdate
                                id=id,
                                user_type=self.user_type,
                            />
                        </div>
                    </div>
                }
            },
            Page::UserList => {
                html! {
                    <div class="page-content-wrapper">
                        { self.navigation() }
                        <div class="container-fluid">
                            <h1>{ "User" }</h1>
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
                        <div class="container-fluid">
                            <h1>{ "User" } </h1>
                            <UserAddUpdate
                                id=id,
                                user_type=self.user_type,
                            />
                        </div>
                    </div>
                }
            },
            Page::MapList => {
               html! {
                    <div class="page-content-wrapper">
                        { self.navigation() }
                        <div class="container-fluid">
                            <h1>{ "Map" }</h1>
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
                        <div class="container-fluid">
                            <h1>{ "Map" }</h1>
                            <MapAddUpdate
                                opt_id=opt_id,
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
        let select_user = match self.user_type {
            WebUserType::Admin => html! {
                <>
                    <a class="nav-link dropdown-toggle" id="navbarDropDownUsers" role="button" data-toggle="dropdown" aria-haspopup="true" aria-expanded="false">
                        { "User Config" }
                    </a>
                    <div class="dropdown-menu" aria-labelledby="navbarDropDownUsers">
                        <a class ="dropdown-item" onclick=|_| Msg::ChangePage(Page::UserList), disabled={self.current_page == Page::UserList},>
                            { "User List" }
                        </a>
                        <a
                            class="dropdown-item",
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
            WebUserType::Responder => html! {
                <></>
            },
        };

        let select_beacon = match self.user_type {
            WebUserType::Admin => html! {
                <>
                    <a class="nav-link dropdown-toggle" id="navbarDropDownBeacons" role="button" data-toggle="dropdown" aria-haspopup="true" aria-expanded="false">
                        { "Beacons" }
                    </a>
                    <div class="dropdown-menu" aria-labelledby="navbarDropdownBeacons">
                        <a class="dropdown-item" onclick=|_| Msg::ChangePage(Page::BeaconList), disabled={self.current_page == Page::BeaconList},>
                            { "Beacon List" }
                        </a>
                        <a
                            class="dropdown-item",
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
            WebUserType::Responder => html! {
                <></>
            }
        };

        let select_map = match self.user_type {
            WebUserType::Admin => html! {
                <>
                    <a class="nav-link dropdown-toggle" id="navbarDropDownMaps" role="button" data-toggle="dropdown" aria-haspopup="true" aria-expanded="false">
                        { "Maps" }
                    </a>
                    <div class="dropdown-menu" aria-labelledby="navbarDropDownMaps">
                        <a class="dropdown-item" onclick=|_| Msg::ChangePage(Page::MapList), disabled={self.current_page == Page::MapList},>
                            { "Map List" }
                        </a>
                        <a
                            class="dropdown-item"
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
                        <a
                            class="dropdown-item"
                            onclick=|_| Msg::ChangePage(Page::MapView(None)),
                            disabled={
                                match self.current_page {
                                    Page::MapView { .. } => true,
                                    _ => false,
                                }
                            },
                        >
                            { "MapView" }
                        </a>
                    </div>
                </>
            },
            WebUserType::Responder => html! {
                <button
                    class="btn btn-primary btn-lg active"
                    onclick=|_| Msg::ChangePage(Page::MapView(None)),
                    disabled={
                        match self.current_page {
                            Page::MapView { .. } => true,
                            _ => false,
                        }   
                    },
                >
                    { "MapView" }
                </button>
            }
        };

        let diagnostics = match self.user_type {
            WebUserType::Admin => html! {
                <>
                    <a class="nav-link" onclick=|_| Msg::ChangePage(Page::Diagnostics), disabled={self.current_page == Page::Diagnostics},>{ "Diagnostics" }</a>
                </>
            },
            WebUserType::Responder => html! {
                <></>
            }
        };

        html! {
            <nav class="navbar navbar-expand-lg navbar-dark bg-dark" /*style="background-color: #be0010;"*/>
                <a class="navbar-brand">
                    <img src="/images/icon_780_720.png" width="39" height="36" class="d-inline-block align-top" alt=""/>
                    {"Akriveia"}
                </a>
                <div class="collapse navbar-collapse" id="navbarNav">
                    <ul class="navbar-nav">
                        <li class="nav-item">
                            { diagnostics }
                        </li>
                        <li class="nav-item">
                            <a class="nav-link" onclick=|_| Msg::ChangePage(Page::Status), disabled={self.current_page == Page::Status},>{ "Status" }</a>
                        </li>
                        <li class="nav-item dropdown">
                            { select_beacon }
                        </li>
                        <li class="nav-item dropdown">
                            { select_user }
                        </li>
                        <li class="nav-item dropdown">
                            { select_map }
                        </li>
                    </ul>
                </div>
            </nav>
        }
    }
}