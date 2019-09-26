use common::*;
use crate::util;
use super::root;
use yew::format::Json;
use yew::services::fetch::{ FetchService, FetchTask, };
use yew::services::interval::{ IntervalTask, IntervalService, };
use yew::prelude::*;
use std::time::Duration;

const POLL_RATE: Duration = Duration::from_millis(1000);

pub enum PageState {
    BeaconStatus,
    UserStatus,
}

pub enum Msg {
    //ChangeRootPage(root::Page),
    ChangeStatus(PageState),

    RequestGetBeacons,
    RequestGetUsers,

    ResponseGetBeacons(util::Response<Vec<(Beacon, Map)>>),
    ResponseGetUsers(util::Response<Vec<(TrackedUser, Map)>>),
}

pub struct Status {
    state: PageState,
    _change_page: Callback<root::Page>,
    fetch_service: FetchService,
    fetch_task: Option<FetchTask>,
    interval_service: IntervalService,
    interval_service_task: Option<IntervalTask>,
    users: Vec<(TrackedUser, Map)>,
    beacons: Vec<(Beacon, Map)>,
    self_link: ComponentLink<Self>,
}

impl Status {
    fn restart_service(&mut self) {
        let callback = match self.state {
            PageState::UserStatus => self.self_link.send_back(|_| Msg::RequestGetUsers),
            PageState::BeaconStatus => self.self_link.send_back(|_| Msg::RequestGetBeacons),
        };
        self.interval_service_task = Some(self.interval_service.spawn(POLL_RATE, callback));
    }
}

#[derive(Properties)]
pub struct StatusProps {
    #[props(required)]
    pub change_page: Callback<root::Page>,
}

impl Component for Status {
    type Message = Msg;
    type Properties = StatusProps;

    fn create(props: Self::Properties, mut link: ComponentLink<Self>) -> Self {
        link.send_self(Msg::RequestGetBeacons);
        let mut result = Status {
            state: PageState::UserStatus,
            fetch_service: FetchService::new(),
            interval_service: IntervalService::new(),
            interval_service_task: None,
            users: Vec::new(),
            beacons: Vec::new(),
            fetch_task: None,
            self_link: link,
            _change_page: props.change_page,
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
            /*Msg::ChangeRootPage(page) => {
                self.change_page.emit(page);
            }*/
            Msg::RequestGetBeacons => {
                self.fetch_task = get_request!(
                    self.fetch_service,
                    &format!("{}?prefetch=true", beacons_url()),
                    self.self_link,
                    Msg::ResponseGetBeacons
                );
            },
            Msg::RequestGetUsers => {
                self.fetch_task = get_request!(
                    self.fetch_service,
                    &format!("{}?prefetch=true", users_url()),
                    self.self_link,
                    Msg::ResponseGetUsers
                );
            },
            Msg::ResponseGetBeacons(response) => {
                let (meta, Json(body)) = response.into_parts();
                if meta.status.is_success() {
                    match body {
                        Ok(beacons_and_maps) => {
                            self.beacons = beacons_and_maps;
                        }
                        _ => { }
                    }
                } else {
                    Log!("response - failed to obtain beacon list");
                }
            },
            Msg::ResponseGetUsers(response) => {
                let (meta, Json(body)) = response.into_parts();
                if meta.status.is_success() {
                    match body {
                        Ok(users_and_maps) => {
                            self.users = users_and_maps;
                        }
                        _ => { }
                    }
                } else {
                    Log!("response - failed to obtain beacon list");
                }
            },
        }
        true
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        true
    }
}

impl Status {
    fn beacon_table(&self) -> Html<Self> {
        let mut rows = self.beacons.iter().map(|(beacon, map)| {
            html! {
                <tr>
                    <td>{ &beacon.mac_address.to_hex_string() }</td>
                    <td>{ format!("{},{}", &beacon.coordinates.x, &beacon.coordinates.y) }</td>
                    <td>{ &map.name }</td>
                    <td>{ &beacon.name }</td>
                    <td>{ beacon.note.clone().unwrap_or(String::new()) }</td>
                </tr>
            }
        });

        html! {
            <>
                <tr>
                    <td>{ "Mac" }</td>
                    <td>{ "Coordinates" }</td>
                    <td>{ "Floor" }</td>
                    <td>{ "Name" }</td>
                    <td>{ "Note" }</td>
                    <td>{ "Actions" }</td>
                </tr>
                { for rows }
            </>
        }
    }

    fn user_table(&self) -> Html<Self> {
        let mut rows = self.users.iter().map(|(user, map)| {
            html! {
                <tr>
                    <td>{ &user.name }</td>
                    <td>{ format!("{},{}", &user.coordinates.x, &user.coordinates.y) }</td>
                    <td>{ &map.name }</td>
                </tr>
            }
        });

        html! {
            <>
                <tr>
                    <td>{ "Name" }</td>
                    <td>{ "Coordinates" }</td>
                    <td>{ "Floor" }</td>
                </tr>
                { for rows }
            </>
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
                <p>{ "Status" }</p>
                <button
                    onclick=|_| Msg::ChangeStatus(PageState::UserStatus),
                >
                    {"User Status"}
                </button>
                <button
                    onclick=|_| Msg::ChangeStatus(PageState::BeaconStatus),
                >
                    {"Beacon Status"}
                </button>
                <table>
                    { table }
                </table>
            </>

        }
    }
}
