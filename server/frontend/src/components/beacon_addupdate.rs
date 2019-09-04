use yew::services::fetch::{ FetchService, FetchTask, };
use yew::{ Component, ComponentLink, Html, Renderable, ShouldRender, html, };
use crate::util;
use yew::format::Json;
use common::*;

pub enum Msg {
    AddAnotherBeacon,
    InputName(String),
    InputMacAddress(String),
    InputFloorName(String),
    InputNote(String),

    RequestAddUpdateBeacon,

    ResponseUpdateBeacon(util::Response<common::Beacon>),
    ResponseAddBeacon(util::Response<common::Beacon>),
}

// keep all of the transient data together, since its not easy to create
// a "new" method for a component.
struct Data {
    pub beacon: common::Beacon,
    pub raw_mac: String,
    pub floor_names: Vec<String>,
    pub id: Option<i32>,
    pub error_messages: Vec<String>,
    pub success_message: Option<String>,
}

impl Data {
    fn new() -> Data {
        Data {
            beacon: Beacon::new(),
            raw_mac: MacAddress::nil().to_hex_string(),
            floor_names: Vec::new(),
            id: None,
            error_messages: Vec::new(),
            success_message: None,
        }
    }
}

pub struct BeaconAddUpdate {
    fetch_service: FetchService,
    fetch_task: Option<FetchTask>,
    self_link: ComponentLink<Self>,
    data: Data,
}

#[derive(Clone, Default, PartialEq)]
pub struct BeaconAddUpdateProps {
    pub id: Option<i32>,
}

impl Component for BeaconAddUpdate {
    type Message = Msg;
    type Properties = BeaconAddUpdateProps;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let mut result = BeaconAddUpdate {
            data: Data::new(),
            fetch_service: FetchService::new(),
            fetch_task: None,
            self_link: link,
        };
        result.data.id = props.id;
        result
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::AddAnotherBeacon => {
                self.data = Data::new();
            }
            Msg::InputName(name) => {
                self.data.beacon.name = name;
            },
            Msg::InputFloorName(name) => {
                self.data.beacon.map_id = Some(name);
            },
            Msg::InputNote(note) => {
                self.data.beacon.note = note;
            },
            Msg::InputMacAddress(mac) => {
                self.data.raw_mac = mac;
            },
            Msg::RequestAddUpdateBeacon => {
                self.data.error_messages = Vec::new();

                let success = match MacAddress::parse_str(&self.data.raw_mac) {
                    Ok(m) => {
                        self.data.beacon.mac_address = m;
                        true
                    },
                    Err(e) => {
                        self.data.error_messages.push(format!("failed to parse mac address: {}", e));
                        false
                    },
                };

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
            Msg::ResponseUpdateBeacon(response) => {
                let (meta, Json(body)) = response.into_parts();
                if meta.status.is_success() {
                    match body {
                        Ok(result) => {
                            Log!("returned beacon is {:?}", result);
                            self.data.success_message = Some("successfully updated beacon".to_string());
                            self.data.beacon = result;
                        }
                        _ => { }
                    }
                } else {
                    Log!("response - failed to update beacon");
                }
            },
            Msg::ResponseAddBeacon(response) => {
                let (meta, Json(body)) = response.into_parts();
                if meta.status.is_success() {
                    match body {
                        Ok(result) => {
                            Log!("returned beacon is {:?}", result);
                            self.data.success_message = Some("successfully added beacon".to_string());
                            self.data.beacon = result;
                            self.data.id = Some(self.data.beacon.id);
                            self.data.raw_mac = self.data.beacon.mac_address.to_hex_string();
                        }
                        _ => { }
                    }
                } else {
                    Log!("response - failed to update beacon");
                }
            },
        }
        true
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.data.id = props.id;
        true
    }
}

impl Renderable<BeaconAddUpdate> for BeaconAddUpdate {
    fn view(&self) -> Html<Self> {
        let submit_name = match self.data.id {
            Some(_id) => "Update Beacon",
            None => "Add Beacon",
        };
        let title_name = match self.data.id {
            Some(_id) => "Beacon Update",
            None => "Beacon Add",
        };
        let floor_name = match &self.data.beacon.map_id {
            Some(name) => name,
            None => "Unset",
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

        let mut floor_options = self.data.floor_names.iter().cloned().map(|floor| {
            let clone = floor.to_string();
            html! {
                <option
                    onclick=|_| Msg::InputFloorName(clone.clone()),
                    disabled={ floor == floor_name },
                >
                    { floor }
                </option>
            }
        });

        let mut errors = self.data.error_messages.iter().cloned().map(|msg| {
            html! {
                <p>{msg}</p>
            }
        });

        html! {
            <>
                <p>{ title_name }</p>
                {
                    match &self.data.success_message {
                        Some(msg) => { format!("Success: {}", msg) },
                        None => { "".to_string() },
                    }
                }
                { if self.data.error_messages.len() > 0 { "Failure: " } else { "" } }
                { for errors }
                <div/>
                { "Mac Address: " }
                <input
                    type="text",
                    value=&self.data.raw_mac,
                    oninput=|e| Msg::InputMacAddress(e.value),
                />
                <div/>
                { "Floor Name: " }
                <select>
                    { for floor_options }
                </select>
                <div/>
                { "Name: " }
                <input
                    type="text",
                    value=&self.data.beacon.name,
                    oninput=|e| Msg::InputName(e.value),
                />
                <div/>
                { "Note: " }
                <textarea
                    rows=5
                    value=&self.data.beacon.note,
                    oninput=|e| Msg::InputNote(e.value),
                />
                <div/>
                <button onclick=|_| Msg::RequestAddUpdateBeacon,>{ submit_name }</button>
                { add_another_button }
            </>
        }
    }
}
