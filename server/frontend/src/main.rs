extern crate yew;

use yew::{html, Component, ComponentLink, Html, Renderable, ShouldRender};

struct Model {
}

enum Msg {
    Click,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, _: ComponentLink<Self>) -> Self {
        Model { }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Click => {
            }
        }
        true
    }
}

impl Renderable<Model> for Model {
    fn view(&self) -> Html<Self> {
        html! {
            <div>
                <button onclick=|_| Msg::Click,>{ "Click" }</button>
            </div>
        }
    }
}

fn main() {
    yew::start_app::<Model>();
}

