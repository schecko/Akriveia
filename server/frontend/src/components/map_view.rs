
use stdweb::web::html_element::CanvasElement;
use stdweb::web::{ CanvasRenderingContext2d, Element, Node, FillRule };
use yew::services::fetch::{ FetchService, FetchTask, Request, Response, };
use yew::services::interval::{ IntervalTask, IntervalService, };
use yew::virtual_dom::vnode::VNode;
use yew::{ Component, ComponentLink, Html, Renderable, ShouldRender, html, };
use yew::services::console::ConsoleService;
use crate::util;
use std::time::Duration;
use yew::format::{ Nothing, Json };
use std::collections::BTreeMap;
use super::value_button::ValueButton;
use na;

const REALTIME_USER_POLL_RATE: Duration = Duration::from_millis(1000);

const MAP_WIDTH: u32 = 800;
const MAP_HEIGHT: u32 = 800;
const MAP_SCALE: f64 = MAP_WIDTH as f64 / 4.0;

pub enum Msg {
    RenderMap,
    ViewDistance(String),

    RequestRealtimeUser,

    ResponseRealtimeUser(util::Response<Vec<common::User>>),
}

pub struct MapViewComponent {
    context: CanvasRenderingContext2d,
    fetch_service: FetchService,
    fetch_task: Option<FetchTask>,
    interval_service: IntervalService,
    interval_service_task: IntervalTask,
    map_canvas: CanvasElement,
    self_link: ComponentLink<MapViewComponent>,
    users: BTreeMap<String, Box<common::User>>,
    show_distance: Option<String>,
}

fn screen_space(x: f64, y: f64) -> na::Vector2<f64> {
    na::Vector2::new(x, MAP_HEIGHT as f64 - y)
}

fn screen_space_vector(coords: na::Vector2<f64>) -> na::Vector2<f64> {
    na::Vector2::new(coords.x, MAP_HEIGHT as f64 - coords.y)
}

impl MapViewComponent {
    fn clear_map(&self) {
        // clear the canvas and draw a border

        self.context.set_line_dash(vec![]);
        self.context.clear_rect(0.0, 0.0, self.map_canvas.width().into(), self.map_canvas.height().into());
        self.context.stroke_rect(0.0, 0.0, self.map_canvas.width().into(), self.map_canvas.height().into());

        self.context.save();
        self.context.set_line_dash(vec![5.0, 15.0]);
        // vertical gridlines
        for i in (MAP_SCALE as u32..MAP_WIDTH as u32).step_by(MAP_SCALE as usize) {
            let pos0 = screen_space(i as f64, MAP_HEIGHT as f64);
            let pos1 = screen_space(i as f64, 0.0);
            self.context.begin_path();
            self.context.move_to(pos0.x, pos0.y);
            self.context.line_to(pos1.x, pos1.y);
            self.context.stroke();
        }
        // horizontal gridlines
        for i in (MAP_SCALE as u32..MAP_HEIGHT as u32).step_by(MAP_SCALE as usize) {
            let pos0 = screen_space(MAP_WIDTH as f64, i as f64);
            let pos1 = screen_space(0.0, i as f64);
            self.context.begin_path();
            self.context.move_to(pos0.x, pos0.y);
            self.context.line_to(pos1.x, pos1.y);
            self.context.stroke();
        }
        self.context.restore();

        let text_adjustment = 10.0;
        // x axis
        for i in 0..(MAP_WIDTH / MAP_SCALE as u32) {
            let pos = screen_space(i as f64 * MAP_SCALE + text_adjustment, text_adjustment);
            self.context.fill_text(&format!("{}m", i), pos.x, pos.y, None);
        }
        // y axis
        // skip 0 because it was rendered by the y axis.
        for i in 1..(MAP_HEIGHT / MAP_SCALE as u32) {
            let pos = screen_space(text_adjustment, i as f64 * MAP_SCALE + text_adjustment);
            self.context.fill_text(&format!("{}m", i), pos.x, pos.y, None);
        }

    }
}

