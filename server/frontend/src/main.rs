extern crate yew;
extern crate common;
#[macro_use]
extern crate stdweb;

#[macro_use]
mod util;
mod components;

use components::root::RootComponent;
use stdweb::web::{IElement, INode, IParentNode, document};

fn main() {
    yew::initialize();
    let body = document().query_selector("body").unwrap().unwrap();


    let mount_point = document().create_element("div").unwrap();
    mount_point.class_list().add("main_div").unwrap();
    body.append_child(&mount_point);

    let canvas = document().create_element("canvas").unwrap();
    canvas.class_list().add("map_canvas").unwrap();
    body.append_child(&canvas);


    yew::App::<RootComponent>::new().mount(mount_point);
    yew::run_loop();
}

