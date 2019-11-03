use common::*;
use crate::canvas::{ Canvas, screen_space };
use crate::util::{ self, WebUserType, };
use stdweb::web::event::{ ClickEvent, };
use stdweb::web::{ Node, };
use yew::format::Json;
use yew::services::fetch::{ FetchService, FetchTask, };
use yew::virtual_dom::vnode::VNode;
use yew::prelude::*;
use yew::IMouseEvent;
use stdweb::traits::*;
use yew::services::reader::{File, FileData, ReaderService, ReaderTask};

pub enum Msg {
    Ignore,
    AddAnotherMap,
    InputFile(File),
    FileLoaded(FileData),
    InputBound(usize, String),
    InputScale(String),
    InputName(String),
    InputNote(String),
    ToggleBeaconPlacement(i32),
    CanvasClick(ClickEvent),

    RequestAddUpdateMap,
    RequestGetMap(i32),
    RequestGetBeaconsForMap(i32),
    RequestPutBeacon(i32),

    ResponseAddMap(util::Response<Map>),
    ResponseGetBeaconsForMap(util::Response<Vec<Beacon>>),
    ResponseGetMap(util::Response<Option<Map>>),
    ResponseUpdateMap(util::Response<Map>),
    ResponsePutBeacon(util::Response<Option<Beacon>>),
}

// keep all of the transient data together, since its not easy to create
// a "new" method for a component.
struct Data {
    pub map: Map,
    pub error_messages: Vec<String>,
    pub attached_beacons: Vec<Beacon>,
    pub opt_id: Option<i32>,
    pub raw_bounds: [String; 2],
    pub raw_scale: String,
    pub success_message: Option<String>,
    pub current_beacon: Option<i32>,
}

