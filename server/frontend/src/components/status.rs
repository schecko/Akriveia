use common::*;
use crate::util::{ self, WebUserType, };
use std::collections::HashMap;
use std::time::Duration;
use super::root;
use super::value_button::{ ValueButton, DisplayButton, };
use yew::format::Json;
use yew::prelude::*;
use yew::services::fetch::{ FetchService, FetchTask, };
use yew::services::interval::{ IntervalTask, IntervalService, };
use super::user_message::UserMessage;

const POLL_RATE: Duration = Duration::from_millis(1000);

#[derive(PartialEq, Copy, Clone)]
pub enum PageState {
    BeaconStatus,
    UserStatus,
}

pub enum Msg {
    ChangeRootPage(root::Page),
    ChangeStatus(PageState),

    RequestCommandBeacon(BeaconRequest),
    RequestGetBeacons,
    RequestGetBeaconsStatus,
    RequestGetMap(i32),
    RequestGetMaps,
    RequestGetUser(i32),
    RequestGetUsers,
    RequestGetUsersStatus,

    ResponseCommandBeacon(util::Response<()>),
    ResponseGetBeacons(util::Response<Vec<Beacon>>),
    ResponseGetBeaconsStatus(util::Response<Vec<RealtimeBeacon>>),
    ResponseGetMap(util::Response<Map>),
    ResponseGetMaps(util::Response<Vec<Map>>),
    ResponseGetUser(util::Response<TrackedUser>),
    ResponseGetUsers(util::Response<Vec<TrackedUser>>),
    ResponseGetUsersStatus(util::Response<Vec<RealtimeUserData>>),
}

pub struct Status {
    beacons: HashMap<i32, Beacon>,
    change_page: Callback<root::Page>,
    fetch_service: FetchService,
    interval_service: IntervalService,
    interval_service_task: Option<IntervalTask>,
    maps: HashMap<i32, Map>,
    self_link: ComponentLink<Self>,
    state: PageState,
    user_msg: UserMessage<Self>,
    user_type: WebUserType,
    users: HashMap<i32, TrackedUser>,

    // ugh.
    fetch_commands: Option<FetchTask>,
    fetch_beacons: Option<FetchTask>,
    fetch_beacons_status: Option<FetchTask>,
    fetch_users: Option<FetchTask>,
    fetch_user: Option<FetchTask>,
    fetch_user_status: Option<FetchTask>,
    fetch_maps: Option<FetchTask>,
    fetch_map: Option<FetchTask>,
}

impl Status {
    fn restart_service(&mut self) {
        let callback = match self.state {
            PageState::UserStatus => self.self_link.send_back(|_| Msg::RequestGetUsersStatus),
            PageState::BeaconStatus => self.self_link.send_back(|_| Msg::RequestGetBeaconsStatus),
        };
        self.interval_service_task = Some(self.interval_service.spawn(POLL_RATE, callback));
    }
}

#[derive(Properties)]
pub struct StatusProps {
    #[props(required)]
    pub change_page: Callback<root::Page>,
    #[props(required)]
    pub state: PageState,
    #[props(required)]
    pub user_type: WebUserType,
}

impl Component for Status {
    type Message = Msg;
    type Properties = StatusProps;

