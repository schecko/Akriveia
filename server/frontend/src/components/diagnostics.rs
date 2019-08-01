
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
use std::collections::{ VecDeque, BTreeSet };
use super::value_button::ValueButton;
use na;

const DIAGNOSTIC_POLLING_RATE: Duration = Duration::from_millis(1000);
const MAX_BUFFER_SIZE: usize = 0x50;

pub enum Msg {
    ClearBuffer,
    ToggleBeaconSelected(String),

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
    active_beacons: BTreeSet<String>,
    selected_beacons: BTreeSet<String>,
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
            active_beacons: BTreeSet::new(),
            selected_beacons: BTreeSet::new(),
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
            Msg::ToggleBeaconSelected(b_mac) => {
                if self.selected_beacons.contains(&b_mac) {
                    self.selected_beacons.remove(&b_mac);
                } else {
                    self.selected_beacons.insert(b_mac.clone());
                }
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
                                if !self.active_beacons.contains(&point.beacon_mac) {
                                    self.active_beacons.insert(point.beacon_mac.clone());
                                    self.selected_beacons.insert(point.beacon_mac.clone());
                                }
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
            let mut beacon_selections = self.active_beacons.iter().map(|b_mac| {
                let set_border = self.selected_beacons.contains(b_mac);
                html! {
                    <ValueButton
                        on_click=|value| Msg::ToggleBeaconSelected(value),
                        border=set_border,
                        value={b_mac.clone()}
                    />
                }
            });
            let filtered: Vec<common::TagData> = self.diagnostic_data.iter().filter(|point| self.selected_beacons.contains(&point.beacon_mac)).cloned().collect();
            html! {
                <>
                    <button
                        onclick=|_| Msg::ClearBuffer,
                    >
                        {"Reset Data"}
                    </button>
                    <div>
                        { "Select Beacons: " }
                        { for beacon_selections }
                    </div>
                    <table>
                    {
                        for filtered.iter().map(|row| {
                            match row.tag_distance {
                                common::DataType::RSSI(strength) => {
                                    html! {
                                        <tr>{ format!("beacon_mac: {}\tname: {}\tmac: {}\trssi: {}", &row.beacon_mac, &row.tag_name, &row.tag_mac, strength ) } </tr>
                                    }
                                },
                                common::DataType::TOF(distance) => {
                                    html! {
                                        <tr>{ format!("beacon_mac: {}\tname: {}\tmac: {}\ttof: {}", &row.beacon_mac, &row.tag_name, &row.tag_mac, distance ) } </tr>
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
