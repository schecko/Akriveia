use common::*;
use crate::util;
use super::root;
use super::value_button::ValueButton;
use yew::format::Json;
use yew::services::fetch::{ FetchService, FetchTask, };
use yew::prelude::*;

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
                        Ok(maps) => {
                            self.list = maps;
                        }
                        _ => { }
                    }
                } else {
                    Log!("response - failed to obtain map list");
                }
            },
            Msg::ResponseDeleteMap(response) => {
                let (meta, Json(body)) = response.into_parts();
                if meta.status.is_success() {
                    match body {
                        Ok(_list) => {
                            Log!("successfully deleted map");
                        }
                        _ => { }
                    }
                } else {
                    Log!("response - failed to delete map");
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
                <p>{ "Map List" }</p>
                <table>
                <tr>
                    <td>{ "Name" }</td>
                    <td>{ "Bounds" }</td>
                    <td>{ "Scale" }</td>
                    <td>{ "Note" }</td>
                    <td>{ "Actions" }</td>
                </tr>
                { for rows }
                </table>
            </>
        }
    }
}
