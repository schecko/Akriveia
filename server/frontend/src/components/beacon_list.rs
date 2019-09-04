use yew::services::fetch::{ FetchService, FetchTask, };
//use yew::services::interval::{ IntervalTask, IntervalService, };
use yew::{ Callback, Component, ComponentLink, Html, Renderable, ShouldRender, html, };
use crate::util;
//use std::time::Duration;
use yew::format::Json;
use super::value_button::ValueButton;
use super::root;
use common::*;

pub enum Msg {
    ChangeRootPage(root::Page),

    RequestDeleteBeacon(i32),
    RequestGetBeacons,

    ResponseGetBeacons(util::Response<Vec<common::Beacon>>),
    ResponseDeleteBeacon(util::Response<Vec<common::Beacon>>),
}

pub struct BeaconList {
    list: Vec<common::Beacon>,
    //interval_service: Option<IntervalService>,
    //interval_service_task: Option<IntervalTask>,
    fetch_service: FetchService,
    fetch_task: Option<FetchTask>,
    self_link: ComponentLink<Self>,
    change_page: Option<Callback<root::Page>>,
}

#[derive(Clone, Default, PartialEq)]
pub struct BeaconListProps {
    pub change_page: Option<Callback<root::Page>>,
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

    fn create(props: Self::Properties, mut link: ComponentLink<Self>) -> Self {
        link.send_self(Msg::RequestGetBeacons);
        let result = BeaconList {
            fetch_service: FetchService::new(),
            list: Vec::new(),
            fetch_task: None,
            //interval_service: None,
            //interval_service_task: None,
            self_link: link,
            change_page: props.change_page,
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
            Msg::RequestDeleteBeacon(id) => {
                self.fetch_task = delete_request!(
                    self.fetch_service,
                    &beacon_url(&id.to_string()),
                    self.self_link,
                    Msg::ResponseDeleteBeacon
                );
            },
            Msg::ResponseGetBeacons(response) => {
                let (meta, Json(body)) = response.into_parts();
                if meta.status.is_success() {
                    match body {
                        Ok(list) => {
                            Log!("list is: {:?}", list);
                            self.list = list;
                        }
                        _ => { }
                    }
                } else {
                    Log!("response - failed to obtain beacon list");
                }
            },
            Msg::ResponseDeleteBeacon(response) => {
                let (meta, Json(body)) = response.into_parts();
                if meta.status.is_success() {
                    match body {
                        Ok(_list) => {
                            Log!("successfully deleted beacon");
                        }
                        _ => { }
                    }
                } else {
                    Log!("response - failed to delete beacon");
                }
                // now that the beacon is deleted, get the updated list
                self.self_link.send_self(Msg::RequestGetBeacons);
            },
            Msg::ChangeRootPage(page) => {
                self.change_page.as_mut().unwrap().emit(page);
            }
        }
        true
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        self.self_link.send_self(Msg::RequestGetBeacons);
        true
    }
}

impl Renderable<BeaconList> for BeaconList {
    fn view(&self) -> Html<Self> {
        let mut rows = self.list.iter().map(|row| {
            let map_id = match &row.map_id {
                Some(id) => id,
                None => "",
            };

            html! {
                <tr>
                    <td>{ &row.mac_address.to_hex_string() }</td>
                    <td>{ format!("{},{}", &row.coordinates.x, &row.coordinates.y) }</td>
                    <td>{ map_id }</td>
                    <td>{ &row.name }</td>
                    <td>{ &row.note }</td>
                    <td>
                        <ValueButton<i32>
                            display=Some("Edit".to_string()),
                            on_click=|value: i32| Msg::ChangeRootPage(root::Page::BeaconAddUpdate(Some(value))),
                            border=false,
                            value={row.id}
                        />
                        <ValueButton<i32>
                            display=Some("Delete".to_string()),
                            on_click=|value: i32| Msg::RequestDeleteBeacon(value),
                            border=false,
                            value={row.id}
                        />
                    </td>
                </tr>
            }
        });

        html! {
            <>
                <p>{ "Beacon List" }</p>
                <table>
                <tr>
                    <td>{ "Mac" }</td>
                    <td>{ "Coordinates" }</td>
                    <td>{ "Floor" }</td>
                    <td>{ "Name" }</td>
                    <td>{ "Note" }</td>
                    <td>{ "Actions" }</td>
                </tr>
                { for rows }
                </table>
            </>
        }
    }
}