    fn create(props: Self::Properties, mut link: ComponentLink<Self>) -> Self {
        link.send_self(Msg::RequestGetBeacons);
        link.send_self(Msg::RequestGetUsers);
        link.send_self(Msg::RequestGetMaps);
        let mut result = Status {
            beacons: HashMap::new(),
            change_page: props.change_page,
            fetch_service: FetchService::new(),
            interval_service: IntervalService::new(),
            interval_service_task: None,
            maps: HashMap::new(),
            self_link: link,
            state: props.state,
            user_msg: UserMessage::new(),
            user_type: props.user_type,
            users: HashMap::new(),

            fetch_beacons: None,
            fetch_commands: None,
            fetch_beacons_status: None,
            fetch_map: None,
            fetch_maps: None,
            fetch_user: None,
            fetch_user_status: None,
            fetch_users: None,
        };

        result.restart_service();
        result
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::ChangeStatus(state) => {
                self.state = state;
                self.restart_service();
            }
            Msg::ChangeRootPage(page) => {
                self.change_page.emit(page);
            }
            Msg::RequestGetMaps => {
                self.user_msg.reset();
                self.fetch_maps = get_request!(
                    self.fetch_service,
                    &maps_url(),
                    self.self_link,
                    Msg::ResponseGetMaps
                );
            },
            Msg::RequestGetMap(id) => {
                self.user_msg.reset();
                self.fetch_map = get_request!(
                    self.fetch_service,
                    &map_url(&id.to_string()),
                    self.self_link,
                    Msg::ResponseGetMap
                );
            },
            Msg::RequestGetBeacons => {
                self.user_msg.reset();
                self.fetch_beacons = get_request!(
                    self.fetch_service,
                    &beacons_url(),
                    self.self_link,
                    Msg::ResponseGetBeacons
                );
            },
            Msg::RequestGetBeaconsStatus => {
                self.user_msg.reset();
                self.fetch_beacons_status = get_request!(
                    self.fetch_service,
                    &beacons_status_url(),
                    self.self_link,
                    Msg::ResponseGetBeaconsStatus
                );
            },
            Msg::RequestCommandBeacon(command) => {
                self.user_msg.reset();
                self.fetch_commands = post_request!(
                    self.fetch_service,
                    &beacon_command_url(),
                    command,
                    self.self_link,
                    Msg::ResponseCommandBeacon
                );
            },
            Msg::RequestGetUsers => {
                self.user_msg.reset();
                self.fetch_users = get_request!(
                    self.fetch_service,
                    &users_url(),
                    self.self_link,
                    Msg::ResponseGetUsers
                );
            },
            Msg::RequestGetUsersStatus => {
                self.user_msg.reset();
                self.fetch_user_status = get_request!(
                    self.fetch_service,
                    &users_status_url(),
                    self.self_link,
                    Msg::ResponseGetUsersStatus
                );
            },
            Msg::RequestGetUser(id) => {
                self.user_msg.reset();
                self.fetch_user = get_request!(
                    self.fetch_service,
                    &user_url(&id.to_string()),
                    self.self_link,
                    Msg::ResponseGetUser
                );
            },
            Msg::ResponseGetMaps(response) => {
                let (meta, Json(body)) = response.into_parts();
                if meta.status.is_success() {
                    match body {
                        Ok(maps) => {
                            for map in maps {
                                self.maps.insert(map.id, map);
                            }
                        }
                        _ => { }
                    }
                } else {
                    self.user_msg.error_messages.push("failed to obtain map list".to_owned());
                }
            },
            Msg::ResponseGetMap(response) => {
                let (meta, Json(body)) = response.into_parts();
                if meta.status.is_success() {
                    match body {
                        Ok(map) => {
                            self.maps.insert(map.id, map);
                        }
                        _ => { }
                    }
                } else {
                    self.user_msg.error_messages.push("failed to get map".to_owned());
                }
            },
            Msg::ResponseGetBeacons(response) => {
                let (meta, Json(body)) = response.into_parts();
                if meta.status.is_success() {
                    match body {
                        Ok(beacons) => {
                            for b in beacons {
                                if let Some(mid) = b.map_id {
                                    if !self.maps.contains_key(&mid) {
                                        let mid = mid.clone();
                                        self.self_link.send_back(move |_: ()| Msg::RequestGetMap(mid));
                                    }
                                }
                                self.beacons.insert(b.id, b);
                            }
                        }
                        _ => { }
                    }
                } else {
                    self.user_msg.error_messages.push("failed to obtain beacon list".to_owned());
                }
            },
            Msg::ResponseGetBeaconsStatus(response) => {
                let (meta, Json(body)) = response.into_parts();
                if meta.status.is_success() {
                    match body {
                        Ok(realtime_beacons) => {
                            for rb in realtime_beacons {
                                match self.beacons.get_mut(&rb.id) {
                                    Some(b) => {
                                        b.merge(rb);
                                    },
                                    None => {
                                        // just drop the realtime data for now until
                                        // the user object is retrieved, more realtime data
                                        // will come eventually and the UI user likely wont
                                        // notice.
                                        self.self_link
                                            .send_back(move |_: ()| Msg::RequestGetUser(rb.id));
                                    }
                                }
                            }
                        }
                        _ => { }
                    }
                } else {
                    self.user_msg.error_messages.push("failed to get beacon status".to_owned());
                }
            },
            Msg::ResponseCommandBeacon(response) => {
                let (meta, Json(_body)) = response.into_parts();
                if meta.status.is_success() {
                    self.user_msg.success_message = Some("Successfully sent command".to_string());
                } else {
                    self.user_msg.error_messages.push("failed to send command".to_owned());
                }
            },
            Msg::ResponseGetUser(response) => {
                let (meta, Json(body)) = response.into_parts();
                if meta.status.is_success() {
                    match body {
                        Ok(user) => {
                            if let Some(mid) = user.map_id {
                                if !self.maps.contains_key(&mid) {
                                    let mid = mid.clone();
                                    self.self_link.send_back(move |_: ()| Msg::RequestGetMap(mid));
                                }
                            }
                            self.users.insert(user.id, user);
                        }
                        _ => { }
                    }
                } else {
                    self.user_msg.error_messages.push("failed to get user".to_owned());
                }
            },
            Msg::ResponseGetUsers(response) => {
                let (meta, Json(body)) = response.into_parts();
                if meta.status.is_success() {
                    match body {
                        Ok(users) => {
                            for user in users {
                                if let Some(mid) = user.map_id {
                                    if !self.maps.contains_key(&mid) {
                                        let mid = mid.clone();
                                        self.self_link.send_back(move |_: ()| Msg::RequestGetMap(mid));
                                    }
                                }
                                self.users.insert(user.id, user);
                            }
                        }
                        _ => { }
                    }
                } else {
                    self.user_msg.error_messages.push("failed to get user list".to_owned());
                }
            },
            Msg::ResponseGetUsersStatus(response) => {
                let (meta, Json(body)) = response.into_parts();
                if meta.status.is_success() {
                    match body {
                        Ok(realtime_users) => {
                            for ru in realtime_users {
                                match self.users.get_mut(&ru.id) {
                                    Some(u) => {
                                        u.merge(ru);
                                    },
                                    None => {
                                        // just drop the realtime data for now until
                                        // the user object is retrieved, more realtime data
                                        // will come eventually and the UI user likely wont
                                        // notice.
                                        self.self_link
                                            .send_back(move |_: ()| Msg::RequestGetUser(ru.id));
                                    }
                                }
                            }
                        }
                        _ => { }
                    }
                } else {
                    self.user_msg.error_messages.push("failed to get user status".to_owned());
                }
            },
        }
        true
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        if self.state != props.state {
            self.self_link.send_self(Msg::ChangeStatus(props.state));
        }
        true
    }
}

