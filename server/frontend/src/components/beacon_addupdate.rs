use common::*;
use crate::util::{ self, WebUserType, };
use super::root;
use yew::Callback;
use yew::format::Json;
use yew::prelude::*;
use yew::services::fetch::{ FetchService, FetchTask, };

pub enum Msg {
    AddAnotherBeacon,
    ChangeRootPage(root::Page),
    InputCoordinate(usize, String),
    InputFloorName(i32),
    InputMacAddress(String),
    InputName(String),
    InputNote(String),

    RequestAddUpdateBeacon,
    RequestGetAvailMaps,
    RequestGetBeacon(i32),

    ResponseAddBeacon(util::Response<Beacon>),
    ResponseGetAvailMaps(util::Response<Vec<Map>>),
    ResponseGetBeacon(util::Response<Option<(Beacon, Option<Map>)>>),
    ResponseUpdateBeacon(util::Response<Beacon>),
}

// keep all of the transient data together, since its not easy to create
// a "new" method for a component.
struct Data {
    pub beacon: Beacon,
    // the mac address needs to be parsed (and validated) as a mac address.
    // keep the raw string from the user in case the parsing fails.
    pub error_messages: Vec<String>,
    pub avail_floors: Vec<Map>,
    pub id: Option<i32>,
    pub raw_coord0: String,
    pub raw_coord1: String,
    pub raw_mac: String,
    pub success_message: Option<String>,
}

impl Data {
    fn new() -> Data {
        Data {
            beacon: Beacon::new(),
            error_messages: Vec::new(),
            avail_floors: Vec::new(),
            id: None,
            raw_coord0: "0".to_string(),
            raw_coord1: "0".to_string(),
            raw_mac: MacAddress8::nil().to_hex_string(),
            success_message: None,
        }
    }

    fn validate(&mut self) -> bool {
        let mut success = match MacAddress8::parse_str(&self.raw_mac) {
            Ok(m) => {
                self.beacon.mac_address = m;
                true
            },
            Err(e) => {
                self.error_messages.push(format!("failed to parse mac address: {}", e));
                false
            },
        };

        success = success && match self.raw_coord0.parse::<f64>() {
            Ok(coord) => {
                self.beacon.coordinates[0] = coord;
                true
            },
            Err(e) => {
                self.error_messages.push(format!("failed to parse x coordinate: {}", e));
                false
            },
        };

        success = success && match self.raw_coord1.parse::<f64>() {
            Ok(coord) => {
                self.beacon.coordinates[1] = coord;
                true
            },
            Err(e) => {
                self.error_messages.push(format!("failed to parse y coordinate: {}", e));
                false
            },
        };

        success
    }
}

pub struct BeaconAddUpdate {
    data: Data,
    fetch_service: FetchService,
    fetch_task: Option<FetchTask>,
    // TODO more robust way of handling concurrent requests
    get_fetch_task: Option<FetchTask>,
    self_link: ComponentLink<Self>,
    user_type: WebUserType,
    change_page: Callback<root::Page>,
}

#[derive(Properties)]
pub struct BeaconAddUpdateProps {
    #[props(required)]
    pub change_page: Callback<root::Page>,
    pub id: Option<i32>,
    #[props(required)]
    pub user_type: WebUserType,
}

impl Component for BeaconAddUpdate {
    type Message = Msg;
    type Properties = BeaconAddUpdateProps;

