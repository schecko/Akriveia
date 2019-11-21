use common::*;
use crate::util;
use super::root;
use super::value_button::{ ValueButton, DisplayButton, };
use yew::format::Json;
use yew::services::fetch::{ FetchService, FetchTask, };
use yew::prelude::*;
use super::user_message::UserMessage;

pub enum Msg {
    ChangeRootPage(root::Page),

    RequestDeleteBeacon(i32),
    RequestGetBeacons,
    RequestCommandBeacon(BeaconRequest),

    ResponseGetBeacons(util::Response<Vec<(Beacon, Map)>>),
    ResponseDeleteBeacon(util::Response<Vec<()>>),
    ResponseCommandBeacon(util::Response<()>),
}
pub struct BeaconList {
    change_page: Callback<root::Page>,
    fetch_service: FetchService,
    fetch_task: Option<FetchTask>,
    list: Vec<(Beacon, Map)>,
    self_link: ComponentLink<Self>,
    user_msg: UserMessage<Self>,
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
            user_msg: UserMessage::new(),
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
                        Ok(mut beacons_and_maps) => {
                            beacons_and_maps.sort_unstable_by(|(beacon_a, _), (beacon_b, _)| beacon_a.name.cmp(&beacon_b.name));
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
                    self.user_msg.success_message = Some("Successfully sent command".to_owned());
                } else {
                    self.user_msg.error_messages.push("failed to command beacon".to_owned());
                }
            },
            Msg::ResponseDeleteBeacon(response) => {
                let (meta, Json(body)) = response.into_parts();
                if meta.status.is_success() {
                    match body {
                        Ok(_list) => {
                            self.user_msg.success_message = Some("Successfully deleted beacon".to_owned());
                        }
                        _ => { }
                    }
                } else {
                    self.user_msg.error_messages.push("failed to delete beacon".to_owned());
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

        html! {
            <>
                <p>{ "Beacon List" }</p>
                { self.user_msg.view() }
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