fn get_context(canvas: &CanvasElement) -> CanvasRenderingContext2d {
    unsafe {
        js! (
            return @{canvas}.getContext("2d");
        ).into_reference_unchecked().unwrap()
    }
}

impl Component for MapViewComponent {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, mut link: ComponentLink<Self>) -> Self {
        let canvas: CanvasElement = unsafe {
            js! (
                let c = document.createElement("canvas");
                c.setAttribute("id", "map_canvas");
                return c;
            ).into_reference_unchecked().unwrap()
        };
        canvas.set_width(MAP_WIDTH);
        canvas.set_height(MAP_HEIGHT);
        let context = get_context(&canvas);

        let mut interval_service = IntervalService::new();
        let task = interval_service.spawn(REALTIME_USER_POLL_RATE, link.send_back(|_| Msg::RequestRealtimeUser));

        let mut result = MapViewComponent {
            context: context,
            fetch_service: FetchService::new(),
            fetch_task: None,
            interval_service: interval_service,
            interval_service_task: task,
            map_canvas: canvas,
            users: BTreeMap::new(),
            self_link: link,
            show_distance: None,
        };

        result.clear_map();
        result
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::RenderMap => {
                self.context.save();
                for (tag_mac, user) in self.users.iter() {
                    let user_pos = screen_space(
                        user.location.x as f64 * MAP_SCALE,
                        user.location.y as f64 * MAP_SCALE,
                    );

                    for beacon_source in &user.beacon_sources {
                        let beacon_loc = screen_space(
                            beacon_source.location.x * MAP_SCALE,
                            beacon_source.location.y * MAP_SCALE,
                        );
                        self.context.set_fill_style_color("#0000FFFF");
                        self.context.fill_rect(beacon_loc.x, beacon_loc.y - 30.0, 30.0, 30.0);
                        self.context.set_fill_style_color("#000000FF");
                        self.context.fill_rect(user_pos.x, user_pos.y, 20.0, 20.0);
                        match &self.show_distance {
                            Some(tag_mac) if tag_mac == &user.tag_mac => {
                                self.context.set_fill_style_color("#00000034");
                                self.context.begin_path();
                                self.context.arc(beacon_loc.x, beacon_loc.y, beacon_source.distance_to_tag * MAP_SCALE, 0.0, std::f64::consts::PI * 2.0, true);
                                self.context.fill(FillRule::NonZero);
                            },
                            _ => { }
                        }
                    }
                }
                self.context.restore();

                return false;
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
                return true;
            },
            Msg::RequestRealtimeUser => {
                self.fetch_task = get_request!(
                    self.fetch_service,
                    common::REALTIME_USERS,
                    self.self_link,
                    Msg::ResponseRealtimeUser
                );

                return false;
            },
            Msg::ResponseRealtimeUser(response) => {
                self.clear_map();
                let (meta, Json(body)) = response.into_parts();
                if meta.status.is_success() {
                    if let Ok(data) = body {
                        for user in data.iter() {
                            match self.users.get_mut(&user.tag_mac) {
                                Some(mut local_user_data) => {
                                    **local_user_data = user.clone();
                                },
                                None => {
                                    self.users.insert(user.tag_mac.clone(), Box::new(user.clone()));
                                }

                            }
                        }
                    }
                } else {
                    Log!("response - failed to get realtime user data");
                }

                self.self_link.send_self(Msg::RenderMap);
                return true;
            }
        }
    }

    fn change(&mut self, _: Self::Properties) -> ShouldRender {
        // do not overwrite the canvas or context.
        true
    }
}

impl Renderable<MapViewComponent> for MapViewComponent {
    fn view(&self) -> Html<Self> {
        let mut render_distance_buttons = self.users.iter().map(|(user_mac, user)| {
            let set_border = match &self.show_distance {
                Some(selected) => selected == user_mac,
                None => false,
            };
            html! {
                <ValueButton
                    on_click=|value| Msg::ViewDistance(value),
                    border=set_border,
                    value={user_mac.clone()}
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
