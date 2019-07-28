
use stdweb::web::html_element::CanvasElement;
use stdweb::web::{ CanvasRenderingContext2d, Element, Node, FillRule };
use yew::services::fetch::{ FetchService, FetchTask, Request, Response, };
use yew::services::interval::{ IntervalTask, IntervalService, };
use yew::virtual_dom::vnode::VNode;
use yew::{ Component, ComponentLink, Html, Renderable, ShouldRender, html, };
use yew::services::console::ConsoleService;
use crate::util;
use std::time::Duration;
use yew::format::{ Nothing, Json };
use std::collections::VecDeque;
use super::value_button::ValueButton;
use na;

const DIAGNOSTIC_POLLING_RATE: Duration = Duration::from_millis(1000);
const MAX_BUFFER_SIZE: usize = 0x50;

pub enum Msg {
    ClearBuffer,

    RequestDiagnostics,

    ResponseDiagnostics(util::Response<common::DiagnosticData>),
}

pub struct Diagnostics {
    emergency: bool,
    diagnostic_data: VecDeque<common::TagData>,
    interval_service: Option<IntervalService>,
    interval_service_task: Option<IntervalTask>,
    fetch_service: FetchService,
    fetch_task: Option<FetchTask>,
    self_link: ComponentLink<Diagnostics>,
}

#[derive(Clone, Default, PartialEq)]
pub struct DiagnosticsProps {
    pub emergency: bool,
}

impl Diagnostics {
    fn start_service(&mut self) {
        let mut interval_service = IntervalService::new();
        self.interval_service_task = Some(interval_service.spawn(DIAGNOSTIC_POLLING_RATE, self.self_link.send_back(|_| Msg::RequestDiagnostics)));
        self.interval_service = Some(interval_service);
    }

    fn end_service(&mut self) {
        self.interval_service = None;
        self.interval_service_task = None;
    }
}

impl Component for Diagnostics {
    type Message = Msg;
    type Properties = DiagnosticsProps;

    fn create(props: Self::Properties, mut link: ComponentLink<Self>) -> Self {
        let mut result = Diagnostics {
            emergency: props.emergency,
            fetch_service: FetchService::new(),
            diagnostic_data: VecDeque::new(),
            fetch_task: None,
            interval_service: None,
            interval_service_task: None,
            self_link: link,
        };

        if result.emergency {
            result.start_service();
        }
        result
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::ClearBuffer => {
                self.diagnostic_data = VecDeque::new();
            },
            Msg::RequestDiagnostics => {
                self.fetch_task = get_request!(
                    self.fetch_service,
                    common::DIAGNOSTICS,
                    self.self_link,
                    Msg::ResponseDiagnostics
                );
            },

            Msg::ResponseDiagnostics(response) => {
                let (meta, Json(body)) = response.into_parts();
                if meta.status.is_success() {
                    match body {
                        Ok(common::DiagnosticData { mut tag_data }) => {
                            for point in tag_data.into_iter() {
                                self.diagnostic_data.push_front(point);
                            }
                            self.diagnostic_data.truncate(MAX_BUFFER_SIZE);
                        }
                        _ => { }
                    }
                } else {
                    Log!("response - failed to request diagnostics");
                }
            },
        }
        true
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.emergency = props.emergency;

        if self.emergency {
            self.start_service();
        } else {
            self.end_service();
        }

        true
    }
}

impl Renderable<Diagnostics> for Diagnostics {
    fn view(&self) -> Html<Self> {
        if self.diagnostic_data.len() > 0 {
            html! {
                <>
                    <button
                        onclick=|_| Msg::ClearBuffer,
                    >
                        {"Reset Data"}
                    </button>
                    <table>
                    {
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
                    }
                    </table>
                </>
            }
        } else {
            html! {
                <p>{ "No diagnostics yet..." }</p>
            }
        }
    }
}
