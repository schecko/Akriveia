use common::*;
use crate::util;
use yew::format::Json;
use yew::services::fetch::{ FetchService, FetchTask, };
use yew::{ Component, ComponentLink, Html, Renderable, ShouldRender, html, };

pub enum Msg {
    AddAnotherUser,
    // Refers to ID tag MAC address
    InputMacAddress(String),
    // Does there need to be coordinates as well? (Yes)
    InputCoordinate(usize, usize),
    InputName(String),
    InputAddress(String),
    // Do we include employee ID #?
    InputFloorName(i32),
    InputPhone(String),
    InputMobilePhone(String),
    InputEmergencyContact(String),
    InputNote(String),

    RequestAddUpdateUser,
    RequestGetAvailMaps,

    // Response holds HTTP response from HTTP request
    ResponseUpdateUser(util::Response<User>),
    ResponseGetAvailMaps(util::Response<Vec<Map>>),
    ResponseAddUser(util::Response<User>),
}

// keep all of the transient data together, since its not easy to create
// a "new" method for a component.
struct Data {
    // Where do you get the type User?
    pub user: TrackedUser,
    pub error_messages: Vec<String>,
    pub avail_floors: Vec<Map>,
    // What's the difference b/w str (string slice) and String?

    pub name: String,
    pub employee_id: Option<String>,
    pub coordinates: Vec<usize, usize>,
    pub work_phone: Option<String>,
    pub emergency_contact: Option<i32>, // Would that reference to the User ID?
    pub mobile_phone: Option<String>,
    pub note: String, // Is this note needed?
    // --- Employee Info ----//

    // What ID is this? Why is it an option? 
    pub id: Option<i32>,
    // This should be the ID tag Mac address
    pub raw_mac: String,
    pub success_message: Option<String>,
}

impl Data {
    fn new() -> Data {
        Data {
            user: TrackedUser::new(),
            error_messages: Vec::new(),
            avail_floors: Vec::new(),
            // Do you input a blank string
            name: String::new();
            work_phone: String::new();
            mobile_phone: String::new();
            emergency_name: String::new();
            emergency_phone: String::new();
            emergency_relations: String::new();
            note: String::new();

            id: None,
            raw_mac: MacAddress::nil().to_hex_string(),
            success_message: None,
        }
    }

    // How do we validate that the user data is valid?
    // Check raw Mac address of the ID tag
    fn validate(&mut self) -> bool {
        let mut success = match MacAddress::parse_str(&self.raw_mac) {
            Ok(m) => {
                self.user.mac_address = m;
                true
            },
            Err(e) => {
                self.error_messages.push(format!("failed to parse mac address: {}", e));
                false
            },
        };
        
        // What else do we need to validate?

        success
    }
}

// What does fetch_service/task and self_link do?
pub struct UserAddUpdate {
    data: Data,
    fetch_service: FetchService,
    fetch_task: Option<FetchTask>,
    self_link: ComponentLink<Self>,
}

#[derive(Clone, Default, PartialEq)]
pub struct UserAddUpdateProps {
    pub id: Option<i32>,
}

impl Component for UserAddUpdate {
    type Message = Msg;
    type Properties = UserAddUpdateProps;