    fn create(props: Self::Properties, mut link: ComponentLink<Self>) -> Self {
        link.send_self(Msg::RequestGetAvailMaps);
        if let Some(id) = props.id {
            link.send_self(Msg::RequestGetBeacon(id));
        }
        let mut result = BeaconAddUpdate {
            change_page: props.change_page,
            data: Data::new(),
            fetch_service: FetchService::new(),
            fetch_task: None,
            get_fetch_task: None,
            self_link: link,
            user_type: props.user_type,
        };
        result.data.id = props.id;
        result
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::ChangeRootPage(page) => {
                self.change_page.emit(page);
            }
            Msg::AddAnotherBeacon => {
                self.data = Data::new();
                self.self_link.send_self(Msg::RequestGetAvailMaps);
            }
            Msg::InputName(name) => {
                self.data.beacon.name = name;
            },
            Msg::InputFloorName(map_id) => {
                self.data.beacon.map_id = Some(map_id);
            },
            Msg::InputNote(note) => {
                self.data.beacon.note = Some(note);
            },
            Msg::InputMacAddress(mac) => {
                self.data.raw_mac = mac;
            },
            Msg::InputCoordinate(index, value) => {
                match index {
                    0 => { self.data.raw_coord0 = value; },
                    1 => { self.data.raw_coord1 = value; },
                    _ => panic!("invalid coordinate index specified"),
                };
            },
            Msg::RequestGetAvailMaps => {
                self.fetch_task = get_request!(
                    self.fetch_service,
                    &maps_url(),
                    self.self_link,
                    Msg::ResponseGetAvailMaps
                );
            },
            Msg::RequestGetBeacon(id) => {
                self.get_fetch_task = get_request!(
                    self.fetch_service,
                    &beacon_url(&id.to_string()),
                    self.self_link,
                    Msg::ResponseGetBeacon
                );
            },
            Msg::RequestAddUpdateBeacon => {
                self.data.error_messages = Vec::new();
                self.data.success_message = None;

                let success = self.data.validate();

                match self.data.id {
                    Some(id) if success => {
                        //ensure the beacon id does not mismatch.
                        self.data.beacon.id = id;

                        self.fetch_task = put_request!(
                            self.fetch_service,
                            &beacon_url(&self.data.beacon.id.to_string()),
                            self.data.beacon,
                            self.self_link,
                            Msg::ResponseUpdateBeacon
                        );
                    },
                    None if success => {
                        self.fetch_task = post_request!(
                            self.fetch_service,
                            &beacon_url(""),
                            self.data.beacon,
                            self.self_link,
                            Msg::ResponseAddBeacon
                        );
                    }
                    _ => {},
                }
            },
            Msg::ResponseGetAvailMaps(response) => {
                let (meta, Json(body)) = response.into_parts();
                if meta.status.is_success() {
                    match body {
                        Ok(result) => {
                            // TODO add validation on the backend, and send decent messages to the
                            // frontend when the map_id is not set.
                            if self.data.beacon.map_id == None && result.len() > 0 {
                                self.data.beacon.map_id = Some(result[0].id);
                            }

                            self.data.avail_floors = result;
                        },
                        Err(e) => {
                            self.data.error_messages.push(format!("failed to obtain available floors list, reason: {}", e));
                        }
                    }
                } else {
                    self.data.error_messages.push("failed to obtain available floors list".to_string());
                }
            },
            Msg::ResponseUpdateBeacon(response) => {
                let (meta, Json(body)) = response.into_parts();
                if meta.status.is_success() {
                    match body {
                        Ok(result) => {
                            self.data.success_message = Some("successfully updated beacon".to_string());
                            self.data.beacon = result;
                        },
                        Err(e) => {
                            self.data.error_messages.push(format!("failed to update beacon, reason: {}", e));
                        }
                    }
                } else {
                    self.data.error_messages.push("failed to update beacon".to_string());
                }
            },
            Msg::ResponseGetBeacon(response) => {
                let (meta, Json(body)) = response.into_parts();
                if meta.status.is_success() {
                    match body {
                        Ok(result) => {
                            self.data.beacon = result.unwrap_or((Beacon::new(), None)).0;
                            self.data.raw_mac = self.data.beacon.mac_address.to_hex_string();
                            self.data.raw_coord0 = self.data.beacon.coordinates[0].to_string();
                            self.data.raw_coord1 = self.data.beacon.coordinates[1].to_string();
                        },
                        Err(e) => {
                            self.data.error_messages.push(format!("failed to find beacon, reason: {}", e));
                        }
                    }
                } else {
                    self.data.error_messages.push("failed to find beacon".to_string());
                }
            },
            Msg::ResponseAddBeacon(response) => {
                let (meta, Json(body)) = response.into_parts();
                if meta.status.is_success() {
                    match body {
                        Ok(result) => {
                            self.data.success_message = Some("successfully added beacon".to_string());
                            self.data.beacon = result;
                            self.data.id = Some(self.data.beacon.id);
                            self.data.raw_mac = self.data.beacon.mac_address.to_hex_string();
                        },
                        Err(e) => {
                            self.data.error_messages.push(format!("failed to add beacon, reason: {}", e));
                        }
                    }
                } else {
                    self.data.error_messages.push("failed to add beacon".to_string());
                }
            },
        }
        true
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.data.id = props.id;
        self.user_type = props.user_type;
        true
    }
}

