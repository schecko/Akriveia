
use common::*;
use crate::util;
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

#[derive(PartialEq)]
pub enum Page {
    BeaconAddUpdate(Option<i32>),
    BeaconList,
    UserAddUpdate(Option<i32>),
    UserList,
    Diagnostics,
    FrontPage,
    Login,
    MapView(Option<i32>),
    MapList,
    MapAddUpdate(Option<i32>),
}

pub struct RootComponent {
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
            current_page: Page::FrontPage,
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
                match self.current_page {
                    _ => { }
                }
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
            }
            Page::Login => {
                html! {
                    <div>
                        <h>{ "Login" }</h>
                        { self.navigation() }
                        { self.view_data() }
                    </div>
                }
            }
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
            }
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
            }
            Page::BeaconAddUpdate(id) => {
               html! {
                    <div>
                        <h>{ "Beacon" }</h>
                        { self.navigation() }
                        <BeaconAddUpdate
                            id=id,
                        />
                    </div>
                }
            }
            Page::UserList => {
                html! {
                    <div>
                        <h>{ "User" }</h>
                        {self.navigation() }
                        <UserList
                            change_page=|page| Msg::ChangePage(page),
                            />
                    </div>
                }
            }
            Page::UserAddUpdate(id) => {
                html! {
                    <div>
                        <h>{ "User" } </h>
                        { self.navigation() }
                        <UserAddUpdate
                            id=id,
                        />
                    </div>
                }
            }
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
            }
            Page::MapAddUpdate(opt_id) => {
               html! {
                    <div>
                        <h>{ "Map" }</h>
                        { self.navigation() }
                        <MapAddUpdate
                            opt_id=opt_id,
                        />
                    </div>
                }
            }
            Page::FrontPage => {
                html! {
                    <div>
                        <h>{ "FrontPage" }</h>
                        <button onclick=|_| Msg::ChangePage(Page::Login),>{ "Click" }</button>
                    </div>
                }
            }
        }
    }
}

impl RootComponent {
    fn navigation(&self) -> Html<Self> {
        html! {
            <div>
                <button onclick=|_| Msg::ChangePage(Page::Login), disabled={self.current_page == Page::Login},>{ "Login Page" }</button>
                <button onclick=|_| Msg::ChangePage(Page::Diagnostics), disabled={self.current_page == Page::Diagnostics},>{ "Diagnostics" }</button>
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
                <select>
                    // TODO CSS for navigation bar
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
                <select>
                    // Adding User List
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
                    // TODO CSS for navigation bar
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
                </select>
            </div>
        }
    }


    fn view_data(&self) -> Html<RootComponent> {
        html! {
            <p>{ "Its empty in here." }</p>
        }
    }
}