impl Data {
    fn new() -> Data {
        Data {
            map: Map::new(),
            error_messages: Vec::new(),
            attached_beacons: Vec::new(),
            opt_id: None,
            raw_bounds: ["0".to_string(), "0".to_string()],
            raw_scale: "1".to_string(),
            success_message: None,
            current_beacon: None,
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

pub struct MapAddUpdate {
    file_reader: ReaderService,
    file_task: Option<ReaderTask>,
    canvas: Canvas,
    data: Data,
    fetch_service: FetchService,
    fetch_task: Option<FetchTask>,
    get_fetch_task: Option<FetchTask>,
    self_link: ComponentLink<Self>,
    user_type: WebUserType,
}

#[derive(Properties)]
pub struct MapAddUpdateProps {
    pub opt_id: Option<i32>,
    #[props(required)]
    pub user_type: WebUserType,
}

impl Component for MapAddUpdate {
    type Message = Msg;
    type Properties = MapAddUpdateProps;

    fn create(props: Self::Properties, mut link: ComponentLink<Self>) -> Self {
        if let Some(id) = props.opt_id {
            link.send_self(Msg::RequestGetMap(id));
            link.send_self(Msg::RequestGetBeaconsForMap(id));
        }
        let data = Data::new();

        let click_callback = link.send_back(|event| Msg::CanvasClick(event));
        let mut result = MapAddUpdate {
            canvas: Canvas::new("addupdate_canvas", click_callback),
            data,
            fetch_service: FetchService::new(),
            fetch_task: None,
            get_fetch_task: None,
            self_link: link,
            user_type: props.user_type,
            file_reader: ReaderService::new(),
            file_task: None,
        };

        result.canvas.reset(&result.data.map);
        result.canvas.draw_beacons(&result.data.map, &result.data.attached_beacons);
        result.data.opt_id = props.opt_id;
        result
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Ignore => {
            },
            Msg::InputFile(file) => {
                let callback = self.self_link.send_back(Msg::FileLoaded);
                let task = self.file_reader.read_file(file, callback);
                self.file_task = Some(task);
            },
            Msg::FileLoaded(data) => {
                Log!("data from file: {:?}", data);
            },
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
            Msg::ToggleBeaconPlacement(beacon_id) => {
                match self.data.current_beacon {
                    Some(id) if beacon_id == id => {
                        self.data.current_beacon = None;
                    }
                    _ => {
                        self.data.current_beacon = Some(beacon_id);
                    },
                }
            },
            Msg::CanvasClick(event) => {
                let canvas_bound = self.canvas.canvas.get_bounding_client_rect();
                match self.data.current_beacon {
                    Some(id) => {
                        match self.data.attached_beacons.iter().position(|beacon| beacon.id == id) {
                            Some(index) => {
                                let pix_coords = na::Vector2::new(event.client_x() - canvas_bound.get_left() as i32, event.client_y() - canvas_bound.get_top() as i32);
                                let world_coords = screen_space(&self.data.map, pix_coords.x as f64, pix_coords.y as f64);
                                let coords = na::Vector2::new(world_coords.x / self.data.map.scale as f64, world_coords.y / self.data.map.scale as f64);
                                self.data.attached_beacons[index].coordinates = coords;
                                self.canvas.reset(&self.data.map);
                                self.canvas.draw_beacons(&self.data.map, &self.data.attached_beacons);
                            },
                            _ => {
                                Log!("invalid current beacon");
                            },
                        }
                    },
                    _ => {
                        Log!("ignoring input location because a beacon has not been selected");
                    }
                }
            },
            Msg::RequestPutBeacon(id) => {
                match self.data.attached_beacons.iter().position(|beacon| beacon.id == id) {
                    Some(index) => {
                        self.fetch_task = put_request!(
                            self.fetch_service,
                            &beacon_url(&id.to_string()),
                            self.data.attached_beacons[index],
                            self.self_link,
                            Msg::ResponsePutBeacon
                        );
                    },
                    _ => {
                        Log!("could not save invalid beacon");
                    },
                }
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

                match self.data.opt_id {
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
            Msg::ResponsePutBeacon(response) => {
                let (meta, Json(body)) = response.into_parts();
                if meta.status.is_success() {
                    match body {
                        Ok(opt_beacon) => {
                            match opt_beacon {
                                Some(result) => {
                                    match self.data.attached_beacons.iter().position(|beacon| beacon.id == result.id) {
                                        Some(index) => {
                                            self.data.success_message = Some("successfully updated attached beacon".to_string());
                                            self.data.attached_beacons[index] = result;
                                        },
                                        _ => {
                                            Log!("updated beacon is no longer attached to this map");
                                        },
                                    }
                                },
                                None => {
                                    Log!("beacon does not exist");
                                }
                            }
                        },
                        Err(e) => {
                            self.data.error_messages.push(format!("failed to update attached beacon, reason: {}", e));
                        }
                    }
                } else {
                    self.data.error_messages.push("failed to updated attached beacon".to_string());
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
            },
            Msg::ResponseAddMap(response) => {
                let (meta, Json(body)) = response.into_parts();
                if meta.status.is_success() {
                    match body {
                        Ok(result) => {
                            self.data.success_message = Some("successfully added map".to_string());
                            self.data.map = result;
                            self.data.opt_id = Some(self.data.map.id);
                        },
                        Err(e) => {
                            self.data.error_messages.push(format!("failed to add map, reason: {}", e));
                        }
                    }
                } else {
                    self.data.error_messages.push("failed to add map".to_string());
                }
            },
        }

        self.canvas.reset(&self.data.map);
        self.canvas.draw_beacons(&self.data.map, &self.data.attached_beacons);
        true
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.data.opt_id = props.opt_id;
        self.user_type = props.user_type;
        if let Some(id) = props.opt_id {
            self.self_link.send_self(Msg::RequestGetMap(id));
            self.self_link.send_self(Msg::RequestGetBeaconsForMap(id));
        }
        true
    }
}

impl MapAddUpdate {
    fn render_beacon_placement(&self) -> Html<Self> {
        let mut beacon_placement_rows = self.data.attached_beacons.iter().cloned().map(|beacon| {
            let beacon_id = beacon.id;
            let this_beacon_selected = match self.data.current_beacon {
                Some(id) => id == beacon_id,
                _ => false,
            };

            html! {
                <tr>
                    <td>
                        { &beacon.name }
                    </td>
                    <td>
                        { &format!("{} {}", beacon.coordinates[0], beacon.coordinates[1]) }
                    </td>
                    <td>
                        <button
                            onclick=|_| Msg::RequestPutBeacon(beacon_id),
                        >
                            { "Save" }
                        </button>
                        <button
                            onclick=|_| Msg::ToggleBeaconPlacement(beacon_id),
                            class={ if this_beacon_selected { "bold_font" } else { "" } },
                        >
                            { "Toggle Placement" }
                        </button>
                    </td>
                </tr>
            }
        });

        match self.data.opt_id {
            Some(_) => {
                if self.data.attached_beacons.len() > 0 {
                    html! {
                        <>
                            <p>{ "Beacon Placement" }</p>
                            <table>
                                <tr>
                                    <td>
                                        { "Name" }
                                    </td>
                                    <td>
                                        { "Location" }
                                    </td>
                                    <td>
                                        { "Actions" }
                                    </td>
                                </tr>
                                { for beacon_placement_rows }
                            </table>
                            <div>
                                { VNode::VRef(Node::from(self.canvas.canvas.to_owned()).to_owned()) }
                            </div>
                        </>
                    }
                } else {
                    html! {
                        <p>{ "No Attached Beacons for this Map." }</p>
                    }
                }
            },
            None => {
                html! {
                    <></>
                }
            },
        }
    }
}

impl Renderable<MapAddUpdate> for MapAddUpdate {
    fn view(&self) -> Html<Self> {
        let submit_name = match self.data.opt_id {
            Some(_id) => "Update Map",
            None => "Add Map",
        };
        let title_name = match self.data.opt_id {
            Some(_id) => "Map Update",
            None => "Map Add",
        };

        let add_another_map = match &self.data.opt_id {
            Some(_) => {
                html! {
                    <button onclick=|_| Msg::AddAnotherMap,>{ "Add Another" }</button>
                }
            },
            None => {
                html! { <></> }
            },
        };

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
                    <tr>
                        <td>{ "Blueprint: " }</td>
                        <td>
                            <input
                                type="file",
                                onchange=|value| {
                                    if let ChangeData::Files(file_names) = value {
                                        match file_names.iter().next() {
                                            Some(file_name) => Msg::InputFile(file_name),
                                            None => Msg::Ignore,
                                        }
                                    } else {
                                        Msg::Ignore
                                    }
                                },
                            />
                        </td>
                    </tr>
                </table>
                {
                    match self.user_type {
                        WebUserType::Admin => html! {
                            <>
                                <button onclick=|_| Msg::RequestAddUpdateMap,>{ submit_name }</button>
                                { add_another_map }
                            </>
                        },
                        WebUserType::Responder => html! {
                            <></>
                        },
                    }
                }
                { self.render_beacon_placement() }
            </>
        }
    }
}
