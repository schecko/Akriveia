
use failure::Error;
use serde_derive::{Deserialize, Serialize};
use yew::format::{Nothing, Json};
use yew::services::console::ConsoleService;
use yew::services::fetch::{FetchService, FetchTask, Request, Response};
use yew::{html, Component, ComponentLink, Html, Renderable, ShouldRender};

pub enum Page {
    Login,
    FrontPage,
}

macro_rules! Log {
    ($($arg:tt)*) => (
        let mut console = ConsoleService::new();
        console.log(format!($($arg)*).as_str());
    )
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HelloFrontEnd {
    data: u32,
}

pub struct RootComponent {
    current_page: Page,
    data: Option<HelloFrontEnd>,
    fetch_service: FetchService,
    fetch_in_flight: bool,
    fetch_task: Option<FetchTask>,
    link: ComponentLink<RootComponent>,
}

pub enum Msg {
    Ignore,
    ChangePage(Page),
    FetchHello,
    FetchReady(Result<HelloFrontEnd, Error>),
}

impl Component for RootComponent {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        RootComponent {
            current_page: Page::Login,
            data: None,
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
            },
            Msg::FetchHello => {
                self.fetch_in_flight = true;
                let callback = self.link.send_back(move |response: Response<Json<Result<HelloFrontEnd, Error>>>| {
                    let (meta, Json(data)) = response.into_parts();
                    println!("META: {:?}", meta);
                    Log!("META: {:?}", meta);
                    if meta.status.is_success() {
                        Msg::FetchReady(data)
                    } else {
                        Msg::Ignore
                    }
                });
                let request = Request::get("/hello")
                    .header("Content-Type", "text/html")
                    .header("Accept", "text/html")
                    .body(Nothing)
                    .unwrap();
                let task = self.fetch_service.fetch(request, callback);
                self.fetch_task = Some(task);
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
            Page::Login => {
                html! {
                    <div>
                        <p>{ "Hello Login Page!" }</p>
                        <button onclick=|_| Msg::ChangePage(Page::FrontPage),>{ "Click" }</button>
                        <button onclick=|_| Msg::FetchHello,>{ "Get Hello" }</button>
                        { self.view_data() }
                    </div>
                }
            }
            Page::FrontPage => {
                html! {
                    <div>
                        <p>{ "Hello FrontPage Page!" }</p>
                        <button onclick=|_| Msg::ChangePage(Page::Login),>{ "Click" }</button>
                    </div>
                }
            }
        }
    }
}

impl RootComponent {
    fn view_data(&self) -> Html<RootComponent> {
        if let Some(value) = &self.data {
            html! {
                <p>{ value.data }</p>
            }
        } else {
            html! {
                <p>{ "Data hasn't fetched yet." }</p>
            }
        }
    }
}
