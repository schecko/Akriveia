use yew::services::fetch::{ FetchService, FetchTask, Request, };
//use yew::services::interval::{ IntervalTask, IntervalService, };
use yew::{ Component, ComponentLink, Html, Renderable, ShouldRender, html, };
use crate::util;
//use std::time::Duration;
use yew::format::{ Nothing, Json };
use common::*;

pub enum Msg {
    RequestGetBeacons,

    ResponseGetBeacons(util::Response<Vec<common::Beacon>>),
}

pub struct BeaconList {
    list: Vec<common::Beacon>,
    //interval_service: Option<IntervalService>,
    //interval_service_task: Option<IntervalTask>,
    fetch_service: FetchService,
    fetch_task: Option<FetchTask>,
    self_link: ComponentLink<Self>,
}

#[derive(Clone, Default, PartialEq)]
pub struct BeaconListProps {
}

/*impl BeaconList {
    fn start_service(&mut self) {
        let mut interval_service = IntervalService::new();
        self.interval_service_task = Some(interval_service.spawn(DIAGNOSTIC_POLLING_RATE, self.self_link.send_back(|_| Msg::RequestDiagnostics)));
        self.interval_service = Some(interval_service);
    }

    fn end_service(&mut self) {
        self.interval_service = None;
        self.interval_service_task = None;
    }
}*/

impl Component for BeaconList {
    type Message = Msg;
    type Properties = BeaconListProps;

    fn create(_props: Self::Properties, mut link: ComponentLink<Self>) -> Self {
        link.send_self(Msg::RequestGetBeacons);
        let result = BeaconList {
            fetch_service: FetchService::new(),
            list: Vec::new(),
            fetch_task: None,
            //interval_service: None,
            //interval_service_task: None,
            self_link: link,
        };
        result
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::RequestGetBeacons => {
                self.fetch_task = get_request!(
                    self.fetch_service,
                    &beacons_url(),
                    self.self_link,
                    Msg::ResponseGetBeacons
                );
            },
            Msg::ResponseGetBeacons(response) => {
                let (meta, Json(body)) = response.into_parts();
                if meta.status.is_success() {
                    match body {
                        Ok(list) => {
                            self.list = list;
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

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        true
    }
}

impl Renderable<BeaconList> for BeaconList {
    fn view(&self) -> Html<Self> {
        if self.list.len() > 0 {
            html! {
                <>
                    <table>
                    <tr>
                        <td>{ "mac" }</td>
                        <td>{ "location" }</td>
                        <td>{ "floor" }</td>
                        <td>{ "name" }</td>
                        <td>{ "note" }</td>
                    </tr>
                    {
                        for self.list.iter().map(|row| {
                            let map_id = match &row.map_id {
                                Some(id) => id,
                                None => "",
                            };

                            html! {
                            <tr>
                                <td>{ &row.mac_address.to_hex_string() }</td>
                                <td>{ format!("{} {}", &row.coordinates.x, &row.coordinates.y) }</td>
                                <td>{ map_id }</td>
                                <td>{ &row.name }</td>
                                <td>{ &row.note }</td>
                            </tr>
                            }
                        })
                    }
                    </table>
                </>
            }
        } else {
            html! {
                <p>{ "no beacons" }</p>
            }
        }
    }
}