    fn create(props: Self::Properties, mut link: ComponentLink<Self>) -> Self {
        link.send_self(Msg::RequestGetAvailMaps);
        let mut result = UserAddUpdate {
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
            Msg::AddAnotherUser => {
                self.data = Data::new();
            }
            Msg::InputMacAddress(mac) => {
                self.data.raw_mac = mac;
            },
            Msg::InputName(name) => {
                self.data.user.name = name;
            },
            Msg::InputAddress(name) => {
                self.data.user.address;
            },
            Msg::InputJobTitle(job_title) => {
                self.data.user.job_title;
            }
            Msg::InputEmployeeID(employee_id) => {
                self.data.user.job_title;
            }
            Msg::InputFloorName(map_id) => {
                self.data.user.map_id = Some(map_id);
            },
            Msg::InputDepartment(department) => {
                self.data.user.department;
            }
            Msg::InputPhone(phone) => {
                self.data.user.phone;
            }
            Msg::InputMobilePhone(mobile_phone) => {
                self.data.user.mobile_phone;
            }
            Msg::InputEmergencyName(emergency_name) => {
                self.data.user.emergency_name;
            }
            Msg::InputEmergencyPhone(emergency_phone) => {
                self.data.user.emergency_phone;
            }
            Msg::InputRelations(emergency_relations) => {
                self.data.user.emergency_relations;
            }
            Msg::InputNote(note) => {
                self.data.user.note = Some(note);
            },
            Msg::RequestGetAvailMaps => {
                self.fetch_task = get_request!(
                    self.fetch_service,
                    &maps_url(),
                    self.self_link,
                    Msg::ResponseGetAvailMaps
                );
            },
            Msg::RequestAddUpdateUser => {
                self.data.error_messages = Vec::new();
                self.data.success_message = None;

                // Do we need to validate the data?
                let success = self.data.validate();

                match self.data.id {
                    Some(id) if success => {
                        //Ensure the beacon id does not mismatch.
                        // Where do we compare the list of user ID tags?
                        self.data.user.id = id;

                        self.fetch_task = put_request!(
                            self.fetch_service,
                            &user_url(&self.data.user.id.to_string()),
                            //&beacon_url(&self.data.beacon.id.to_string()),
                            self.data.user,
                            self.self_link,
                            Msg::ResponseUpdateBeacon
                        );
                    },
                    None if success => {
                        self.fetch_task = post_request!(
                            self.fetch_service,
                            &user_url(""),
                            //&beacon_url(""),
                            self.data.beacon,
                            self.self_link,
                            Msg::ResponseAddUser
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
                            Log!("returned avail maps is {:?}", result);
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
            Msg::ResponseUpdateUser(response) => {
                let (meta, Json(body)) = response.into_parts();
                if meta.status.is_success() {
                    match body {
                        Ok(result) => {
                            Log!("returned user is {:?}", result);
                            self.data.success_message = Some("successfully updated user".to_string());
                            self.data.user = result;
                        },
                        Err(e) => {
                            self.data.error_messages.push(format!("failed to update user, reason: {}", e));
                        }
                    }
                } else {
                    self.data.error_messages.push("failed to update user".to_string());
                }
            },
            Msg::ResponseAddUser(response) => {
                let (meta, Json(body)) = response.into_parts();
                if meta.status.is_success() {
                    match body {
                        Ok(result) => {
                            Log!("returned beacon is {:?}", result);
                            self.data.success_message = Some("successfully added user".to_string());
                            self.data.user = result;
                            self.data.id = Some(self.data.user.id);
                            self.data.raw_mac = self.data.user.mac_address.to_hex_string();
                        },
                        Err(e) => {
                            self.data.error_messages.push(format!("failed to add user, reason: {}", e));
                        }
                    }
                } else {
                    self.data.error_messages.push("failed to add user".to_string());
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

// The front-end layout in HTML
impl Renderable<UserAddUpdate> for UserAddUpdate {
    fn view(&self) -> Html<Self> {
        let submit_name = match self.data.id {
            Some(_id) => "Update User",
            None => "Add User",
        };
        let title_name = match self.data.id {
            Some(_id) => "User Update",
            None => "User Add",
        };
        // Users ID tag doesn't have a fixed floor
        let chosen_floor_id = match self.data.user.map_id {
            Some(id) => id,
            None => -1,
        };
        let add_another_button = match &self.data.id {
            Some(_) => {
                html! {
                    <button onclick=|_| Msg::AddAnotherUser,>{ "Add Another" }</button>
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
                <p>{msg}</p>
            }
        });

        let note = self.data.user.note.clone().unwrap_or(String::new());

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
                                value=&self.data.user.name,
                                oninput=|e| Msg::InputName(e.value),
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
                <button onclick=|_| Msg::RequestAddUpdateBeacon,>{ submit_name }</button>
                { add_another_button }
            </>
        }
    }
}
