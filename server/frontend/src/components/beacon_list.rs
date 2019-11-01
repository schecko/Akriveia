use common::*;
use crate::util;
use super::root;
use super::value_button::{ ValueButton, DisplayButton, };
use yew::format::Json;
use yew::services::fetch::{ FetchService, FetchTask, };
use yew::prelude::*;

pub enum Msg {
    ChangeRootPage(root::Page),

    RequestDeleteBeacon(i32),
    RequestGetBeacons,
    RequestCommandBeacon(BeaconRequest),

    ResponseGetBeacons(util::Response<Vec<(Beacon, Map)>>),
    ResponseDeleteBeacon(util::Response<Vec<()>>),
    ResponseCommandBeacon(util::Response<()>),
}

struct Data {
    pub error_messages: Vec<String>,
    pub success_message: Option<String>,
}

pub struct BeaconList {
    change_page: Callback<root::Page>,
    fetch_service: FetchService,
    fetch_task: Option<FetchTask>,
    list: Vec<(Beacon, Map)>,
    self_link: ComponentLink<Self>,
    data: Data,
}

impl Data {
    fn new() -> Data {
        Data {
            error_messages: Vec::new(),
            success_message: None,
        }
    }
}

#[derive(Properties)]
pub struct BeaconListProps {
    #[props(required)]
    pub change_page: Callback<root::Page>,
}

impl Component for BeaconList {
    type Message = Msg;
    type Properties = BeaconListProps;

    fn create(props: Self::Properties, mut link: ComponentLink<Self>) -> Self {
        link.send_self(Msg::RequestGetBeacons);
        let result = BeaconList {
            fetch_service: FetchService::new(),
            list: Vec::new(),
            fetch_task: None,
            self_link: link,
            change_page: props.change_page,
            data: Data::new(),
        };
        result
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::RequestGetBeacons => {
                self.fetch_task = get_request!(
                    self.fetch_service,
                    &format!("{}?prefetch=true", beacons_url()),
                    self.self_link,
                    Msg::ResponseGetBeacons
                );
            },
            Msg::RequestCommandBeacon(command) => {
                Log!("wtf {:?}", command);
                self.fetch_task = post_request!(
                    self.fetch_service,
                    &beacon_command_url(),
                    command,
                    self.self_link,
                    Msg::ResponseCommandBeacon
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
                        Ok(beacons_and_maps) => {
                            self.list = beacons_and_maps;
                        }
                        _ => { }
                    }
                } else {
                    Log!("response - failed to obtain beacon list");
                }
            },
            Msg::ResponseCommandBeacon(response) => {
                let (meta, Json(_body)) = response.into_parts();
                if meta.status.is_success() {
                    self.data.success_message = Some("Successfully sent command".to_string());
                } else {
                    Log!("response - failed to command beacon");
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
                self.change_page.emit(page);
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
        let mut rows = self.list.iter().map(|(beacon, map)| {
            html! {
                <tr>
                    <td>{ &beacon.mac_address.to_hex_string() }</td>
                    <td>{ format!("{},{}", &beacon.coordinates.x, &beacon.coordinates.y) }</td>
                    <td>{ &map.name }</td>
                    <td>{ &beacon.name }</td>
                    <td>{ beacon.note.clone().unwrap_or(String::new()) }</td>
                    <td>
                        <ValueButton<i32>
                            display=Some("Edit".to_string()),
                            on_click=|value: i32| Msg::ChangeRootPage(root::Page::BeaconAddUpdate(Some(value))),
                            border=false,
                            value={beacon.id}
                        />
                        <ValueButton<i32>
                            display=Some("Delete".to_string()),
                            on_click=|value: i32| Msg::RequestDeleteBeacon(value),
                            border=false,
                            value=beacon.id
                        />
                        <DisplayButton<BeaconRequest>
                            display="Ping".to_string(),
                            on_click=|value| Msg::RequestCommandBeacon(value),
                            border=false,
                            value=BeaconRequest::Ping(Some(beacon.mac_address)),
                        />
                        <DisplayButton<BeaconRequest>
                            display="Reboot".to_string(),
                            on_click=|value| Msg::RequestCommandBeacon(value),
                            border=false,
                            value=BeaconRequest::Reboot(Some(beacon.mac_address)),
                        />
                    </td>
                </tr>
            }
        });

        let mut errors = self.data.error_messages.iter().cloned().map(|msg| {
            html! {
                <p>{msg}</p>
            }
        });

        html! {
            <>
                <p>{ "Beacon List" }</p>
                {
                    match &self.data.success_message {
                        Some(msg) => { format!("Success: {}", msg) },
                        None => { "".to_string() },
                    }
                }
                { if self.data.error_messages.len() > 0 { "Failure: " } else { "" } }
                { for errors }
                <table class="table table-striped">
                    <thead class="thead-light">
                        <h2>{ "Beacon List" }</h2>
                        <tr>
                            <th>{ "Mac" }</th>
                            <th>{ "Coordinates" }</th>
                            <th>{ "Floor" }</th>
                            <th>{ "Name" }</th>
                            <th>{ "Note" }</th>
                            <th>{ "Actions" }</th>
                        </tr>
                    </thead>
                    <tbody>
                        { for rows }
                    </tbody>
                </table>
            </>
        }
    }
}
