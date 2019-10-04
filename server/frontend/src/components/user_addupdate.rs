use common::*;
use crate::util;
use yew::format::Json;
use yew::services::fetch::{ FetchService, FetchTask, };
use yew::{ Component, ComponentLink, Html, Renderable, ShouldRender, html, Properties};

#[derive(Copy, Clone)]
pub enum UserType {
    Normal,
    Contact,
}

pub enum Msg {
    AddAnotherUser,
    InputMacAddress(String),
    InputName(String, UserType),
    InputEmployeeID(String, UserType),
    InputWorkPhone(String, UserType),
    InputMobilePhone(String, UserType),
    InputNote(String, UserType),

    RequestAddUpdateUser,
    RequestGetUser(i32),
    ResponseAddUser(util::Response<(TrackedUser, Option<TrackedUser>)>),
    ResponseGetUser(util::Response<(Option<TrackedUser>, Option<TrackedUser>)>),
    ResponseUpdateUser(util::Response<(TrackedUser, Option<TrackedUser>)>),
}

struct Data {
    pub user: TrackedUser,
    pub emergency_user: Option<TrackedUser>,
    pub error_messages: Vec<String>,
    pub id: Option<i32>,
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
            raw_mac: ShortAddress::nil().to_string(),
            success_message: None,
        }
    }

    fn validate(&mut self) -> bool {
        let success = match ShortAddress::parse_str(&self.raw_mac) {
            Ok(m) => {
                self.user.mac_address = Some(m);
                true
            },
            Err(e) => {
                self.error_messages.push(format!("failed to parse mac address: {}", e));
                false
            }
        };
        success
    }
}

pub struct UserAddUpdate {
    data: Data,
    fetch_service: FetchService,
    fetch_task: Option<FetchTask>,
    get_fetch_task: Option<FetchTask>,
    self_link: ComponentLink<Self>,
}

#[derive(Clone, Default, PartialEq, Properties)]
pub struct UserAddUpdateProps {
    pub id: Option<i32>,
}

impl Component for UserAddUpdate {
    type Message = Msg;
    type Properties = UserAddUpdateProps;

    fn create(props: Self::Properties, mut link: ComponentLink<Self>) -> Self {
        if let Some(id) = props.id {
            link.send_self(Msg::RequestGetUser(id));
        }

        let mut result = UserAddUpdate {
            data: Data::new(),
            fetch_service: FetchService::new(),
            fetch_task: None,
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
                            Some(user) => user.work_phone = Some(work_phone),
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

            Msg::RequestAddUpdateUser => {
                self.data.error_messages = Vec::new();
                self.data.success_message = None;

                let success = self.data.validate();

                match self.data.id {
                    Some(id) if success => {
                        self.data.user.id = id;
                        if let Some(e_user) = &mut self.data.emergency_user {
                            // ensure the emergency user is attached to the user
                            e_user.attached_user = Some(id);
                        }

                        self.fetch_task = put_request!(
                            self.fetch_service,
                            &user_url(&self.data.user.id.to_string()),
                            (&self.data.user, &self.data.emergency_user),
                            self.self_link,
                            Msg::ResponseUpdateUser
                        );

                    },
                    None if success => {
                        self.fetch_task = post_request!(
                            self.fetch_service,
                            &user_url(""),
                            (&self.data.user, &self.data.emergency_user),
                            self.self_link,
                            Msg::ResponseAddUser
                        );
                    }
                    _ => {},
                }
            },
            Msg::RequestGetUser(id) => {
                self.get_fetch_task = get_request! (
                    self.fetch_service,
                    &format!("{}?prefetch=true", user_url(&id.to_string())),
                    self.self_link,
                    Msg::ResponseGetUser
                );
            },
            Msg::ResponseGetUser(response) => {
                let (meta, Json(body)) = response.into_parts();
                if meta.status.is_success() {
                    match body {
                        Ok((opt_user, opt_e_user)) => {
                            self.data.user = opt_user.unwrap_or(TrackedUser::new());
                            self.data.raw_mac = self.data.user.mac_address.map_or(String::new(), |m| m.to_string());
                            self.data.emergency_user = opt_e_user;
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
                        Ok((opt_user, opt_e_user)) => {
                            self.data.success_message = Some("successfully added user".to_string());
                            self.data.user = opt_user;
                            self.data.emergency_user = opt_e_user;

                            self.data.id = Some(self.data.user.id);
                            self.data.raw_mac = self.data.user.mac_address.map_or(String::new(), |m| m.to_string());
                        },
                        Err(e) => {
                            self.data.error_messages.push(format!("failed to add user, reason: {}", e));
                        }
                    }
                } else {
                    self.data.error_messages.push("failed to add user".to_string());
                }
            },
            Msg::ResponseUpdateUser(response) => {
                let (meta, Json(body)) = response.into_parts();
                if meta.status.is_success() {
                    match body {
                        Ok((opt_user, opt_e_user)) => {
                            self.data.success_message = Some("successfully updated user".to_string());
                            self.data.user = opt_user;
                            self.data.emergency_user = opt_e_user;
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

impl UserAddUpdate {
    fn render_input_form(&self, user: &TrackedUser, type_user: UserType) -> Html<Self> {

        html! {
            <>
                <tr>
                    <td>{ "Employee ID: " }</td>
                    <td>
                        <input
                            type="text"
                            value=user.employee_id.as_ref().unwrap_or(&String::new()),
                            oninput=|e| Msg::InputEmployeeID(e.value, type_user),
                        />
                    </td>
                </tr>
                <tr>
                    <td>{ "Work Phone: " }</td>
                    <td>
                        <input
                            type="text",
                            value=user.work_phone.as_ref().unwrap_or(&String::new()),
                            oninput=|e| Msg::InputWorkPhone(e.value, type_user)
                        />
                    </td>
                </tr>
                <tr>
                    <td>{ "Mobile Phone: " }</td>
                    <td>
                        <input
                            type="text",
                            value=user.mobile_phone.as_ref().unwrap_or(&String::new()),
                            oninput=|e| Msg::InputMobilePhone(e.value, type_user)
                        />
                    </td>
                </tr>
                <tr>
                    <td>{ "Note: " }</td>
                    <td>
                        <textarea
                            rows=5,
                            value = user.note.as_ref().unwrap_or(&String::new()),
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

        let mut errors = self.data.error_messages.iter().map(|msg| {
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
                    { self.render_input_form(&self.data.user, UserType::Normal) }
                    <tr> { "Emergency Contact Information"} </tr>
                    <tr>
                        <td>{ "Name: " }</td>
                        <td>
                            <input
                                type="text",
                                value=self.data.emergency_user.as_ref().map_or(&String::new(), |u| &u.name),
                                oninput=|e| Msg::InputName(e.value, UserType::Contact)
                            />
                        </td>
                    </tr>
                    {
                        match &self.data.emergency_user {
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