extern crate yew;

mod components;

use components::root::RootComponent;

fn main() {
    yew::start_app::<RootComponent>();
}

