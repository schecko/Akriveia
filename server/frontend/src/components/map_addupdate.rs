use common::*;
use crate::util;
use yew::format::Json;
use yew::services::fetch::{ FetchService, FetchTask, };
use yew::{ Component, ComponentLink, Html, Renderable, ShouldRender, html, };
use stdweb::web::{ CanvasRenderingContext2d, Node, };
use stdweb::web::html_element::CanvasElement;
use crate::canvas;
use yew::virtual_dom::vnode::VNode;

pub enum Msg {
    AddAnotherMap,
    InputBound(usize, String),
    InputScale(String),
    InputName(String),
    InputNote(String),

    StartBeaconPlacement(i32),
    EndBeaconPlacement,
    InputBeaconLocation(i32, na::Vector2<f64>),


    RequestAddUpdateMap,
    RequestGetMap(i32),
    RequestGetBeaconsForMap(i32),

    ResponseAddMap(util::Response<Map>),
    ResponseGetBeaconsForMap(util::Response<Vec<Beacon>>),
    ResponseGetMap(util::Response<Option<Map>>),
    ResponseUpdateMap(util::Response<Map>),
}

// keep all of the transient data together, since its not easy to create
// a "new" method for a component.
struct Data {
    pub map: Map,
    pub error_messages: Vec<String>,
    pub attached_beacons: Vec<Beacon>,
    pub id: Option<i32>,
    pub raw_bounds: [String; 2],
    pub raw_scale: String,
    pub success_message: Option<String>,
}

impl Data {
    fn new() -> Data {
        Data {
            map: Map::new(),
            error_messages: Vec::new(),
            attached_beacons: Vec::new(),
            id: None,
            raw_bounds: ["0".to_string(), "0".to_string()],
            raw_scale: "1".to_string(),
            success_message: None,
            current_beacon: Option<i32>,
        }
    }

    fn validate(&mut self) -> bool {
        let mut success = match self.raw_bounds[0].parse::<i32>() {
            Ok(coord) => {
                self.map.bounds[0] = coord;
                true
            },
            Err(e) => {
                self.error_messages.push(format!("failed to parse x coordinate: {}", e));
                false
            },
        };

        success = success && match self.raw_bounds[1].parse::<i32>() {
            Ok(coord) => {
                self.map.bounds[1] = coord;
                true
            },
            Err(e) => {
                self.error_messages.push(format!("failed to parse y coordinate: {}", e));
                false
            },
        };

        success = success && match self.raw_scale.parse::<f64>() {
            Ok(scale) => {
                self.map.scale = scale;
                true
            },
            Err(e) => {
                self.error_messages.push(format!("failed to parse scale: {}", e));
                false
            },
        };

        success
    }
}

enum PageMode {
    Add,
    Update,
}

pub struct MapAddUpdate {
    canvas: CanvasElement,
    context: CanvasRenderingContext2d,
    data: Data,
    fetch_service: FetchService,
    fetch_task: Option<FetchTask>,
    get_fetch_task: Option<FetchTask>,
    mode: PageMode,
    self_link: ComponentLink<Self>,
}

#[derive(Clone, Default, PartialEq)]
pub struct MapAddUpdateProps {
    pub opt_id: Option<i32>,
}

impl Component for MapAddUpdate {
    type Message = Msg;
    type Properties = MapAddUpdateProps;

