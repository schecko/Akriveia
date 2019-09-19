use common::*;
use crate::util;
use crate::canvas::{ Canvas, screen_space };
use na;
use std::collections::BTreeMap;
use std::time::Duration;
use stdweb::web::html_element::CanvasElement;
use stdweb::web::{ CanvasRenderingContext2d, Node, FillRule };
use super::value_button::ValueButton;
use yew::format::Json;
use yew::services::fetch::{ FetchService, FetchTask, };
use yew::services::interval::{ IntervalService, IntervalTask, };
use yew::virtual_dom::vnode::VNode;
use yew::{ Component, ComponentLink, Html, Renderable, ShouldRender, html, };

const REALTIME_USER_POLL_RATE: Duration = Duration::from_millis(1000);

pub enum Msg {
    Ignore,
    RenderMap,
    ViewDistance(MacAddress),

    RequestRealtimeUser,
    RequestGetBeaconsForMap,
    RequestGetMap,

    ResponseRealtimeUser(util::Response<Vec<TrackedUser>>),
    ResponseGetBeaconsForMap(util::Response<Vec<Beacon>>),
    ResponseGetMap(util::Response<Option<Map>>),
}

pub struct MapViewComponent {
    beacons: Vec<Beacon>,
    canvas: Canvas,
    emergency: bool,
    fetch_service: FetchService,
    fetch_task: Option<FetchTask>,
    interval_service: Option<IntervalService>,
    interval_service_task: Option<IntervalTask>,
    map_id: i32,
    opt_map: Option<Map>,
    self_link: ComponentLink<MapViewComponent>,
    show_distance: Option<MacAddress>,
    users: Vec<TrackedUser>,
}

impl MapViewComponent {
    fn start_service(&mut self) {
        let mut interval_service = IntervalService::new();
        self.interval_service_task = Some(
            interval_service.spawn(REALTIME_USER_POLL_RATE, self.self_link.send_back(|_| Msg::RequestRealtimeUser))
        );
        self.interval_service = Some(interval_service);
    }

    fn end_service(&mut self) {
        self.interval_service = None;
        self.interval_service_task = None;
    }
}

#[derive(Clone, Default, PartialEq)]
pub struct MapViewProps {
    pub emergency: bool,
    pub id: i32,
}

impl Component for MapViewComponent {
    type Message = Msg;
    type Properties = MapViewProps;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        link.send_self(Msg::RequestGetMap(props.id));
        link.send_self(Msg::RequestGetBeaconsForMap(props.id));

        let click_callback = link.send_back(|_event| Msg::Ignore);
        let mut result = MapViewComponent {
            beacons: Vec::new(),
            canvas: Canvas::new("map_canvas", click_callback),
            emergency: props.emergency,
            fetch_service: FetchService::new(),
            fetch_task: None,
            id: props.id,
            interval_service: None,
            interval_service_task: None,
            opt_map: None,
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
            Msg::RenderMap => {
                self.canvas.reset();
                if let Some(map) = self.opt_map {
                    self.canvas.draw_users(map, &self.users, self.show_distance);
                    self.canvas.draw_beacons(map, &self.beacons);
                }
            },
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
            Msg::RequestRealtimeUser => {
                self.fetch_task = get_request!(
                    self.fetch_service,
                    &users_realtime_url(),
                    self.self_link,
                    Msg::ResponseRealtimeUser
                );
            },
            Msg::RequestGetBeaconsForMap(id) => {
                self.fetch_task = get_request!(
                    self.fetch_service,
                    &beacons_for_map_url(&id.to_string()),
                    self.self_link,
                    Msg::ResponseGetBeaconsForMap
                );
            },
            Msg::RequestGetMap(id) => {
                self.get_fetch_task = get_request!(
                    self.fetch_service,
                    &map_url(&id.to_string()),
                    self.self_link,
                    Msg::ResponseGetMap
                );
            },
            Msg::ResponseRealtimeUser(response) => {
                self.clear_map();
                let (meta, Json(body)) = response.into_parts();
                if meta.status.is_success() {
                    match body {
                        Ok(result) => {
                            self.users = result;
                        },
                        Err(e) => {
                            self.data.error_messages.push(format!("failed to obtain available floors list, reason: {}", e));
                        }
                    }
                } else {
                    Log!("response - failed to get realtime user data");
                }

                self.self_link.send_self(Msg::RenderMap);
            },
            Msg::ResponseGetBeaconsForMap(response) => {
                let (meta, Json(body)) = response.into_parts();
                if meta.status.is_success() {
                    match body {
                        Ok(result) => {
                            self.beacons = result;
                        },
                        Err(e) => {
                            self.data.error_messages.push(format!("failed to obtain available floors list, reason: {}", e));
                        }
                    }
                } else {
                    self.data.error_messages.push("failed to obtain available floors list".to_string());
                }
            },
            Msg::ResponseGetMap(response) => {
                let (meta, Json(body)) = response.into_parts();
                if meta.status.is_success() {
                    match body {
                        Ok(result) => {
                            self.map = result;
                        },
                        Err(e) => {
                            self.data.error_messages.push(format!("failed to find map, reason: {}", e));
                        }
                    }
                } else {
                    self.data.error_messages.push("failed to find map".to_string());
                }
            },
        }
        true
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        // do not overwrite the canvas or context.
        self.emergency = props.emergency;

        if self.emergency {
            self.start_service();
        } else {
            self.end_service();
        }
        true
    }
}

impl Renderable<MapViewComponent> for MapViewComponent {
    fn view(&self) -> Html<Self> {
        let mut render_distance_buttons = self.users.iter().map(|(user_mac, _user)| {
            let set_border = match &self.show_distance {
                Some(selected) => selected == user_mac,
                None => false,
            };
            html! {
                <ValueButton<String>
                    on_click=|value: String| Msg::ViewDistance(MacAddress::parse_str(&value).unwrap()),
                    border=set_border,
                    value={user_mac.to_hex_string()}
                />
            }
        });

        html! {
            <div>
                <div>
                    {
                        if self.users.len() > 0 {
                            "View Tag Distance Values: "
                        } else {
                            ""
                        }
                    }
                    { for render_distance_buttons }
                </div>
                { VNode::VRef(Node::from(self.map_canvas.to_owned()).to_owned()) }
            </div>
        }
    }
}
