use common::*;
use crate::canvas::{ Canvas, /*screen_space*/ };
use crate::util::{ self, WebUserType, };
use std::time::Duration;
use stdweb::web::{ Node, html_element::ImageElement, };
use super::value_button::{ ValueButton, DisplayButton, };
use yew::format::Json;
use yew::services::fetch::{ FetchService, FetchTask, };
use yew::services::interval::{ IntervalService, IntervalTask, };
use yew::virtual_dom::vnode::VNode;
use yew::prelude::*;
use super::user_message::UserMessage;

const REALTIME_USER_POLL_RATE: Duration = Duration::from_millis(1000);

pub enum Msg {
    Ignore,
    ViewDistance(ShortAddress),
    ChooseMap(i32),

    RequestGetBeaconsForMap(i32),
    RequestGetMap(i32),
    RequestGetMaps,
    RequestRealtimeUser,

    ResponseGetBeaconsForMap(util::Response<Vec<Beacon>>),
    ResponseGetMap(util::Response<Option<Map>>),
    ResponseGetMaps(util::Response<Vec<Map>>),
    ResponseRealtimeUser(util::Response<Vec<RealtimeUserData>>),
}

pub struct MapViewComponent {
    beacons: Vec<Beacon>,
    canvas: Canvas,
    current_map: Option<Map>,
    emergency: bool,
    fetch_service: FetchService,
    fetch_task_beacons: Option<FetchTask>,
    fetch_task_realtime_users: Option<FetchTask>,
    get_fetch_task: Option<FetchTask>,
    get_many_fetch_task: Option<FetchTask>,
    interval_service: IntervalService,
    interval_service_task: Option<IntervalTask>,
    interval_service_task_beacon: Option<IntervalTask>,
    legend_canvas: Canvas,
    map_img: Option<ImageElement>,
    maps: Vec<Map>,
    realtime_users: Vec<RealtimeUserData>,
    self_link: ComponentLink<MapViewComponent>,
    show_distance: Option<ShortAddress>,
    user_msg: UserMessage<Self>,
    user_type: WebUserType,
}

impl MapViewComponent {
    fn start_service(&mut self) {
        self.interval_service_task = Some(
            self.interval_service.spawn(REALTIME_USER_POLL_RATE, self.self_link.send_back(|_| Msg::RequestRealtimeUser))
        );
        if let Some(map) = &self.current_map {
            let id = map.id;
            self.interval_service_task_beacon = Some(
                self.interval_service.spawn(REALTIME_USER_POLL_RATE, self.self_link.send_back(move |_| Msg::RequestGetBeaconsForMap(id)))
            );
        }
    }

    fn end_service(&mut self) {
        self.interval_service_task = None;
        self.interval_service_task_beacon = None;
    }

    fn render(&mut self) {
        if let Some(map) = &self.current_map {
            self.canvas.reset(map, &self.map_img);

            self.canvas.draw_users(map, &self.realtime_users, self.show_distance);
            if self.user_type == WebUserType::Admin {
                self.canvas.draw_beacons(map, &self.beacons.iter().collect());
            }
            self.legend_canvas.legend(80, map.bounds.y as u32, self.user_type);
        }
    }
}

#[derive(Properties)]
pub struct MapViewProps {
    pub emergency: bool,
    pub opt_id: Option<i32>,
    #[props(required)]
    pub user_type: WebUserType,
}

impl Component for MapViewComponent {
    type Message = Msg;
    type Properties = MapViewProps;

