
use yew::{html, Component, ComponentLink, Html, Renderable, ShouldRender};
use stdweb::web::html_element::CanvasElement;
use stdweb::web::CanvasRenderingContext2d;
use yew::virtual_dom::vnode::VNode;
use stdweb::web::Element;
use stdweb::web::Node;

pub enum Msg {
    HelloView,
}

pub struct MapViewComponent {
    map_canvas: CanvasElement,
    self_link: ComponentLink<MapViewComponent>,
    context: CanvasRenderingContext2d,
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

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
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
        MapViewComponent {
            map_canvas: canvas,
            self_link: link,
            context: context,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::HelloView => { },
        }

        true
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
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
