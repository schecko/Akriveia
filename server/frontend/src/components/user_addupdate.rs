use common::*;
use crate::util;
use yew::format::Json;
use yew::services::fetch::{ FetchService, FetchTask, };
use yew::{ Component, ComponentLink, Html, Renderable, ShouldRender, html, };
// Remove after AddUser works

pub enum Msg {
    AddAnotherUser,
    // Refers to ID tag MAC address
    InputMacAddress(String),
    InputName(String),
    // Do we include employee ID #?
    InputEmployeeID(String),
    InputPhone(String),
    InputMobilePhone(String),
    InputNote(String),
    // Do you use a Option or not?
    // Can you have an option as a message parameter?
    InputEmergencyName(String),
    InputEmergencyEmployeeID(String),
    InputEmergencyNote(String),
    InputEmergencyPhone(String),
    InputEmergencyMobilePhone(String),

    RequestAddUpdateUser,
    RequestGetUser(i32),
    // Do we need the map for it as well? Probably not
    // Response holds HTTP response from request
    ResponseAddUser(util::Response<TrackedUser>),
    ResponseGetUser(util::Response<Option<TrackedUser>>),
    ResponseUpdateUser(util::Response<TrackedUser>),
}

// keep all of the transient data together, since its not easy to create
// a "new" method for a component.
struct Data {
    // Where do you get the type User?
    pub user: TrackedUser,
    pub emergency_user: Option<TrackedUser>,
    pub error_messages: Vec<String>,
    // What's the difference b/w str (string slice) and String?

    pub id: Option<i32>,
    // This should be the ID tag Mac address
    pub raw_mac: String,
    pub success_message: Option<String>,
}

impl Data {
    fn new() -> Data {
        Data {
            user: TrackedUser::new(),
            emergency_user: None,
            error_messages: Vec::new(),
            id: None,
            raw_mac: MacAddress::nil().to_hex_string(),
            success_message: None,
        }
    }

    // How do we validate that the user data is valid?
    // Check raw Mac address of the ID tag
    fn validate(&mut self) -> bool {
        let success = match MacAddress::parse_str(&self.raw_mac) {
            Ok(m) => {
                self.user.mac_address = m;
                true
            },
            Err(e) => {
                self.error_messages.push(format!("failed to parse mac address: {}", e));
                false
            }
        };
        // Don't need to validate the emergency contact since it can be NOne  
        success
    }
}

// What does fetch_service/task and self_link do?
pub struct UserAddUpdate {
    data: Data,
    fetch_service: FetchService,
    fetch_task: Option<FetchTask>,
    get_fetch_task: Option<FetchTask>,
    self_link: ComponentLink<Self>,
}

#[derive(Clone, Default, PartialEq)]
pub struct UserAddUpdateProps {
    pub id: Option<i32>,
}

impl Component for UserAddUpdate {
    type Message = Msg;
    type Properties = UserAddUpdateProps;