    fn create(props: Self::Properties, mut link: ComponentLink<Self>) -> Self {
        if let Some(id) = props.opt_id {
            link.send_self(Msg::RequestGetMap(id));
            link.send_self(Msg::RequestGetBeaconsForMap(id));
        }
        link.send_self(Msg::RequestGetMaps);
        let click_callback = link.send_back(|_event| Msg::Ignore);

        let mut result = MapViewComponent {
            beacons: Vec::new(),
            canvas: Canvas::new("map_canvas", click_callback.clone()),
            current_map: None,
            emergency: props.emergency,
            fetch_service: FetchService::new(),
            fetch_task_beacons: None,
            fetch_task_realtime_users: None,
            get_fetch_task: None,
            get_many_fetch_task: None,
            interval_service: IntervalService::new(),
            interval_service_task: None,
            interval_service_task_beacon: None,
            legend_canvas: Canvas::new("legend_canvas", click_callback),
            map_img: None,
            maps: Vec::new(),
            realtime_users: Vec::new(),
            self_link: link,
            show_distance: None,
            user_msg: UserMessage::new(),
            user_type: props.user_type,
        };

        if props.emergency {
            result.start_service();
        }

        result
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::ViewDistance(selected_tag_mac) => {
                match &self.show_distance {
                    Some(current_tag) => {
                        if current_tag == &selected_tag_mac {
                            self.show_distance = None;
                        } else {
                            self.show_distance = Some(selected_tag_mac);
                        }
                    },
                    None => {
                        self.show_distance = Some(selected_tag_mac);
                    }
                }
            },
            Msg::ChooseMap(id) => {
                self.current_map = match &self.current_map {
                    Some(curr) if curr.id == id => {
                        None
                    },
                    _ => {
                        match self.maps.iter().find(|map| map.id == id) {
                            Some(map) => {
                                let img = ImageElement::new();
                                img.set_src(&map_blueprint_url(&map.id.to_string()));
                                self.map_img = Some(img);
                                Some(map.clone())
                            },
                            None => None,
                        }
                    }
                };

                if let Some(map) = &self.current_map {
                    self.self_link.send_self(Msg::RequestGetMap(map.id));
                }

                if self.emergency {
                    self.start_service();
                }
            },
            Msg::RequestRealtimeUser => {
                self.user_msg.reset();
                self.fetch_task_realtime_users = get_request!(
                    self.fetch_service,
                    &users_status_url(),
                    self.self_link,
                    Msg::ResponseRealtimeUser
                );
            },
            Msg::RequestGetBeaconsForMap(id) => {
                self.user_msg.reset();
                self.fetch_task_beacons = get_request!(
                    self.fetch_service,
                    &beacons_for_map_url(&id.to_string()),
                    self.self_link,
                    Msg::ResponseGetBeaconsForMap
                );
            },
            Msg::RequestGetMap(id) => {
                self.user_msg.reset();
                self.get_fetch_task = get_request!(
                    self.fetch_service,
                    &map_url(&id.to_string()),
                    self.self_link,
                    Msg::ResponseGetMap
                );
            },
            Msg::RequestGetMaps => {
                self.user_msg.reset();
                self.get_many_fetch_task = get_request!(
                    self.fetch_service,
                    &maps_url(),
                    self.self_link,
                    Msg::ResponseGetMaps
                );
            },
            Msg::ResponseRealtimeUser(response) => {
                let (meta, Json(body)) = response.into_parts();
                if meta.status.is_success() {
                    match body {
                        Ok(result) => {
                            self.realtime_users = result;
                        },
                        Err(e) => {
                            self.user_msg.error_messages.push(format!("failed to request realtime user data, reason: {}", e));
                        }
                    }
                } else {
                    self.user_msg.error_messages.push("failed to request realtime user data, reason: {}".to_owned());
                }
            },
            Msg::ResponseGetBeaconsForMap(response) => {
                let (meta, Json(body)) = response.into_parts();
                if meta.status.is_success() {
                    match body {
                        Ok(result) => {
                            self.beacons = result;
                        },
                        Err(e) => {
                            self.user_msg.error_messages.push(format!("failed to obtain list of beacons for this map, reason: {}", e));
                        }
                    }
                } else {
                    self.user_msg.error_messages.push("failed to obtain list of beacons for this map".to_owned());
                }
            },
            Msg::ResponseGetMap(response) => {
                let (meta, Json(body)) = response.into_parts();
                if meta.status.is_success() {
                    match body {
                        Ok(result) => {
                            if let Some(map) = &result {
                                let img = ImageElement::new();
                                img.set_src(&map_blueprint_url(&map.id.to_string()));
                                self.map_img = Some(img);
                            }
                            self.current_map = result.clone();
                            result.and_then(|new_map_data| {
                                self.maps.iter_mut().find(|m| m.id == new_map_data.id).and_then(|map| {
                                    *map = new_map_data;
                                    Some(())
                                });
                                Some(())
                            });
                        },
                        Err(e) => {
                            self.user_msg.error_messages.push(format!("failed to get map, reason: {}", e));
                        }
                    }
                } else {
                    self.user_msg.error_messages.push("failed to get map".to_owned());
                }
            },
            Msg::ResponseGetMaps(response) => {
                let (meta, Json(body)) = response.into_parts();
                if meta.status.is_success() {
                    match body {
                        Ok(mut result) => {
                            result.sort_unstable_by(|a, b| a.name.cmp(&b.name));
                            self.maps = result;
                            if self.maps.len() > 0 {
                                self.self_link.send_self(Msg::ChooseMap(self.maps[0].id));
                            }
                        },
                        Err(e) => {
                            self.user_msg.error_messages.push(format!("failed to get maps, reason: {}", e));
                        }
                    }
                } else {
                    self.user_msg.error_messages.push("failed to get map".to_string());
                }
            },
            Msg::Ignore => {
            },
        }

