
use failure::Error;
use yew::format::{Nothing, Json};
use yew::services::console::ConsoleService;
use yew::services::fetch::{FetchService, FetchTask, Request, Response};
use yew::services::interval::*;
use yew::{html, Component, ComponentLink, Html, Renderable, ShouldRender};
use common;
use std::time::Duration;

pub enum Page {
    Diagnostics,
    FrontPage,
    Login,
}

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
    ChangePage(Page),
    FetchDiagnostics,
    FetchDiagnosticsReady(Result<common::DiagnosticData, Error>),
    FetchHello,
    FetchEmergency,
    FetchEndEmergency,
    FetchReady(Result<common::HelloFrontEnd, Error>),
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
                        self.diagnostic_service_task = Some(interval_service.spawn(Duration::from_millis(1000), self.link.send_back(|_| Msg::FetchDiagnostics)));
                        self.diagnostic_service = Some(interval_service);
                    }
                    _ => {
                        // do nothing
                    }
                }
            },
            Msg::FetchHello => {
                self.fetch_in_flight = true;
                let callback = self.link.send_back(move |response: Response<Json<Result<common::HelloFrontEnd, Error>>>| {
                    let (meta, Json(data)) = response.into_parts();
                    println!("META: {:?}", meta);
                    Log!("META: {:?}", meta);
                    if meta.status.is_success() {
                        Msg::FetchReady(data)
                    } else {
                        Msg::Ignore
                    }
                });
                let request = Request::get(common::PING)
                    .header("Content-Type", "text/html")
                    .header("Accept", "text/html")
                    .body(Nothing)
                    .unwrap();
                let task = self.fetch_service.fetch(request, callback);
                self.fetch_task = Some(task);
            },
            Msg::FetchEmergency => {
                self.fetch_in_flight = true;
                let callback = self.link.send_back(move |response: Response<Json<Result<_, Error>>>| {
                    let (meta, Json(data)) = response.into_parts();
                    println!("META: {:?}", meta);
                    Log!("META: {:?}", meta);
                    if meta.status.is_success() {
                        Msg::FetchReady(data)
                    } else {
                        Msg::Ignore
                    }
                });
                let request = Request::post(common::EMERGENCY)
                    .header("Content-Type", "text/html")
                    .header("Accept", "text/html")
                    .body(Nothing)
                    .unwrap();
                let task = self.fetch_service.fetch(request, callback);
                self.fetch_task = Some(task);
            },
            Msg::FetchEndEmergency => {
                self.fetch_in_flight = true;
                let callback = self.link.send_back(move |response: Response<Json<Result<_, Error>>>| {
                    let (meta, Json(data)) = response.into_parts();
                    println!("META: {:?}", meta);
                    Log!("META: {:?}", meta);
                    if meta.status.is_success() {
                        Msg::FetchReady(data)
                    } else {
                        Msg::Ignore
                    }
                });
                let request = Request::post(common::END_EMERGENCY)
                    .header("Content-Type", "text/html")
                    .header("Accept", "text/html")
                    .body(Nothing)
                    .unwrap();
                let task = self.fetch_service.fetch(request, callback);
                self.fetch_task = Some(task);
                self.diagnostic_data = Vec::new();
            },
            Msg::FetchDiagnostics => {
                self.fetch_in_flight = true;
                let callback = self.link.send_back(move |response: Response<Json<Result<common::DiagnosticData, Error>>>| {
                    let (meta, Json(data)) = response.into_parts();
                    println!("META: {:?}", meta);
                    Log!("META: {:?}", meta);
                    if meta.status.is_success() {
                        Msg::FetchDiagnosticsReady(data)
                    } else {
                        Log!("failed request");
                        Msg::Ignore
                    }
                });
                let request = Request::get(common::DIAGNOSTICS)
                    .header("Content-Type", "text/html")
                    .header("Accept", "text/html")
                    .body(Nothing)
                    .unwrap();
                let task = self.fetch_service.fetch(request, callback);
                self.fetch_task = Some(task);
            },
            Msg::FetchDiagnosticsReady(response) => {
                self.fetch_in_flight = false;
                if let Ok(mut data) = response {
                    self.diagnostic_data.append(&mut data.tag_data);
                }
            },
            Msg::FetchReady(response) => {
                self.fetch_in_flight = false;
                self.data = response.ok();
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
                            <button onclick=|_| Msg::FetchEmergency,>{ "Start Emergency" }</button>
                            <button onclick=|_| Msg::FetchEndEmergency,>{ "End Emergency" }</button>
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
