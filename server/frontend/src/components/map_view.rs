
use stdweb::web::html_element::CanvasElement;
use stdweb::web::{ CanvasRenderingContext2d, Element, Node, };
use yew::services::fetch::{ FetchService, FetchTask, Request, Response, };
use yew::services::interval::{ IntervalTask, IntervalService, };
use yew::virtual_dom::vnode::VNode;
use yew::{ Component, ComponentLink, Html, Renderable, ShouldRender, html, };
use crate::util;
use std::time::Duration;
use yew::format::{ Nothing, Json };

const REALTIME_USER_POLL_RATE: Duration = Duration::from_millis(1000);

pub enum Msg {
    RequestRealtimeUser,

    ResponseRealtimeUser(util::Response<Vec<common::User>>),
}

pub struct MapViewComponent {
    context: CanvasRenderingContext2d,
    interval_service: IntervalService,
    interval_service_task: IntervalTask,
    fetch_service: FetchService,
    map_canvas: CanvasElement,
    self_link: ComponentLink<MapViewComponent>,
}

fn get_context(canvas: &CanvasElement) -> CanvasRenderingContext2d {
    unsafe {
        js! (
            return @{canvas}.getContext("2d");
        ).into_reference_unchecked().unwrap()
    }
}

impl Component for MapViewComponent {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, mut link: ComponentLink<Self>) -> Self {
        let canvas: CanvasElement = unsafe {
            js! (
                let c = document.createElement("canvas");
                c.setAttribute("id", "test_canvas");
                return c;
            ).into_reference_unchecked().unwrap()
        };
        canvas.set_width(800);
        canvas.set_height(800);
        let context = get_context(&canvas);
        context.fill_rect(0.0, 0.0, 70.0, 40.0);
        context.fill_rect(350.0, 350.0, 70.0, 40.0);

        let mut interval_service = IntervalService::new();
        let task = interval_service.spawn(REALTIME_USER_POLL_RATE, link.send_back(|_| Msg::RequestRealtimeUser));

        MapViewComponent {
            context: context,
            fetch_service: FetchService::new(),
            interval_service: interval_service,
            interval_service_task: task,
            map_canvas: canvas,
            self_link: link,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::RequestRealtimeUser => {
                get_request!(
                    self.fetch_service,
                    common::REALTIME_USERS,
                    self.self_link,
                    Msg::ResponseRealtimeUser
                );
            },
            Msg::ResponseRealtimeUser(data) => {
                self.context.clear_rect(0.0, 0.0, self.map_canvas.width().into(), self.map_canvas.height().into());
            }
        }

        true
    }

    fn change(&mut self, _: Self::Properties) -> ShouldRender {
        // do not overwrite the canvas or context.
        true
    }
}

impl Renderable<MapViewComponent> for MapViewComponent {
    fn view(&self) -> Html<Self> {
        html! {
            <div>
                <p> { "hello map" } </p>
                { VNode::VRef(Node::from(self.map_canvas.to_owned()).to_owned()) }
            </div>
        }
    }
}
