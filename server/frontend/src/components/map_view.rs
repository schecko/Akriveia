use common::*;
use crate::canvas::{ Canvas, /*screen_space*/ };
use crate::util;
use std::time::Duration;
use stdweb::web::{ Node, html_element::ImageElement, };
use super::value_button::ValueButton;
use yew::format::Json;
use yew::services::fetch::{ FetchService, FetchTask, };
use yew::services::interval::{ IntervalService, IntervalTask, };
use yew::virtual_dom::vnode::VNode;
use yew::prelude::*;

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
    legend_canvas: Canvas,
    emergency: bool,
    error_messages: Vec<String>,
    fetch_service: FetchService,
    fetch_task_users: Option<FetchTask>,
    fetch_task_beacons: Option<FetchTask>,
    get_fetch_task: Option<FetchTask>,
    get_many_fetch_task: Option<FetchTask>,
    interval_service: IntervalService,
    interval_service_task: Option<IntervalTask>,
    interval_service_task_beacon: Option<IntervalTask>,
    current_map: Option<Map>,
    map_img: Option<ImageElement>,
    maps: Vec<Map>,
    self_link: ComponentLink<MapViewComponent>,
    show_distance: Option<ShortAddress>,
    users: Vec<RealtimeUserData>,
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
            self.canvas.draw_users(map, &self.users, self.show_distance);
            self.canvas.draw_beacons(map, &self.beacons.iter().collect());
            self.legend_canvas.legend(100, 600);
        }
    }
}

#[derive(Properties)]
pub struct MapViewProps {
    pub emergency: bool,
    pub opt_id: Option<i32>,
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
            legend_canvas: Canvas::new("legend_canvas", click_callback),
            emergency: props.emergency,
            error_messages: Vec::new(),
            fetch_service: FetchService::new(),
            fetch_task_users: None,
            fetch_task_beacons: None,
            get_fetch_task: None,
            get_many_fetch_task: None,
            interval_service: IntervalService::new(),
            interval_service_task: None,
            interval_service_task_beacon: None,
            maps: Vec::new(),
            current_map: None,
            map_img: None,
            self_link: link,
            show_distance: None,
            users: Vec::new(),
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
                self.fetch_task_users = get_request!(
                    self.fetch_service,
                    &users_status_url(),
                    self.self_link,
                    Msg::ResponseRealtimeUser
                );
            },
            Msg::RequestGetBeaconsForMap(id) => {
                self.error_messages = Vec::new();

                self.fetch_task_beacons = get_request!(
                    self.fetch_service,
                    &beacons_for_map_url(&id.to_string()),
                    self.self_link,
                    Msg::ResponseGetBeaconsForMap
                );
            },
            Msg::RequestGetMap(id) => {
                self.error_messages = Vec::new();

                self.get_fetch_task = get_request!(
                    self.fetch_service,
                    &map_url(&id.to_string()),
                    self.self_link,
                    Msg::ResponseGetMap
                );
            },
            Msg::RequestGetMaps => {
                self.error_messages = Vec::new();

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
                            self.users = result;
                        },
                        Err(e) => {
                            self.error_messages.push(format!("failed to obtain available floors list, reason: {}", e));
                        }
                    }
                } else {
                    Log!("response - failed to get realtime user data");
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
                            self.error_messages.push(format!("failed to obtain available floors list, reason: {}", e));
                        }
                    }
                } else {
                    self.error_messages.push("failed to obtain available floors list".to_string());
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
                            self.current_map = result;
                        },
                        Err(e) => {
                            self.error_messages.push(format!("failed to get map, reason: {}", e));
                        }
                    }
                } else {
                    self.error_messages.push("failed to get map".to_string());
                }
            },
            Msg::ResponseGetMaps(response) => {
                let (meta, Json(body)) = response.into_parts();
                if meta.status.is_success() {
                    match body {
                        Ok(result) => {
                            self.maps = result;
                        },
                        Err(e) => {
                            self.error_messages.push(format!("failed to get maps, reason: {}", e));
                        }
                    }
                } else {
                    self.error_messages.push("failed to get map".to_string());
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
            self.users = Vec::new();
        }
        self.render();
        true
    }
}

impl Renderable<MapViewComponent> for MapViewComponent {
    fn view(&self) -> Html<Self> {
        let mut render_distance_buttons = self.users.iter().map(|user| {
            let set_border = match &self.show_distance {
                Some(selected) => &user.addr == selected,
                None => false,
            };
            html! {
                <ValueButton<String>
                    on_click=|value: String| Msg::ViewDistance(ShortAddress::parse_str(&value).unwrap()),
                    border=set_border,
                    value={user.addr.to_string()}
                />
            }
        });

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

        let mut errors = self.error_messages.iter().cloned().map(|msg| {
            html! {
                <p>{msg}</p>
            }
        });

        html! {
            <div>
                { for errors }
                <div>
                    <p>{ "Maps " }</p>
                    { for maps }
                </div>
                <div>
                    <p>
                    {
                        if self.users.len() > 0 {
                            "View TOF: "
                        } else {
                            ""
                        }
                    }
                    </p>
                    { for render_distance_buttons }
                </div>
                <table>
                    <tr><td>
                    { VNode::VRef(Node::from(self.canvas.canvas.to_owned()).to_owned()) }
                    { VNode::VRef(Node::from(self.legend_canvas.canvas.to_owned()).to_owned()) }
                    </td></tr>
                </table>
            </div>
        }
    }
}
