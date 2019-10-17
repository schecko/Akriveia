
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
                    <div>
                        <h>{ "Diagnostics" }</h>
                        { self.navigation() }
                        <div>
                            <EmergencyButtons
                                is_emergency={self.emergency},
                                on_emergency=|_| Msg::RequestPostEmergency(true),
                                on_end_emergency=|_| Msg::RequestPostEmergency(false),
                            />
                        </div>
                        <Diagnostics
                            emergency={self.emergency}
                        />
                    </div>
                }
            },
            Page::Status => {
                html! {
                    <div>
                        <h>{ "Status" }</h>
                        { self.navigation() }
                        <div>
                            <EmergencyButtons
                                is_emergency={self.emergency},
                                on_emergency=|_| Msg::RequestPostEmergency(true),
                                on_end_emergency=|_| Msg::RequestPostEmergency(false),
                            />
                        </div>
                        <Status
                            change_page=|page| Msg::ChangePage(page),
                        />
                    </div>
                }
            },
            Page::Login => {
                html! {
                    <div>
                        <h>{ "Login" }</h>
                        <Login
                            change_page=|page| Msg::ChangePage(page),
                            change_user_type=|user_type| Msg::ChangeWebUserType(user_type),
                        />
                    </div>
                }
            },
            Page::MapView(opt_id) => {
                html! {
                    <div>
                        <h>{ "MapView" }</h>
                        { self.navigation() }
                        <div>
                            <EmergencyButtons
                                is_emergency={self.emergency},
                                on_emergency=|_| Msg::RequestPostEmergency(true),
                                on_end_emergency=|_| Msg::RequestPostEmergency(false),
                            />
                        </div>
                        <MapViewComponent
                            emergency={self.emergency},
                            opt_id=opt_id,
                        />
                    </div>
                }
            },
            Page::BeaconList => {
               html! {
                    <div>
                        <h>{ "Beacon" }</h>
                        { self.navigation() }
                        <BeaconList
                            change_page=|page| Msg::ChangePage(page),
                        />
                    </div>
                }
            },
            Page::BeaconAddUpdate(id) => {
               html! {
                    <div>
                        <h>{ "Beacon" }</h>
                        { self.navigation() }
                        <BeaconAddUpdate
                            id=id,
                            user_type=self.user_type,
                        />
                    </div>
                }
            },
            Page::UserList => {
                html! {
                    <div>
                        <h>{ "User" }</h>
                        { self.navigation() }
                        <UserList
                            change_page=|page| Msg::ChangePage(page),
                        />
                    </div>
                }
            },
            Page::UserAddUpdate(id) => {
                html! {
                    <div>
                        <h>{ "User" } </h>
                        { self.navigation() }
                        <UserAddUpdate
                            id=id,
                            user_type=self.user_type,
                        />
                    </div>
                }
            },
            Page::MapList => {
               html! {
                    <div>
                        <h>{ "Map" }</h>
                        { self.navigation() }
                        <MapList
                            change_page=|page| Msg::ChangePage(page),
                        />
                    </div>
                }
            },
            Page::MapAddUpdate(opt_id) => {
               html! {
                    <div>
                        <h>{ "Map" }</h>
                        { self.navigation() }
                        <MapAddUpdate
                            opt_id=opt_id,
                            user_type=self.user_type,
                        />
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
                <select>
                    <option disabled=true,>{ "User Config(Header)" }</option>
                    <option onclick=|_| Msg::ChangePage(Page::UserList), disabled={self.current_page == Page::UserList},>{ "User List" } </option>
                    <option
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
                    </option>
                </select>
            },
            WebUserType::Responder => html! {
                <></>
            },
        };

        let select_beacon = match self.user_type {
            WebUserType::Admin => html! {
                <select>
                    <option disabled=true,>{ "Beacon Config(Header)" }</option>
                    <option onclick=|_| Msg::ChangePage(Page::BeaconList), disabled={self.current_page == Page::BeaconList},>{ "Beacon List" }</option>
                    <option
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
                    </option>
                </select>
            },
            WebUserType::Responder => html! {
                <></>
            }
        };

        let select_map = match self.user_type {
            WebUserType::Admin => html! {
                <select>
                    <option disabled=true,>{ "Map Config(Header)" }</option>
                    <option onclick=|_| Msg::ChangePage(Page::MapList), disabled={self.current_page == Page::MapList},>{ "Map List" }</option>
                    <option
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
                    </option>
                    <option
                        onclick=|_| Msg::ChangePage(Page::MapView(None)),
                        disabled={
                            match self.current_page {
                                Page::MapView { .. } => true,
                                _ => false,
                            }
                        },
                    >
                        { "MapView" }
                    </option>
                </select>
            },
            WebUserType::Responder => html! {
                <button
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
                <button onclick=|_| Msg::ChangePage(Page::Diagnostics), disabled={self.current_page == Page::Diagnostics},>{ "Diagnostics" }</button>
            },
            WebUserType::Responder => html! {
                <></>
            }
        };

        html! {
            <div>
                <button onclick=|_| Msg::ChangePage(Page::Login), disabled={self.current_page == Page::Login},>{ "Logout" }</button>
                { diagnostics }
                <button onclick=|_| Msg::ChangePage(Page::Status), disabled={self.current_page == Page::Status},>{ "Status" }</button>
                { select_user }
                { select_beacon }
                { select_map }
            </div>
        }
    }
}
