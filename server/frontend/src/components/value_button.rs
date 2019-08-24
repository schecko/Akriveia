
use yew::{ Component, Callback, ComponentLink, Html, Renderable, ShouldRender, html, };

pub enum Msg {
    Click,
}

pub struct ValueButton<T> {
    pub value: T,
    pub on_click: Option<Callback<T>>,
    pub disabled: bool,
    pub border: bool,
}

#[derive(Clone, PartialEq, Default)]
pub struct ValueButtonProps<T> {
    pub value: T,
    pub on_click: Option<Callback<T>>,
    pub disabled: bool,
    pub border: bool,
}

impl <T: 'static> Component for ValueButton<T>
    where T: std::default::Default + std::clone::Clone + std::cmp::PartialEq + std::fmt::Display
{
    type Message = Msg;
    type Properties = ValueButtonProps<T>;

    fn create(props: Self::Properties, mut _link: ComponentLink<Self>) -> Self {
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

impl <T: 'static> Renderable<ValueButton<T>> for ValueButton<T>
    where T: std::default::Default + std::clone::Clone + std::cmp::PartialEq + std::fmt::Display
{
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