    fn create(props: Self::Properties, mut link: ComponentLink<Self>) -> Self {
        let mut mode = PageMode::Add;
        if let Some(id) = props.opt_id {
            link.send_self(Msg::RequestGetMap(id));
            link.send_self(Msg::RequestGetBeaconsForMap(id));
            mode = PageMode::Update;
        }
        let canvas = canvas::make_canvas("addupdate_canvas");
        let context = canvas::get_context(&canvas);

        let mut result = MapAddUpdate {
            canvas,
            context,
            data: Data::new(),
            fetch_service: FetchService::new(),
            fetch_task: None,
            get_fetch_task: None,
            mode,
            self_link: link,
        };

        canvas::reset_canvas(&result.canvas, &result.context, &result.data.map);
        result.data.id = props.opt_id;
        result
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::AddAnotherMap => {
                self.data = Data::new();
            }
            Msg::InputName(name) => {
                self.data.map.name = name;
            },
            Msg::InputNote(note) => {
                self.data.map.note = Some(note);
            },
            Msg::InputBound(index, value) => {
                self.data.raw_bounds[index] = value;
            },
            Msg::InputScale(value) => {
                self.data.raw_scale = value;
            },
            Msg::StartBeaconPlacement(beacon_id) => {
                self.data.current_beacon = Some(beacon_id);
            },
            Msg::InputBeaconLocation(beacon_id, location) => {
                self.data.current_beacon = Some(beacon_id);
            },
            Msg::EndBeaconPlacement(value) => {
                self.data.current_beacon = None;
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
            Msg::RequestAddUpdateMap => {
                self.data.error_messages = Vec::new();
                self.data.success_message = None;

                let success = self.data.validate();

                match self.data.id {
                    Some(id) if success => {
                        //ensure the id does not mismatch.
                        self.data.map.id = id;

                        self.fetch_task = put_request!(
                            self.fetch_service,
                            &map_url(&self.data.map.id.to_string()),
                            self.data.map,
                            self.self_link,
                            Msg::ResponseUpdateMap
                        );
                    },
                    None if success => {
                        self.fetch_task = post_request!(
                            self.fetch_service,
                            &map_url(""),
                            self.data.map,
                            self.self_link,
                            Msg::ResponseAddMap
                        );
                    },
                    _ => { }
                }
            },
            Msg::ResponseGetBeaconsForMap(response) => {
                let (meta, Json(body)) = response.into_parts();
                if meta.status.is_success() {
                    match body {
                        Ok(result) => {
                            self.data.attached_beacons = result;
                        },
                        Err(e) => {
                            self.data.error_messages.push(format!("failed to obtain available floors list, reason: {}", e));
                        }
                    }
                } else {
                    self.data.error_messages.push("failed to obtain available floors list".to_string());
                }
            },
            Msg::ResponseUpdateMap(response) => {
                let (meta, Json(body)) = response.into_parts();
                if meta.status.is_success() {
                    match body {
                        Ok(result) => {
                            self.data.success_message = Some("successfully updated map".to_string());
                            self.data.map = result;
                        },
                        Err(e) => {
                            self.data.error_messages.push(format!("failed to update map, reason: {}", e));
                        }
                    }
                } else {
                    self.data.error_messages.push("failed to update map".to_string());
                }
                canvas::reset_canvas(&self.canvas, &self.context, &self.data.map);
            },
            Msg::ResponseGetMap(response) => {
                let (meta, Json(body)) = response.into_parts();
                if meta.status.is_success() {
                    match body {
                        Ok(result) => {
                            self.data.map = result.unwrap_or(Map::new());
                            self.data.raw_bounds[0] = self.data.map.bounds[0].to_string();
                            self.data.raw_bounds[1] = self.data.map.bounds[1].to_string();
                            self.data.raw_scale = self.data.map.scale.to_string();
                        },
                        Err(e) => {
                            self.data.error_messages.push(format!("failed to find map, reason: {}", e));
                        }
                    }
                } else {
                    self.data.error_messages.push("failed to find map".to_string());
                }
                canvas::reset_canvas(&self.canvas, &self.context, &self.data.map);
            },
            Msg::ResponseAddMap(response) => {
                let (meta, Json(body)) = response.into_parts();
                if meta.status.is_success() {
                    match body {
                        Ok(result) => {
                            self.data.success_message = Some("successfully added map".to_string());
                            self.data.map = result;
                            self.data.id = Some(self.data.map.id);
                        },
                        Err(e) => {
                            self.data.error_messages.push(format!("failed to add map, reason: {}", e));
                        }
                    }
                } else {
                    self.data.error_messages.push("failed to add map".to_string());
                }
                canvas::reset_canvas(&self.canvas, &self.context, &self.data.map);
            },
        }
        true
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.data.id = props.opt_id;
        if let Some(id) = props.opt_id {
            self.self_link.send_self(Msg::RequestGetMap(id));
            self.self_link.send_self(Msg::RequestGetBeaconsForMap(id));
            self.mode = PageMode::Update;
        } else {
            self.mode = PageMode::Add;
        }
        true
    }
}

impl Renderable<MapAddUpdate> for MapAddUpdate {
    fn view(&self) -> Html<Self> {
        let submit_name = match self.data.id {
            Some(_id) => "Update Map",
            None => "Add Map",
        };
        let title_name = match self.data.id {
            Some(_id) => "Map Update",
            None => "Map Add",
        };

        let add_another_map = match &self.data.id {
            Some(_) => {
                html! {
                    <button onclick=|_| Msg::AddAnotherMap,>{ "Add Another" }</button>
                }
            },
            None => {
                html! { <></> }
            },
        };

        let mut attached_beacons = self.data.attached_beacons.iter().cloned().map(|beacon| {
            html! {
                <option
                    onclick=|_| Msg::StartPlacement(beacon.id),
                    disabled={ floor_id == chosen_floor_id },
                >
                    { &beacon.name }
                </option>
            }
        });

        let mut errors = self.data.error_messages.iter().cloned().map(|msg| {
            html! {
                <p>{msg}</p>
            }
        });

        let note = self.data.map.note.clone().unwrap_or(String::new());

        html! {
            <>
                <p>{ title_name }</p>
                {
                    match &self.data.success_message {
                        Some(msg) => { format!("Success: {}", msg) },
                        None => { String::new() },
                    }
                }
                { if self.data.error_messages.len() > 0 { "Failure: " } else { "" } }
                { for errors }
                <div/>
                <table>
                    <tr>
                        <td>{ "Name: " }</td>
                        <td>
                            <input
                                type="text",
                                value=&self.data.map.name,
                                oninput=|e| Msg::InputName(e.value),
                            />
                        </td>
                    </tr>
                    <tr>
                        <td>{ "Bounds: " }</td>
                        <td>
                            <input
                                type="text",
                                value=&self.data.raw_bounds[0],
                                oninput=|e| Msg::InputBound(0, e.value),
                            />
                        </td>
                        <td>
                            <input
                                type="text",
                                value=&self.data.raw_bounds[1],
                                oninput=|e| Msg::InputBound(1, e.value),
                            />
                        </td>
                    </tr>
                    <tr>
                        <td>{ "Scale: " }</td>
                        <td>
                            <input
                                type="text",
                                value=&self.data.raw_scale,
                                oninput=|e| Msg::InputScale(e.value),
                            />
                        </td>
                    </tr>
                    <tr>
                        <td>{ "Note: " }</td>
                        <td>
                            <textarea
                                rows=5,
                                value=note,
                                oninput=|e| Msg::InputNote(e.value),
                            />
                        </td>
                    </tr>
                </table>
                <p>{ "Beacon Placement" }</p>
                <div>
                    { VNode::VRef(Node::from(self.canvas.to_owned()).to_owned()) }
                </div>
                <button onclick=|_| Msg::RequestAddUpdateMap,>{ submit_name }</button>
                { add_another_map }
            </>
        }
    }
}
