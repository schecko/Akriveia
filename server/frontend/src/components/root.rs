
use common;
use crate::util;
use failure::Error;
use std::convert::TryFrom;
use std::time::Duration;
use stdweb::web::{ CanvasRenderingContext2d, Element, HtmlElement, Node, };
use stdweb::web::html_element::CanvasElement;
use stdweb::web;
use yew::format::{ Nothing, Json };
use yew::services::console::ConsoleService;
use yew::services::fetch::{ FetchService, FetchTask, Request, Response, };
use yew::services::interval::*;
use yew::virtual_dom::vnode::VNode;
use yew::{ Component, ComponentLink, Html, Renderable, ShouldRender, html, };

use super::map_view::MapViewComponent;
use super::emergency_buttons::EmergencyButtons;
use super::diagnostics::Diagnostics;

#[derive(PartialEq)]
pub enum Page {
    Diagnostics,
    FrontPage,
    Login,
    Map,
}

fn get_canvas() -> CanvasElement {
    unsafe {
       js! (
            return document.querySelector("canvas");
        ).into_reference_unchecked().unwrap()
    }
}

fn get_canvas_by_id(id: &str) -> CanvasElement {
    unsafe {
       js! (
            return document.getElementById(id);
        ).into_reference_unchecked().unwrap()
    }
}

fn get_context(canvas: &CanvasElement) -> CanvasRenderingContext2d {
    unsafe {
        js! (
            return @{canvas}.getContext("2d");
        ).into_reference_unchecked().unwrap()
    }
}

pub struct RootComponent {
    emergency: bool,
    current_page: Page,
    data: Option<common::HelloFrontEnd>,
    diagnostic_data: Vec<common::TagData>,
    diagnostic_service: Option<IntervalService>,
    diagnostic_service_task: Option<IntervalTask>,
    fetch_in_flight: bool,
    fetch_service: FetchService,
    fetch_task: Option<FetchTask>,
    link: ComponentLink<RootComponent>,
    map_canvas: Option<CanvasElement>,
}

pub enum Msg {
    // local functionality
    Ignore,

    // page changes
    ChangePage(Page),

    // requests
    RequestPing,
    RequestEmergency,
    RequestEndEmergency,
    RequestGetEmergency,

    // responses
    ResponsePing(util::Response<common::HelloFrontEnd>),
    ResponseEmergency(util::Response<()>),
    ResponseEndEmergency(util::Response<()>),
    ResponseGetEmergency(util::Response<common::SystemCommandResponse>),
}

impl Component for RootComponent {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        RootComponent {
            emergency: false,
            current_page: Page::Login,
            data: None,
            diagnostic_service: None,
            diagnostic_service_task: None,
            diagnostic_data: Vec::new(),
            fetch_service: FetchService::new(),
            fetch_in_flight: false,
            fetch_task: None,
            link: link,
            map_canvas: None,
        }
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
            Msg::RequestPing => {
                self.fetch_task = get_request!(
                    self.fetch_service,
                    common::PING,
                    self.link,
                    Msg::ResponsePing
                );
            },
            Msg::RequestEmergency => {
                self.fetch_task = post_request!(
                    self.fetch_service,
                    common::EMERGENCY,
                    (),
                    self.link,
                    Msg::ResponseEmergency
                );
            },
            Msg::RequestEndEmergency => {
                self.fetch_task = post_request!(
                    self.fetch_service,
                    common::END_EMERGENCY,
                    (),
                    self.link,
                    Msg::ResponseEndEmergency
                );
            },
            Msg::RequestGetEmergency => {
                self.fetch_task = get_request!(
                    self.fetch_service,
                    common::EMERGENCY,
                    self.link,
                    Msg::ResponseGetEmergency
                );
            },

            // responses
            Msg::ResponsePing(response) => {
                let (meta, Json(body)) = response.into_parts();
                if meta.status.is_success() {
                    match body {
                        Ok(common::HelloFrontEnd { data }) => {
                            println!("success {:?}", data);
                        }
                        _ => { }
                    }

                } else {
                    Log!("response - failed to ping");
                }
            },
            Msg::ResponseEmergency(_response) => {
                self.emergency = true;
            },
            Msg::ResponseEndEmergency(_response) => {
                self.emergency = false;
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
                    Log!("response - failed to request diagnostics");
                }
            },

            Msg::Ignore => {
                // do nothing
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
                                on_emergency=|_| Msg::RequestEmergency,
                                on_end_emergency=|_| Msg::RequestEndEmergency,
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
                                on_emergency=|_| Msg::RequestEmergency,
                                on_end_emergency=|_| Msg::RequestEndEmergency,
                            />
                        </div>
                        <MapViewComponent/>
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
            </div>
        }

    }

    fn view_data(&self) -> Html<RootComponent> {
        html! {
            <p>{ "Its empty in here." }</p>
        }
    }
}