        self.render();
        true
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        // do not overwrite the canvas or context.
        self.emergency = props.emergency;

        if self.emergency {
            self.start_service();
        } else {
            self.end_service();
            self.realtime_users = Vec::new();
        }
        self.render();
        true
    }
}

impl Renderable<MapViewComponent> for MapViewComponent {
    fn view(&self) -> Html<Self> {
        let current_map_id = match &self.current_map {
            Some(map) => map.id,
            None => -1,
        };

        let mut maps = self.maps.iter().map(|map| {
            let map_id = map.id;
            let map_name = map.name.clone();
            html! {
                <ValueButton<i32>
                    on_click=move |value: i32| Msg::ChooseMap(map_id),
                    border=map.id == current_map_id,
                    value={map.id},
                    display=Some(map_name),
                />
            }
        });

        let mut realtime_users = self.realtime_users.iter().map(|user| {
            let set_border = match &self.show_distance {
                Some(selected) => &user.addr == selected,
                None => false,
            };
            html! {
                <tr>
                    <td>{user.addr}</td>
                    <td>{&user.name}</td>
                    <td>{user.last_active}</td>
                    {
                        match self.user_type {
                            WebUserType::Admin => html! {
                                <DisplayButton<String>
                                    on_click=|value: String| Msg::ViewDistance(ShortAddress::parse_str(&value).unwrap()),
                                    border=set_border,
                                    value={user.addr.to_string()},
                                    display={"TOF"},
                                    style="btn-secondary"
                                />
                            },
                            _ => html! {}
                        }
                    }
                </tr>
            }
        });

        html! {
            <div>
                { self.user_msg.view() }
                <div>
                    <p>{ "Maps " }</p>
                    { for maps }
                </div>
                <table>
                    <tr>
                        <td>
                        { VNode::VRef(Node::from(self.legend_canvas.canvas.to_owned()).to_owned()) }
                        </td>
                        <td>
                        { VNode::VRef(Node::from(self.canvas.canvas.to_owned()).to_owned()) }
                        </td>
                        <td>
                            <table>
                                <tr>
                                    <td>{ "Address" }</td>
                                    <td>{ "Name" }</td>
                                    <td>{ "Last Seen" }</td>
                                    {
                                        match self.user_type {
                                            WebUserType::Admin => html! {
                                                <td>{ "Debug" }</td>
                                            },
                                            _ => html! {},
                                        }
                                    }
                                </tr>
                                { for realtime_users }
                            </table>
                        </td>
                    </tr>
                </table>
            </div>
        }
    }
}
