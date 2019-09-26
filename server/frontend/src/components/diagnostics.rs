use common::*;
use crate::util;
use std::collections::{ VecDeque, BTreeSet };
use std::time::Duration;
use super::value_button::ValueButton;
use yew::format::Json;
use yew::services::fetch::{ FetchService, FetchTask, };
use yew::services::interval::{ IntervalTask, IntervalService, };
use yew::prelude::*;

const DIAGNOSTIC_POLLING_RATE: Duration = Duration::from_millis(1000);
const MAX_BUFFER_SIZE: usize = 0x50;

pub enum Msg {
    ClearBuffer,
    ToggleBeaconSelected(MacAddress),

    RequestDiagnostics,

    ResponseDiagnostics(util::Response<common::DiagnosticData>),
}

pub struct Diagnostics {
    active_beacons: BTreeSet<MacAddress>,
    diagnostic_data: VecDeque<common::TagData>,
    emergency: bool,
    fetch_service: FetchService,
    fetch_task: Option<FetchTask>,
    interval_service: Option<IntervalService>,
    interval_service_task: Option<IntervalTask>,
    selected_beacons: BTreeSet<MacAddress>,
    self_link: ComponentLink<Diagnostics>,
}

#[derive(Properties)]
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

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let mut result = Diagnostics {
            active_beacons: BTreeSet::new(),
            diagnostic_data: VecDeque::new(),
            emergency: props.emergency,
            fetch_service: FetchService::new(),
            fetch_task: None,
            interval_service: None,
            interval_service_task: None,
            selected_beacons: BTreeSet::new(),
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
                    &system_diagnostics_url(),
                    self.self_link,
                    Msg::ResponseDiagnostics
                );
            },
            Msg::ResponseDiagnostics(response) => {
                let (meta, Json(body)) = response.into_parts();
                if meta.status.is_success() {
                    match body {
                        Ok(common::DiagnosticData { tag_data }) => {
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
            self.diagnostic_data = VecDeque::new();
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
                    <ValueButton<String>
                        on_click=|value: String| Msg::ToggleBeaconSelected(MacAddress::parse_str(&value).unwrap()),
                        border=set_border,
                        value={b_mac.to_hex_string()}
                    />
                }
            });
            let filtered: Vec<common::TagData> = self.diagnostic_data.iter().filter(|point| self.selected_beacons.contains(&point.beacon_mac)).cloned().collect();

            let mut diagnostic_rows = filtered.iter().map(|row| {
                html! {
                    <tr>
                        <td>{ &row.beacon_mac }</td>
                        <td>{ &row.tag_mac }</td>
                        <td>{ &row.tag_distance }</td>
                    </tr>
                }
            });

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
                        <tr>
                            <td>{"Beacon Mac" }</td>
                            <td>{"User Mac" }</td>
                            <td>{"Distance" }</td>
                        </tr>
                        { for diagnostic_rows }
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