lazy_static! {
    static ref DEFAULT_MAP: Map = Map::new();
}

impl Status {

    fn beacon_table(&self) -> Html<Self> {
        let mut rows = self.beacons.iter().map(|(_id, beacon)| {
            let (map, valid_map) = match beacon.map_id {
                Some(mid) => {
                    match self.maps.get(&mid) {
                        Some(map) => (map, true),
                        None => {
                            (&*DEFAULT_MAP, true) // render default map until the correct one loads
                        }
                    }
                },
                None => {
                    (&*DEFAULT_MAP, false) // this beacon doesnt have a map
                }
            };

            let command_buttons = match self.user_type {
                WebUserType::Admin => html! {
                    <>
                        <DisplayButton<BeaconRequest>
                            display="Ping".to_owned(),
                            on_click=|value| Msg::RequestCommandBeacon(value),
                            border=false,
                            value=BeaconRequest::Ping(Some(beacon.mac_address)),
                            style="btn-secondary",
                        />
                        <DisplayButton<BeaconRequest>
                            display="Reboot".to_owned(),
                            on_click=|value| Msg::RequestCommandBeacon(value),
                            border=false,
                            value=BeaconRequest::Reboot(Some(beacon.mac_address)),
                            style="btn-secondary",
                        />
                    </>
                },
                _ => html! {},
            };

            html! {
                <tr>
                    <td>{ &beacon.name }</td>
                    <td>{ &beacon.state }</td>
                    <td>{ &beacon.last_active }</td>
                    <td>{ format!("{:.3},{:.3}", &beacon.coordinates.x, &beacon.coordinates.y) }</td>
                    <td>{ &map.name }</td>
                    <td>{ &beacon.mac_address.to_hex_string() }</td>
                    <td>{ beacon.note.as_ref().unwrap_or(&String::new()) }</td>
                    <td>
                        <ValueButton<i32>
                            display=Some("Details".to_owned()),
                            on_click=|value: i32| Msg::ChangeRootPage(root::Page::BeaconAddUpdate(Some(value))),
                            border=false,
                            value={beacon.id}
                        />
                        <DisplayButton<Option<i32>>
                            display="Map".to_owned(),
                            on_click=|opt_map_id: Option<i32>| Msg::ChangeRootPage(root::Page::MapView(opt_map_id)),
                            border=false,
                            disabled=!valid_map,
                            value={beacon.map_id},
                            style="btn-primary",
                        />
                        { command_buttons }
                    </td>
                </tr>
            }
        });

        html! {
            // TODO find the reason why table is not container-fluid
            <div class="container-fluid">
                <table class="table table-striped">
                    <thead class="thead-light">
                        <h2>{ "Beacon Status" }</h2>
                        <tr>
                            <th>{ "Name" }</th>
                            <th>{ "State" }</th>
                            <th>{ "Last Active" }</th>
                            <th>{ "Coordinates" }</th>
                            <th>{ "Floor" }</th>
                            <th>{ "Mac" }</th>
                            <th>{ "Note" }</th>
                            <th>{ "Actions" }</th>
                        </tr>
                    </thead>
                    <tbody>
                        { for rows }
                    </tbody>
                </table>
            </div>
        }
    }

