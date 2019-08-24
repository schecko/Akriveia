
use common::*;
use crate::util;
use yew::format::{ Nothing, Json };
use yew::services::fetch::{ FetchService, FetchTask, Request, };
use yew::{ Component, ComponentLink, Html, Renderable, ShouldRender, html, };
use super::map_view::MapViewComponent;
use super::emergency_buttons::EmergencyButtons;
use super::diagnostics::Diagnostics;
use super::beacon_list::BeaconList;
use super::beacon_add::BeaconAdd;

#[derive(PartialEq)]
pub enum Page {
    Diagnostics,
    FrontPage,
    Login,
    Map,
    BeaconList,
    BeaconAdd,
}

pub struct RootComponent {
    emergency: bool,
    current_page: Page,
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
            emergency: false,
            current_page: Page::FrontPage,
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
            Page::Map => {
                html! {
                    <div>
                        <h>{ "Map" }</h>
                        { self.navigation() }
                        <div>
                            <EmergencyButtons
                                is_emergency={self.emergency},
                                on_emergency=|_| Msg::RequestPostEmergency(true),
                                on_end_emergency=|_| Msg::RequestPostEmergency(false),
                            />
                        </div>
                        <MapViewComponent
                            emergency={self.emergency}
                        />
                    </div>
                }
            }
            Page::BeaconList => {
               html! {
                    <div>
                        <h>{ "BeaconList" }</h>
                        { self.navigation() }
                        <BeaconList/>
                    </div>
                }
            }
            Page::BeaconAdd => {
               html! {
                    <div>
                        <h>{ "BeaconAdd" }</h>
                        { self.navigation() }
                        <BeaconAdd/>
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
                <button onclick=|_| Msg::ChangePage(Page::Map), disabled={self.current_page == Page::Map},>{ "Map" }</button>
                <select>
                    // TODO CSS for navigation bar
                    <option disabled=true,>{ "Beacon Config(Header)" }</option>
                    <option onclick=|_| Msg::ChangePage(Page::BeaconList), disabled={self.current_page == Page::BeaconList},>{ "Beacon List" }</option>
                    <option onclick=|_| Msg::ChangePage(Page::BeaconAdd), disabled={self.current_page == Page::BeaconAdd},>{ "Add Beacon" }</option>
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
