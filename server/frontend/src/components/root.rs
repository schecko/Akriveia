
use failure::Error;
use yew::format::{Nothing, Json};
use yew::services::console::ConsoleService;
use yew::services::fetch::{FetchService, FetchTask, Request, Response};
use yew::services::interval::*;
use yew::{html, Component, ComponentLink, Html, Renderable, ShouldRender};
use common;
use std::time::Duration;
use crate::util;

pub enum Page {
    Diagnostics,
    FrontPage,
    Login,
}

#[warn(unused_macros)]
macro_rules! Log {
    ($($arg:tt)*) => (
        let mut console = ConsoleService::new();
        console.log(format!($($arg)*).as_str());
    )
}

pub struct RootComponent {
    current_page: Page,
    data: Option<common::HelloFrontEnd>,
    diagnostic_data: Vec<common::TagData>,
    diagnostic_service: Option<IntervalService>,
    diagnostic_service_task: Option<IntervalTask>,
    fetch_in_flight: bool,
    fetch_service: FetchService,
    fetch_task: Option<FetchTask>,
    link: ComponentLink<RootComponent>,
}

pub enum Msg {
    Ignore,
    // page changes
    ChangePage(Page),

    // requests
    RequestPing,
    RequestDiagnostics,
    RequestEmergency,
    RequestEndEmergency,

    // responses
    ResponsePing(util::Response<common::HelloFrontEnd>),
    ResponseDiagnostics(util::Response<common::DiagnosticData>),
    ResponseEmergency(util::Response<()>),
    ResponseEndEmergency(util::Response<()>),
}

impl Component for RootComponent {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        RootComponent {
            current_page: Page::Login,
            data: None,
            diagnostic_service: None,
            diagnostic_service_task: None,
            diagnostic_data: Vec::new(),
            fetch_service: FetchService::new(),
            fetch_in_flight: false,
            fetch_task: None,
            link: link,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::ChangePage(page) => {
                self.current_page = page;
                match self.current_page {
                    Page::Diagnostics => {
                        let mut interval_service = IntervalService::new();
                        self.diagnostic_service_task = Some(interval_service.spawn(Duration::from_millis(1000), self.link.send_back(|_| Msg::RequestDiagnostics)));
                        self.diagnostic_service = Some(interval_service);
                    },
                    _ => {
                        // do nothing
                    }
                }
            },

            // requests
            Msg::RequestPing => {
                self.fetch_task = get_request!(
                    self,
                    common::PING,
                    self.link,
                    Msg::ResponsePing
                );
            },
            Msg::RequestEmergency => {
                self.fetch_task = post_request!(
                    self,
                    common::EMERGENCY,
                    (),
                    self.link,
                    Msg::ResponseEmergency
                );
            },
            Msg::RequestEndEmergency => {
                self.fetch_task = post_request!(
                    self,
                    common::END_EMERGENCY,
                    (),
                    self.link,
                    Msg::ResponseEndEmergency
                );
            },
            Msg::RequestDiagnostics => {
                self.fetch_task = get_request!(
                    self,
                    common::DIAGNOSTICS,
                    self.link,
                    Msg::ResponseDiagnostics
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
            Msg::ResponseDiagnostics(response) => {
                let (meta, Json(body)) = response.into_parts();
                if meta.status.is_success() {
                    match body {
                        Ok(common::DiagnosticData { mut tag_data }) => {
                            self.diagnostic_data.append(&mut tag_data);
                        }
                        _ => { }
                    }
                } else {
                    Log!("response - failed to request diagnostics");
                }
            },
            Msg::ResponseEmergency(_response) => {
                println!("emergency response");
            },
            Msg::ResponseEndEmergency(_response) => {
                println!("endemergency response");
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
                        <div>
                            <button onclick=|_| Msg::ChangePage(Page::Login),>{ "Login Page" }</button>
                        </div>
                        <div>
                            <button onclick=|_| Msg::RequestEmergency,>{ "Start Emergency" }</button>
                            <button onclick=|_| Msg::RequestEndEmergency,>{ "End Emergency" }</button>
                        </div>
                        <h>{ "Diagnostics" }</h>
                        { self.render_diagnostics() }
                    </div>
                }
            }
            Page::Login => {
                html! {
                    <div>
                        <div>
                            <button onclick=|_| Msg::ChangePage(Page::Diagnostics),>{ "Diagnostics Page" }</button>
                        </div>
                        <h>{ "Login" }</h>
                        { self.view_data() }
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
    fn view_data(&self) -> Html<RootComponent> {
        html! {
            <p>{ "Its empty in here." }</p>
        }
    }

    fn render_diagnostics(&self) -> Html<RootComponent> {
        if self.diagnostic_data.len() > 0 {
            html! {
                <table> {
                    for self.diagnostic_data.iter().map(|row| {
                        match row.tag_distance {
                            common::DataType::RSSI(strength) => {
                                html! {
                                    <tr>{ format!("name: {}\tmac: {}\trssi: {}", &row.tag_name, &row.tag_mac, strength ) } </tr>
                                }
                            },
                            common::DataType::TOF(distance) => {
                                html! {
                                    <tr>{ format!("name: {}\tmac: {}\ttof: {}", &row.tag_name, &row.tag_mac, distance ) } </tr>
                                }
                            },
                        }
                    })
                } </table>
            }
        } else {
            html! {
                <p>{ "No diagnostics yet..." }</p>
            }
        }
    }
}