    // mut link or
    fn create(props: Self::Properties, mut link: ComponentLink<Self>) -> Self {
        // Perhaps I'm missing the command to fetch the user IDs
        if let Some(id) = props.id {
            link.send_self(Msg::RequestGetUser(id));
        }

        let mut result = UserAddUpdate {
            data: Data::new(),
            // what does these fetch services do?
            // Assume: Call for requests to the web services endpoints
            fetch_service: FetchService::new(),
            fetch_task: None,
            // What does get_fetch_task do?
            // Is it to get the results of the fetch task?
            get_fetch_task: None,
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
            Msg::InputEmployeeID(employee_id) => {
                self.data.user.employee_id = Some(employee_id);
            },
            Msg::InputPhone(phone) => {
                self.data.user.phone = Some(phone);
            },
            Msg::InputMobilePhone(mobile_phone) => {
                self.data.user.mobile_phone = Some(mobile_phone);
            },
            Msg::InputNote(note) => {
                self.data.user.note = Some(note);
            },
            // Set emergency_user = TrackedUser
            Msg::InputEmergencyName(emergency_name) => {
                let emergency_user = &mut self.data.emergency_user.take();
                match emergency_user {
                    Some(emergency_user) => {
                        emergency_user.name = emergency_name;
                    }
                    None => {
                        let mut user = TrackedUser::new();
                        user.name = emergency_name;
                        *emergency_user = Some(user);
                    }    
                }
            },
            Msg::InputEmergencyEmployeeID(emergency_id) => {
                let emergency_user = &mut self.data.emergency_user.take();
                match emergency_user {
                    Some(emergency_user) => {
                        emergency_user.employee_id = Some(emergency_id);
                    }
                    None => {
                        let mut user = TrackedUser::new();
                        user.employee_id = Some(emergency_id);
                        *emergency_user = Some(user);
                    }
                }
            },
            Msg::InputEmergencyNote(emergency_note) => {
                let emergency_user = &mut self.data.emergency_user.take();
                match emergency_user {
                    Some(emergency_user) => {
                        emergency_user.note = Some(emergency_note);
                    }
                    None => {
                        let mut user = TrackedUser::new();
                        user.note = Some(emergency_note);
                        *emergency_user = Some(user);
                    }
                }
            },
            Msg::InputEmergencyPhone(emergency_phone) => {
                let emergency_user = &mut self.data.emergency_user.take();
                match emergency_user {
                    Some(emergency_user) => {
                        emergency_user.phone = Some(emergency_phone);
                    }
                    None => {
                        let mut user = TrackedUser::new();
                        user.phone = Some(emergency_phone);
                        *emergency_user = Some(user);
                    }
                }
            },
            Msg::InputEmergencyMobilePhone(emergency_mobile) => {
                let emergency_user = &mut self.data.emergency_user.take();
                match emergency_user {
                    Some(emergency_user) => {
                        emergency_user.mobile_phone = Some(emergency_mobile);
                    }
                    None => {
                        let mut user = TrackedUser::new();
                        user.mobile_phone = Some(emergency_mobile);
                        *emergency_user = Some(user);
                    }
                }
            },
            Msg::RequestAddUpdateUser => {
                self.data.error_messages = Vec::new();
                self.data.success_message = None;

                // Do we need to validate the data?
                let success = self.data.validate();

                match self.data.id {
                    Some(id) if success => {
                        // Ensure the beacon id does not mismatch.
                        // Where do we compare the list of user ID tags?
                        self.data.user.id = id;

                        self.data.error_messages.push("Found the id. Arrived before push request".to_string());
                        self.fetch_task = put_request!(
                            self.fetch_service,
                            &user_url(&self.data.user.id.to_string()),
                            self.data.user,
                            self.self_link,
                            Msg::ResponseUpdateUser
                        );
                        self.data.error_messages.push("Should be a success".to_string());
                    },
                    None if success => {
                        self.data.error_messages.push("New user. Sending the new post request".to_string());
                        self.fetch_task = post_request!(
                            self.fetch_service,
                            &user_url(""),
                            //&beacon_url(""),
                            self.data.user,
                            self.self_link,
                            Msg::ResponseAddUser
                        );
                        // Need to add emergency user too
                        self.fetch_task = post_request!(
                            self.fetch_service,
                            &user_url(""),
                            self.data.emergency_user,
                            self.self_link,
                            Msg::ResponseAddUser
                        );
                        self.data.error_messages.push("After the post_request. Should be finished".to_string());
                    }
                    _ => {
                        self.data.error_messages.push("Other cases error in add/updating".to_string());
                    },
                }
            },
            Msg::RequestGetUser(id) => {
                self.get_fetch_task = get_request! (
                    self.fetch_service,
                    &user_url(&id.to_string()),
                    self.self_link,
                    Msg::ResponseGetUser
                    );
            },
            Msg::ResponseGetUser(response) => {
                let (meta, Json(body)) = response.into_parts();
                if meta.status.is_success() { 
                    match body {
                        Ok(result) => {
                            // What reponse is needed as to get user
                            self.data.user = result.unwrap_or(TrackedUser::new());
                            self.data.raw_mac = self.data.user.mac_address.to_hex_string();
                            // Why does this have to be Some(...)
                            // Might be perhaps emergency_contact is an optoin
                            self.data.user.emergency_contact = Some(self.data.emergency_user.clone().unwrap().id);
                        }
                        Err(e) => {
                            self.data.error_messages.push(format!("failed to find user, reason: {}", e));
                        }
                    }
                } else {
                    self.data.error_messages.push("failed to find user".to_string());
                }
            },
            Msg::ResponseAddUser(response) => {
                let (meta, Json(body)) = response.into_parts();
                if meta.status.is_success() {
                    match body {
                        Ok(result) => {
                            Log!("returned user is {:?}", result);
                            self.data.success_message = Some("successfully added user".to_string());
                            self.data.user = result;
                            // Does this line break any rust borrowing rules?
                            self.data.user.emergency_contact = Some(self.data.emergency_user.clone().unwrap().id);
                            self.data.id = Some(self.data.user.id);
                            self.data.raw_mac = self.data.user.mac_address.to_hex_string();
                        },
                        Err(e) => {
                            self.data.error_messages.push(format!("failed to add user, reason: {}", e));
                        }
                    }
                } else {
                    self.data.error_messages.push("failed to add user. Meta.status failed".to_string());
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
        // Backend assigns the chosen floor to the user.map_id
        /*let chosen_floor_id = match self.data.user.map_id {
            Some(id) => id,
            None => -1,
        }; */
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

        let mut errors = self.data.error_messages.iter().cloned().map(|msg| {
            html! {
                <p>{msg}</p>
            }
        });
        let employee_id = self.data.user.employee_id.clone().unwrap_or(String::new());
        let phone = self.data.user.phone.clone().unwrap_or(String::new());
        let mobile_phone = self.data.user.mobile_phone.clone().unwrap_or(String::new());
        let note = self.data.user.note.clone().unwrap_or(String::new());
        
        let emergency_name = self.data.emergency_user.clone().unwrap_or(TrackedUser::new()).name;
        // Isn't this used in the html code?
        let _emergency_employee_id = self.data.emergency_user.clone().unwrap_or(TrackedUser::new()).employee_id.unwrap_or(String::new());
        let emergency_mobile = self.data.emergency_user.clone().unwrap_or(TrackedUser::new()).phone.unwrap_or(String::new());
        let emergency_employee_id = self.data.emergency_user.clone().unwrap_or(TrackedUser::new()).mobile_phone.unwrap_or(String::new());
        let emergency_note = self.data.emergency_user.clone().unwrap_or(TrackedUser::new()).note.unwrap_or(String::new());

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
                        <td>{ "Employee ID: " }</td>
                        <td>
                            <input
                                type="text"
                                value=employee_id,
                                oninput=|e| Msg::InputEmployeeID(e.value),
                            />
                        </td>
                    </tr>
                    <tr>
                        <td>{ "Phone: " }</td>
                        <td>
                            <input
                                type="text",
                                value = phone,
                                oninput=|e| Msg::InputPhone(e.value)
                            />
                        </td>
                    </tr>
                    <tr>
                        <td>{ "Mobile Phone: " }</td>
                        <td>
                            <input
                                type="text",
                                value = mobile_phone,
                                oninput=|e| Msg::InputMobilePhone(e.value)
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
                    // Emergency Contact Information
                    <tr> { "Emergency Contact Information"} </tr>
                    <tr>
                        <td>{ "Name: " }</td>
                        <td>
                            <input
                                type="text",
                                value = emergency_name,
                                oninput=|e| Msg::InputEmergencyName(e.value)
                            />
                        </td>
                    </tr>
                    <tr>
                        <td>{ "Employee ID: (if applicable) " }</td>
                        <td>
                            <input
                                type="text",
                                value = emergency_employee_id,
                                oninput=|e| Msg::InputEmergencyEmployeeID(e.value)
                            />
                        </td>
                    </tr>
                    <tr>
                        <td>{ "Phone: " }</td>
                        <td>
                            <input
                                type="text",
                                value = emergency_mobile,
                                oninput=|e| Msg::InputEmergencyPhone(e.value)
                            />
                        </td>
                    </tr>
                    <tr>
                        <td>{ "Mobile Phone " }</td>
                        <td>
                            <input
                                type="text",
                                value = emergency_mobile,
                                oninput=|e| Msg::InputEmergencyMobilePhone(e.value)
                            />
                        </td>
                    </tr>
                    <tr>
                        <td>{ "Note: " }</td>
                        <td>
                            <textarea
                                rows=5,
                                value = emergency_note,
                                oninput=|e| Msg::InputEmergencyNote(e.value),
                            />
                        </td>
                    </tr>
                </table>
                <button onclick=|_| Msg::RequestAddUpdateUser,>{ submit_name }</button>
                { add_another_button }
            </>
        }
    }
}