    fn user_table(&self) -> Html<Self> {
        let mut rows = self.users.iter().map(|(_id, user)| {
            let (map, valid_map) = match user.map_id {
                Some(mid) => {
                    match self.maps.get(&mid) {
                        Some(map) => (map, true),
                        None => {
                            (&*DEFAULT_MAP, false) // render default map until the correct one loads
                        }
                    }
                },
                None => {
                    (&*DEFAULT_MAP, false) // this user doesnt have a map
                }
            };

            html! {
                <tr>
                    <td>{ &user.name }</td>
                    <td>{ format!("{:.3},{:.3}", &user.coordinates.x, &user.coordinates.y) }</td>
                    <td>{ &map.name }</td>
                    <td>{ &user.last_active }</td>
                    <td>{ &user.note.as_ref().unwrap_or(&String::new()) }</td>
                    <td>
                        <ValueButton<i32>
                            display=Some("Details".to_string()),
                            on_click=|value: i32| Msg::ChangeRootPage(root::Page::UserAddUpdate(Some(value))),
                            border=false,
                            value={user.id}
                        />
                        <DisplayButton<Option<i32>>
                            display="Map".to_string(),
                            on_click=|opt_map_id: Option<i32>| Msg::ChangeRootPage(root::Page::MapView(opt_map_id)),
                            border=false,
                            disabled=!valid_map,
                            value={user.map_id},
                            style="btn-secondary",
                        />
                    </td>
                </tr>
            }
        });

        html! {
            <div class="container-fluid">
                <table class="table table-striped">
                    <thead class="thead-light">
                        <h2>{ "User Status" }</h2>
                        <tr>
                            <th>{ "Name" }</th>
                            <th>{ "Coordinates" }</th>
                            <th>{ "Floor" }</th>
                            <th>{ "Last Seen" }</th>
                            <th>{ "Note" }</th>
                            <th>{ "Actions" }</th>
                        </tr>
                    </thead>
                    <tbody>
                        { for rows }
                    </tbody>
                </table>
            </div>
        }
    }
}

impl Renderable<Status> for Status {
    fn view(&self) -> Html<Self> {
        let table = match self.state {
            PageState::BeaconStatus => self.beacon_table(),
            PageState::UserStatus => self.user_table(),
        };

        html! {
            <>
                { self.user_msg.view() }
                <table>
                    { table }
                </table>
            </>

        }
    }
}
