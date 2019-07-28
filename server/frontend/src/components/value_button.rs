
use yew::{ Component, Callback, ComponentLink, Html, Renderable, ShouldRender, html, };
use yew::services::console::ConsoleService;

pub enum Msg {
    Click,
}

pub struct ValueButton {
    pub value: String,
    pub on_click: Option<Callback<String>>,
    pub disabled: bool,
    pub border: bool,
}

#[derive(Clone, PartialEq, Default)]
pub struct ValueButtonProps {
    pub value: String,
    pub on_click: Option<Callback<String>>,
    pub disabled: bool,
    pub border: bool,
}

impl Component for ValueButton {
    type Message = Msg;
    type Properties = ValueButtonProps;

    fn create(props: Self::Properties, mut link: ComponentLink<Self>) -> Self {
        ValueButton {
            value: props.value,
            on_click: props.on_click,
            disabled: props.disabled,
            border: props.border,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Click => {
                self.on_click.as_mut().unwrap().emit(self.value.clone());
            },
        }
        true
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.value = props.value;
        self.on_click = props.on_click;
        self.disabled = props.disabled;
        self.border = props.border;
        true
    }
}

impl Renderable<ValueButton> for ValueButton {
    fn view(&self) -> Html<Self> {
        let cls = if self.border { "bold_font" } else { "" };

        html! {
            <>
                <button
                    disabled={self.disabled}
                    onclick=|_| Msg::Click,
                    class={cls},
                >
                     { &self.value }
                </button>
            </>
        }
    }
}
