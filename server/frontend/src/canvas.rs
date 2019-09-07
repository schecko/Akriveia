

fn get_context(canvas: &CanvasElement) -> CanvasRenderingContext2d {
    unsafe {
        js! (
            return @{canvas}.getContext("2d");
        ).into_reference_unchecked().unwrap()
    }
}

pub fn reset_canvas(canvas: CanvasRenderingContext2d, canvas: CanvasElement, map: Map) {
    canvas.set_width(map.bounds[0] as u32);
    canvas.set_height(map.bounds[1] as u32);

    context.set_line_dash(vec![]);
    context.clear_rect(0.0, 0.0, canvas.width().into(), canvas.height().into());
    context.stroke_rect(0.0, 0.0, canvas.width().into(), canvas.height().into());

    context.save();
    context.set_line_dash(vec![5.0, 15.0]);
    // vertical gridlines
    for i in (MAP_SCALE as u32..MAP_WIDTH as u32).step_by(MAP_SCALE as usize) {
        let pos0 = screen_space(i as f64, MAP_HEIGHT as f64);
        let pos1 = screen_space(i as f64, 0.0);
        context.begin_path();
        context.move_to(pos0.x, pos0.y);
        context.line_to(pos1.x, pos1.y);
        context.stroke();
    }
    // horizontal gridlines
    for i in (MAP_SCALE as u32..MAP_HEIGHT as u32).step_by(MAP_SCALE as usize) {
        let pos0 = screen_space(MAP_WIDTH as f64, i as f64);
        let pos1 = screen_space(0.0, i as f64);
        context.begin_path();
        context.move_to(pos0.x, pos0.y);
        context.line_to(pos1.x, pos1.y);
        context.stroke();
    }
    context.restore();

    let text_adjustment = 10.0;
    // x axis
    for i in 0..(MAP_WIDTH / MAP_SCALE as u32) {
        let pos = screen_space(i as f64 * MAP_SCALE + text_adjustment, text_adjustment);
        context.fill_text(&format!("{}m", i), pos.x, pos.y, None);
    }
    // y axis
    // skip 0 because it was rendered by the y axis.
    for i in 1..(MAP_HEIGHT / MAP_SCALE as u32) {
        let pos = screen_space(text_adjustment, i as f64 * MAP_SCALE + text_adjustment);
        context.fill_text(&format!("{}m", i), pos.x, pos.y, None);
    }
}
