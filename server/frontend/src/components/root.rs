
use yew::{html, Component, ComponentLink, Html, Renderable, ShouldRender};

pub enum Page {
    Login,
    FrontPage,
}

pub struct RootComponent {
    current_page: Page,
}


pub enum Msg {
    ChangePage(Page),
}

impl Component for RootComponent {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, _: ComponentLink<Self>) -> Self {
        RootComponent {
            current_page: Page::Login,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        // TODO clean this up.
        match msg {
            Msg::ChangePage(page) => {
                self.current_page = page;
            }
        }
        true
    }
}

impl Renderable<RootComponent> for RootComponent {
    fn view(&self) -> Html<Self> {
        match self.current_page {
            Page::Login => {
                html! {
                    <div>
                        <p>{ "Hello Login Page!" }</p>
                        <button onclick=|_| Msg::ChangePage(Page::FrontPage),>{ "Click" }</button>
                    </div>
                }
            }
            Page::FrontPage => {
                html! {
                    <div>
                        <p>{ "Hello FrontPage Page!" }</p>
                        <button onclick=|_| Msg::ChangePage(Page::Login),>{ "Click" }</button>
                    </div>
                }
            }
        }
    }
}

