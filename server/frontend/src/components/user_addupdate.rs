use common::*;
use crate::util;
use yew::format::Json;
use yew::services::fetch::{ FetchService, FetchTask, };
use yew::{ Component, ComponentLink, Html, Renderable, ShouldRender, html, };

#[derive(Copy, Clone)]
pub enum UserType {
    Normal,
    Contact,
}

pub enum Msg {
    AddAnotherUser,
    
    InputMacAddress(String),
    // bool is True if user else emergency user   
    InputName(String, UserType),
    InputEmployeeID(String, UserType),
    InputWorkPhone(String, UserType),
    InputMobilePhone(String, UserType),
    InputNote(String, UserType),

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
            Msg::InputName(name, usertype) => {
                match usertype {
                    UserType::Normal => self.data.user.name = name,
                    UserType::Contact => {
                        match &mut self.data.emergency_user {
                            Some(user) => user.name = name,
                            None => {
                                let mut new_user = TrackedUser::new();
                                new_user.name = name;
                                self.data.emergency_user = Some(new_user);
                            }
                        }
                    }
                }
            },
            Msg::InputEmployeeID(employee_id, usertype) => {
                match usertype {
                    UserType::Normal => self.data.user.employee_id = Some(employee_id),
                    UserType::Contact => {
                        match &mut self.data.emergency_user {
                            Some(user) => user.employee_id = Some(employee_id),
                            None => {},
                        }
                    }
                }            
            },
            Msg::InputWorkPhone(work_phone, usertype) => {
                match usertype {
                    UserType::Normal => self.data.user.work_phone = Some(work_phone),
                    UserType::Contact => {
                        match &mut self.data.emergency_user {
                            Some(user) => user.work_phone= Some(work_phone),
                            None => {},
                        }
                    }
                }
            },
            Msg::InputMobilePhone(mobile_phone, usertype) => {
                match usertype {
                    UserType::Normal => self.data.user.mobile_phone = Some(mobile_phone),
                    UserType::Contact => {
                        match &mut self.data.emergency_user {
                            Some(user) => user.mobile_phone = Some(mobile_phone),
                            None => {},
                        }
                    }
                }
            },
            Msg::InputNote(note, usertype) => {
                match usertype {
                    UserType::Normal => self.data.user.note = Some(note),
                    UserType::Contact => {
                        match &mut self.data.emergency_user {
                            Some(user) => user.note = Some(note),
                            None => {},
                        }
                    }
                }
            },
            // Send a web parameter as in beacon_list.rs
            // self.fetch_service,
            // &format!("{}?emergency=true", user_url()),

            Msg::RequestAddUpdateUser => {
                self.data.error_messages = Vec::new();
                self.data.success_message = None;

                // Do we need to validate the data?
                let success = self.data.validate();
                let many_users = (&self.data.user, &self.data.emergency_user);

                match self.data.id {
                    Some(id) if success => {
                        // Ensure the beacon id does not mismatch.
                        // Where do we compare the list of user ID tags?
                        self.data.user.id = id;

                         self.fetch_task = put_request!(
                            self.fetch_service,
                            &user_url(&self.data.user.id.to_string()),
                            self.data.user,
                            self.self_link,
                            Msg::ResponseUpdateUser
                        );

                    },
                    // Is this the same service that is called in main.rs?
                    None if success => {
                        self.fetch_task = post_request!(
                            self.fetch_service,
                            &user_url(""),
                            many_users,
                            self.self_link,
                            Msg::ResponseAddUser
                        );
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
                            // How do I set it where it returns None if
                            match &self.data.emergency_user {
                                Some(e_user) => {
                                    self.data.user.emergency_contact = Some(e_user.id);  
                                }
                                None => {
                                    self.data.user.emergency_contact = None;
                                }
                            }
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

// should user be a reference or should it just be the value?
// b/c it doesn't matter what is being passed through value

impl UserAddUpdate {
    fn render_input_form(&self, user: &TrackedUser, type_user: UserType) -> Html<Self> {
        
        html! {
            <>
                <tr>
                    <td>{ "Employee ID: " }</td>
                    <td>
                        <input
                            type="text"
                            // string literal is immutable; stored on the stack
                            // Is there a faster method to copy the input of the keyed input
                            // Is value then a placeholder?
                            value = user.employee_id.clone().unwrap_or(String::new()),
                            oninput=|e| Msg::InputEmployeeID(e.value, type_user),
                        />
                    </td>
                </tr>
                <tr>
                    <td>{ "Work Phone: " }</td>
                    <td>
                        <input
                            type="text",
                            value = user.work_phone.clone().unwrap_or(String::new()),
                            oninput=|e| Msg::InputWorkPhone(e.value, type_user)
                        />
                    </td>
                </tr>
                <tr>
                    <td>{ "Mobile Phone: " }</td>
                    <td>
                        <input
                            type="text",
                            value = user.mobile_phone.clone().unwrap_or(String::new()),
                            oninput=|e| Msg::InputMobilePhone(e.value, type_user)
                        />
                    </td>
                </tr>                    
                <tr>
                    <td>{ "Note: " }</td>
                    <td>
                        <textarea
                            rows=5,
                            value = user.note.clone().unwrap_or(String::new()),
                            oninput=|e| Msg::InputNote(e.value, type_user),
                        />
                    </td>
                </tr>
            </>
        }
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
                                oninput=|e| Msg::InputName(e.value, UserType::Normal),
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
                    {   //let normal_user = &self.data.user.clone();
                        self.render_input_form(&self.data.user, UserType::Normal )}
                    // Emergency Contact Informationemergency_user
                    <tr> { "Emergency Contact Information"} </tr>
                    <tr>
                        <td>{ "Name: " }</td>
                        <td>
                            <input
                                type="text",
                                // Do you clone the reference to data.emergency_user or clone emergency_user 
                                value = self.data.emergency_user.clone().unwrap_or(TrackedUser::new()).name,
                                oninput=|e| Msg::InputName(e.value, UserType::Contact)
                            />
                        </td>
                    </tr>
                    {match &self.data.emergency_user {
                        Some(emergency_contact) => self.render_input_form(&emergency_contact, UserType::Contact),
                        None => {
                            html!{
                                <></>
                            }
                        },
                    }
                }
                </table>
                <button onclick=|_| Msg::RequestAddUpdateUser,>{ submit_name }</button>
                { add_another_button }
            </>
        }
    }
}