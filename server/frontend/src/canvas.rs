use common::*;
//use crate::util;
use na;
use stdweb::web::html_element::CanvasElement;
use stdweb::web::{ CanvasRenderingContext2d, };

pub fn get_context(canvas: &CanvasElement) -> CanvasRenderingContext2d {
    unsafe {
        js! (
            return @{canvas}.getContext("2d");
        ).into_reference_unchecked().unwrap()
    }
}

pub fn make_canvas(id: &str) -> CanvasElement {
    let canvas: CanvasElement = unsafe {
        js! (
            let c = document.createElement("canvas");
            c.setAttribute("id", @{id});
            return c;
        ).into_reference_unchecked().unwrap()
    };
    canvas
}

pub fn screen_space(map: &Map, x: f64, y: f64) -> na::Vector2<f64> {
    na::Vector2::new(x, map.bounds.y as f64 - y)
}

pub fn reset_canvas(canvas: &CanvasElement, context: &CanvasRenderingContext2d, map: &Map) {
    canvas.set_width(map.bounds[0] as u32);
    canvas.set_height(map.bounds[1] as u32);

    context.set_line_dash(vec![]);
    context.clear_rect(0.0, 0.0, canvas.width().into(), canvas.height().into());
    context.stroke_rect(0.0, 0.0, canvas.width().into(), canvas.height().into());

    context.save();
    context.set_line_dash(vec![5.0, 15.0]);
    // vertical gridlines
    for i in (map.scale as u32..map.bounds.x as u32).step_by(map.scale as usize) {
        let pos0 = screen_space(&map, i as f64, map.bounds.y as f64);
        let pos1 = screen_space(&map, i as f64, 0.0);
        context.begin_path();
        context.move_to(pos0.x, pos0.y);
        context.line_to(pos1.x, pos1.y);
        context.stroke();
    }
    // horizontal gridlines
    for i in (map.scale as u32..map.bounds.y as u32).step_by(map.scale as usize) {
        let pos0 = screen_space(&map, map.bounds.x as f64, i as f64);
        let pos1 = screen_space(&map, 0.0, i as f64);
        context.begin_path();
        context.move_to(pos0.x, pos0.y);
        context.line_to(pos1.x, pos1.y);
        context.stroke();
    }
    context.restore();

    let text_adjustment = 10.0;
    // x axis
    for i in 0..(map.bounds.x as u32 / map.scale as u32) {
        let pos = screen_space(&map, i as f64 * map.scale + text_adjustment, text_adjustment);
        context.fill_text(&format!("{}m", i), pos.x, pos.y, None);
    }
    // y axis
    // skip 0 because it was rendered by the y axis.
    for i in 1..(map.bounds.y as u32 / map.scale as u32) {
        let pos = screen_space(&map, text_adjustment, i as f64 * map.scale + text_adjustment);
        context.fill_text(&format!("{}m", i), pos.x, pos.y, None);
    }
}
