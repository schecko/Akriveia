use yew::services::fetch::{ FetchService, FetchTask, Request, };
use yew::{ Component, ComponentLink, Html, Renderable, ShouldRender, html, };
use crate::util;
use yew::format::Json;
use common::*;

pub enum Msg {
    RequestPutBeacon,

    ResponsePutBeacon(util::Response<common::Beacon>),
}

pub struct BeaconUpdate {
    beacon: common::Beacon,
    fetch_service: FetchService,
    fetch_task: Option<FetchTask>,
    self_link: ComponentLink<Self>,
}

#[derive(Clone, Default, PartialEq)]
pub struct BeaconUpdateProps {
}

impl Component for BeaconUpdate {
    type Message = Msg;
    type Properties = BeaconUpdateProps;

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let result = BeaconUpdate {
            beacon: common::Beacon::new(),
            fetch_service: FetchService::new(),
            fetch_task: None,
            self_link: link,
        };
        result
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::RequestPutBeacon => {
                self.fetch_task = put_request!(
                    self.fetch_service,
                    &beacon_url(self.id.to_string()),
                    self.beacon,
                    self.self_link,
                    Msg::ResponsePutBeacon
                );
            },
            Msg::ResponsePutBeacon(response) => {
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
                    Log!("response - failed to update beacon");
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
                <p>{ "beacon update" }</p>
                <button onclick=|_| Msg::RequestUpdateBeacon,>{ "Update Beacon" }</button>
            </>
        }
    }
}