impl Renderable<BeaconAddUpdate> for BeaconAddUpdate {
    fn view(&self) -> Html<Self> {
        let submit_name = match self.data.id {
            Some(_id) => "Update Beacon",
            None => "Add New Beacon",
        };
        let title_name = match self.data.id {
            Some(_id) => "Update Beacon",
            None => "Add New Beacon",
        };
        let chosen_floor_id = match self.data.beacon.map_id {
            Some(id) => id,
            None => -1,
        };
        let add_another_button = match &self.data.id {
            Some(_) => {
                html! {
                    <button onclick=|_| Msg::AddAnotherBeacon,>{ "Add Another" }</button>
                }
            },
            None => {
                html! { <></> }
            },
        };

        let mut floor_options = self.data.avail_floors.iter().cloned().map(|floor| {
            let floor_id = floor.id;
            html! {
                <option
                    onclick=|_| Msg::InputFloorName(floor_id),
                    disabled={ floor_id == chosen_floor_id },
                >
                    { &floor.name }
                </option>
            }
        });

        let mut errors = self.data.error_messages.iter().cloned().map(|msg| {
            html! {
                <div
                    class="alert alert-danger"
                    role="alert"
                >
                    {"ERROR: "}
                    {msg}
                </div>
            }
        });

        let display_errors = html! { for errors };

        let note = self.data.beacon.note.clone().unwrap_or(String::new());

        html! {
            <>
                <h2>{ title_name }</h2>
                {
                    match &self.data.success_message {
                        Some(msg) => { format!("Success: {}", msg) },
                        None => { "". to_string() },
                    }
                }
                { display_errors}
                <div/>
                <table>
                    <tr>
                        <td>{ "Mac Address: " }</td>
                        <td>
                            <input
                                type="text",
                                value=&self.data.raw_mac,
                                oninput=|e| Msg::InputMacAddress(e.value),
                            />
                        </td>
                    </tr>
                    <tr>
                        <td>{ "Floor Name: " }</td>
                        <td>
                            <select>
                                { for floor_options }
                            </select>
                        </td>
                    </tr>
                    <tr>
                        <td>{ "Name: " }</td>
                        <td>
                            <input
                                type="text",
                                value=&self.data.beacon.name,
                                oninput=|e| Msg::InputName(e.value),
                            />
                        </td>
                    </tr>
                    <tr>
                        <td>{ "Coordinates: " }</td>
                        <td>
                            <input
                                type="text",
                                value=&self.data.raw_coord0,
                                oninput=|e| Msg::InputCoordinate(0, e.value),
                            />
                        </td>
                        <td>
                            <input
                                type="text",
                                value=&self.data.raw_coord1,
                                oninput=|e| Msg::InputCoordinate(1, e.value),
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
                {
                    match self.user_type {
                        WebUserType::Admin => html! {
                            <>
                                <button onclick=|_| Msg::RequestAddUpdateBeacon,>{ submit_name }</button>
                                { add_another_button }
                            </>
                        },
                        WebUserType::Responder => html! { },
                    }
                }
                <button onclick=|_| Msg::ChangeRootPage(root::Page::BeaconList),>{ "Cancel" }</button>
            </>
        }
    }
}
