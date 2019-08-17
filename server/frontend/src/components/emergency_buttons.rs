
use yew::prelude::*;
use yew::Component;

pub enum Msg {
    RequestEmergency,
    RequestEndEmergency,
}

pub struct EmergencyButtons {
    is_emergency: bool,
    on_emergency: Option<Callback<()>>,
    on_end_emergency: Option<Callback<()>>,
}

#[derive(PartialEq, Clone, Default)]
pub struct EmergencyButtonsProps {
    pub is_emergency: bool,
    pub on_emergency: Option<Callback<()>>,
    pub on_end_emergency: Option<Callback<()>>,
}

impl Component for EmergencyButtons {
    type Message = Msg;
    type Properties = EmergencyButtonsProps;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        let result = EmergencyButtons {
            is_emergency: props.is_emergency,
            on_emergency: props.on_emergency,
            on_end_emergency: props.on_end_emergency,
        };

        result
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::RequestEmergency => {
                self.on_emergency.as_mut().unwrap().emit(())
            },
            Msg::RequestEndEmergency => self.on_end_emergency.as_mut().unwrap().emit(()),
        }
        true
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.is_emergency = props.is_emergency;
        self.on_emergency = props.on_emergency;
        self.on_end_emergency = props.on_end_emergency;
        true
    }
}

impl Renderable<EmergencyButtons> for EmergencyButtons {
    fn view(&self) -> Html<Self> {
        html! {
            <>
                <button
                    onclick=|_| Msg::RequestEmergency,
                    disabled={self.is_emergency},
                >
                    { "Start Emergency" }
                </button>
                <button
                    onclick=|_| Msg::RequestEndEmergency,
                    disabled={!self.is_emergency},
                >
                    { "End Emergency" }
                </button>
            </>
        }
    }
}
