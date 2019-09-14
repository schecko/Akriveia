use common::*;
use crate::util;
use super::root;
use super::value_button::ValueButton;
use yew::format::Json;
use yew::services::fetch::{ FetchService, FetchTask, };
use yew::{ Callback, Component, ComponentLink, Html, Renderable, ShouldRender, html, };

pub enum Msg {
    ChangeRootPage(root::Page),

    RequestDeleteUser(i32),
    RequestGetUsers,

    ResponseGetUsers(util::Response<Vec<(TrackedUser, Map)>>),
    ResponseDeleteUser(util::Response<Vec<()>>),
}

pub struct UserList {
    change_page: Option<Callback<root::Page>>,
    fetch_service: FetchService,
    fetch_task: Option<FetchTask>,
    list: Vec<(TrackedUser, Map)>,
    self_link: ComponentLink<Self>,
}

#[derive(Clone, Default, PartialEq)]
pub struct UserListProps {
    pub change_page: Option<Callback<root::Page>>,
}

impl Component for UserList {
    type Message = Msg;
    type Properties = UserListProps;

    fn create(props: Self::Properties, mut link: ComponentLink<Self>) -> Self {
        link.send_self(Msg::RequestGetUsers);
        let result = UserList {
            fetch_service: FetchService::new(),
            list: Vec::new(),
            fetch_task: None,
            self_link: link,
            change_page: props.change_page,
        };
        result
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::RequestGetUsers => {
                self.fetch_task = get_request!(
                    self.fetch_service,
                    &format!("{}?prefetch=true", users_url()),
                    self.self_link,
                    Msg::ResponseGetUsers
                );
            },
            Msg::RequestDeleteUser(id) => {
                self.fetch_task = delete_request!(
                    self.fetch_service,
                    &user_url(&id.to_string()),
                    self.self_link,
                    Msg::ResponseDeleteUser
                );
            },
            Msg::ResponseGetUsers(response) => {
                let (meta, Json(body)) = response.into_parts();
                if meta.status.is_success() {
                    match body {
                        Ok(users_and_maps) => {
                            self.list = users_and_maps;
                        }
                        _ => { }
                    }
                } else {
                    Log!("response - failed to obtain User list");
                }
            },
            Msg::ResponseDeleteUser(response) => {
                let (meta, Json(body)) = response.into_parts();
                if meta.status.is_success() {
                    match body {
                        Ok(_list) => {
                            Log!("successfully deleted User");
                        }
                        _ => { }
                    }
                } else {
                    Log!("response - failed to delete User");
                }
                // now that the User is deleted, get the updated list
                self.self_link.send_self(Msg::RequestGetUsers);
            },
            Msg::ChangeRootPage(page) => {
                self.change_page.as_mut().unwrap().emit(page);
            }
        }
        true
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        self.self_link.send_self(Msg::RequestGetUsers);
        true
    }
}

impl Renderable<UserList> for UserList {
    fn view(&self) -> Html<Self> {

        let mut rows = self.list.iter().map(|(user, map)| {
            let employee_id = &user.employee_id.clone().unwrap_or(String::new());
            let phone = &user.phone.clone().unwrap_or(String::new());
            let mobile_phone = &user.mobile_phone.clone().unwrap_or(String::new());
            let note = &user.note.clone().unwrap_or(String::new());

            html! {
                <tr>
                    <td>{ &user.mac_address.to_hex_string() }</td>
                    <td>{ format!("{},{}", &user.coordinates.x, &user.coordinates.y) }</td>
                    <td>{ &map.name }</td>
                    <td>{ &user.name }</td>
                    <td>{ employee_id }</td>
                    <td>{ &user.last_active}</td>
                    <td>{ phone }</td>
                    <td>{ mobile_phone}</td>
                    <td>{ note }</td>
                    <td>
                        <ValueButton<i32>
                            display=Some("Edit".to_string()),
                            on_click=|value: i32| Msg::ChangeRootPage(root::Page::UserAddUpdate(Some(value))),
                            border=false,
                            value={user.id}
                        />
                        <ValueButton<i32>
                            display=Some("Delete".to_string()),
                            on_click=|value: i32| Msg::RequestDeleteUser(value),
                            border=false,
                            value=user.id
                        />
                    </td>
                </tr>
            }
        });

        html! {
            <>
                <p>{ "User List" }</p>
                <table>
                <tr>
                    <td>{ "Mac" }</td>
                    <td>{ "Coordinates" }</td>
                    <td>{ "Floor" }</td>
                    <td>{ "Name" }</td>
                    <td>{ "Employee ID" }</td>
                    <td>{ "Last Acive" }</td>
                    <td>{ "Phone" }</td>
                    <td>{ "Mobile Phone" }</td>
                    <td>{ "Note" }</td> 
                </tr>
                { for rows }
                </table>
            </>
        }
    }
}
