use common::*;
use crate::util;
use super::root;
use super::value_button::ValueButton;
use yew::format::Json;
use yew::services::fetch::{ FetchService, FetchTask, };
use yew::prelude::*;
use super::user_message::UserMessage;

pub enum Msg {
    ChangeRootPage(root::Page),

    RequestDeleteMap(i32),
    RequestGetMaps,

    ResponseGetMaps(util::Response<Vec<Map>>),
    ResponseDeleteMap(util::Response<Vec<()>>),
}

pub struct MapList {
    change_page: Callback<root::Page>,
    fetch_service: FetchService,
    fetch_task: Option<FetchTask>,
    list: Vec<Map>,
    self_link: ComponentLink<Self>,
    user_msg: UserMessage<Self>,
}

#[derive(Properties)]
pub struct MapListProps {
    #[props(required)]
    pub change_page: Callback<root::Page>,
}

impl Component for MapList {
    type Message = Msg;
    type Properties = MapListProps;

    fn create(props: Self::Properties, mut link: ComponentLink<Self>) -> Self {
        link.send_self(Msg::RequestGetMaps);
        let result = MapList {
            fetch_service: FetchService::new(),
            list: Vec::new(),
            fetch_task: None,
            self_link: link,
            change_page: props.change_page,
            user_msg: UserMessage::new(),
        };
        result
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::RequestGetMaps => {
                self.fetch_task = get_request!(
                    self.fetch_service,
                    &maps_url(),
                    self.self_link,
                    Msg::ResponseGetMaps
                );
            },
            Msg::RequestDeleteMap(id) => {
                self.fetch_task = delete_request!(
                    self.fetch_service,
                    &map_url(&id.to_string()),
                    self.self_link,
                    Msg::ResponseDeleteMap
                );
            },
            Msg::ResponseGetMaps(response) => {
                let (meta, Json(body)) = response.into_parts();
                if meta.status.is_success() {
                    match body {
                        Ok(mut maps) => {
                            maps.sort_unstable_by(|a, b| a.name.cmp(&b.name));
                            self.list = maps;
                        }
                        _ => { }
                    }
                } else {
                    self.user_msg.error_messages.push("failed to obtain map list".to_owned());
                }
            },
            Msg::ResponseDeleteMap(response) => {
                let (meta, Json(body)) = response.into_parts();
                if meta.status.is_success() {
                    match body {
                        Ok(_list) => {
                            self.user_msg.success_message = Some("successfully deleted user".to_owned());
                        },
                        _ => { }
                    }
                } else {
                    self.user_msg.error_messages.push("failed to delete user".to_owned());
                }
                // now that the map is deleted, get the updated list
                self.self_link.send_self(Msg::RequestGetMaps);
            },
            Msg::ChangeRootPage(page) => {
                self.change_page.emit(page);
            }
        }
        true
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        self.self_link.send_self(Msg::RequestGetMaps);
        true
    }
}

impl Renderable<MapList> for MapList {
    fn view(&self) -> Html<Self> {
        let mut rows = self.list.iter().map(|map| {
            html! {
                <tr>
                    <td>{ &map.name }</td>
                    <td>{ format!("{},{}", &map.bounds.x, &map.bounds.y) }</td>
                    <td>{ map.scale }</td>
                    <td>{ map.note.clone().unwrap_or(String::new()) }</td>
                    <td>
                        <ValueButton<i32>
                            display=Some("Edit".to_string()),
                            on_click=|value: i32| Msg::ChangeRootPage(root::Page::MapAddUpdate(Some(value))),
                            border=false,
                            value=map.id
                        />
                        <ValueButton<i32>
                            display=Some("Delete".to_string()),
                            on_click=|value: i32| Msg::RequestDeleteMap(value),
                            border=false,
                            value=map.id
                        />
                        <ValueButton<i32>
                            display=Some("View".to_string()),
                            on_click=|value: i32| Msg::ChangeRootPage(root::Page::MapView(Some(value))),
                            border=false,
                            value=map.id
                        />
                    </td>
                </tr>
            }
        });

        html! {
            <>
                { self.user_msg.view() }
                <table class="table table-striped">
                    <thead class="thead-light">
                        <h2>{ "Map List" }</h2>
                        <tr>
                            <th>{ "Name" }</th>
                            <th>{ "Bounds" }</th>
                            <th>{ "Scale" }</th>
                            <th>{ "Note" }</th>
                            <th>{ "Actions" }</th>
                        </tr>
                    </thead>
                    <tbody>
                        { for rows }
                    </tbody>
                </table>
            </>
        }
    }
}
