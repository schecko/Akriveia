use yew::services::fetch::{ FetchService, FetchTask, Request, };
use yew::{ Component, ComponentLink, Html, Renderable, ShouldRender, html, };
use crate::util;
use yew::format::Json;
use common::*;

pub enum Msg {
    RequestPostBeacon,

    ResponsePostBeacon(util::Response<common::Beacon>),
}

pub struct BeaconAdd {
    beacon: common::Beacon,
    fetch_service: FetchService,
    fetch_task: Option<FetchTask>,
    self_link: ComponentLink<Self>,
}

#[derive(Clone, Default, PartialEq)]
pub struct BeaconAddProps {
}

impl Component for BeaconAdd {
    type Message = Msg;
    type Properties = BeaconAddProps;

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let result = BeaconAdd {
            beacon: common::Beacon::new(),
            fetch_service: FetchService::new(),
            fetch_task: None,
            self_link: link,
        };
        result
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::RequestPostBeacon => {
                self.fetch_task = post_request!(
                    self.fetch_service,
                    &beacon_url(""),
                    self.beacon,
                    self.self_link,
                    Msg::ResponsePostBeacon
                );
            },
            Msg::ResponsePostBeacon(response) => {
                let (meta, Json(body)) = response.into_parts();
                if meta.status.is_success() {
                    match body {
                        Ok(result) => {
                            Log!("returned beacon is {:?}", result);
                            self.beacon = result;
                        }
                        _ => { }
                    }
                } else {
                    Log!("response - failed to add beacon");
                }
            },
        }
        true
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        true
    }
}

impl Renderable<BeaconAdd> for BeaconAdd {
    fn view(&self) -> Html<Self> {
        html! {
            <>
                <p>{ "hello beacon add" }</p>
                <button onclick=|_| Msg::RequestPostBeacon,>{ "Add Beacon" }</button>
            </>
        }
    }
}
