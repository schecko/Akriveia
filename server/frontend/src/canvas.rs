use common::*;
use stdweb::traits::*;
use na;
use stdweb::web::event::{ ClickEvent, };
use stdweb::web::html_element::CanvasElement;
use stdweb::web::{ CanvasRenderingContext2d, FillRule, };
use yew::prelude::*;
use palette::{ Gradient, LinSrgb, };

const USER_RADIUS: f64 = 5.0;
const BEACON_RADIUS: f64 = 8.0;

pub struct Canvas {
    pub canvas: CanvasElement,
    pub context: CanvasRenderingContext2d,
}

pub fn screen_space(map: &Map, x: f64, y: f64) -> na::Vector2<f64> {
    na::Vector2::new(x, map.bounds.y as f64 - y)
}

fn color_to_hex(c: &LinSrgb<f64>) -> String {
    let comps = c.into_components();
    let color_string = format!(
        "#{:0<2X}{:0<2X}{:0<2X}",
        (comps.0 * 255.0) as u8,
        (comps.1 * 255.0) as u8,
        (comps.2 * 255.0) as u8
    );
    color_string
}

impl Canvas {
    fn get_context(canvas: &CanvasElement) -> CanvasRenderingContext2d {
        unsafe {
            js! (
                return @{canvas}.getContext("2d");
            ).into_reference_unchecked().unwrap()
        }
    }

    fn make_canvas(id: &str) -> CanvasElement {
        let canvas: CanvasElement = unsafe {
            js! (
                let c = document.createElement("canvas");
                c.setAttribute("id", @{id});
                return c;
            ).into_reference_unchecked().unwrap()
        };
        canvas
    }

    pub fn new(id: &str, click_callback: Callback<ClickEvent>) -> Canvas {
        let canvas = Canvas::make_canvas(id);
        let context = Canvas::get_context(&canvas);
        canvas.add_event_listener(move |event| click_callback.emit(event));
        Canvas {
            canvas,
            context,
        }
    }

    pub fn legend(&self, width: u32, height: u32, grad_colors: Vec<LinSrgb<f64>>) {
        self.canvas.set_width(width);
        self.canvas.set_height(height);

        // create a gradient from the top middle to the bottom middle.
        let grad = self.context.create_linear_gradient(width as f64 / 2.0f64, 0.0, width as f64 / 2.0, height.into());

        grad_colors.iter().enumerate().for_each(|(i, color)| {
            grad.add_color_stop((i as f64 + 0.5) / grad_colors.len() as f64, &color_to_hex(color)).unwrap();
        });
        self.context.set_fill_style_gradient(&grad);
        self.context.fill_rect(0.0, 0.0, width.into(), height.into());

    }

    pub fn reset(&mut self, map: &Map) {
        self.canvas.set_width(map.bounds[0] as u32);
        self.canvas.set_height(map.bounds[1] as u32);

        self.context.set_line_dash(vec![]);
        self.context.clear_rect(0.0, 0.0, self.canvas.width().into(), self.canvas.height().into());
        self.context.stroke_rect(0.0, 0.0, self.canvas.width().into(), self.canvas.height().into());

        self.context.save();
        self.context.set_line_dash(vec![5.0, 15.0]);
        // vertical gridlines
        for i in (map.scale as u32..map.bounds.x as u32).step_by(map.scale as usize) {
            let pos0 = screen_space(&map, i as f64, map.bounds.y as f64);
            let pos1 = screen_space(&map, i as f64, 0.0);
            self.context.begin_path();
            self.context.move_to(pos0.x, pos0.y);
            self.context.line_to(pos1.x, pos1.y);
            self.context.stroke();
        }
        // horizontal gridlines
        for i in (map.scale as u32..map.bounds.y as u32).step_by(map.scale as usize) {
            let pos0 = screen_space(&map, map.bounds.x as f64, i as f64);
            let pos1 = screen_space(&map, 0.0, i as f64);
            self.context.begin_path();
            self.context.move_to(pos0.x, pos0.y);
            self.context.line_to(pos1.x, pos1.y);
            self.context.stroke();
        }
        self.context.restore();

        let text_adjustment = 10.0;
        // x axis
        for i in 0..(map.bounds.x as u32 / map.scale as u32) {
            let pos = screen_space(&map, i as f64 * map.scale + text_adjustment, text_adjustment);
            self.context.fill_text(&format!("{}m", i), pos.x, pos.y, None);
        }
        // y axis
        // skip 0 because it was rendered by the y axis.
        for i in 1..(map.bounds.y as u32 / map.scale as u32) {
            let pos = screen_space(&map, text_adjustment, i as f64 * map.scale + text_adjustment);
            self.context.fill_text(&format!("{}m", i), pos.x, pos.y, None);
        }
    }

    pub fn draw_beacons(&mut self, map: &Map, beacons: &Vec<Beacon>) {
        self.context.save();
        for beacon in beacons {
            let beacon_loc = screen_space(
                &map,
                beacon.coordinates.x * map.scale,
                beacon.coordinates.y * map.scale,
            );
            self.context.set_fill_style_color("#0000FFFF");
            self.context.begin_path();
            self.context.arc(beacon_loc.x, beacon_loc.y, BEACON_RADIUS, 0.0, std::f64::consts::PI * 2.0, true);
            self.context.fill(FillRule::NonZero);
        }
        self.context.restore();
    }

    pub fn draw_users(&mut self, map: &Map, users: &Vec<RealtimeUserData>, show_distance: Option<ShortAddress>) {
        self.context.save();
        for user in users.iter() {
            let user_pos = screen_space(
                map,
                user.coordinates.x as f64 * map.scale,
                user.coordinates.y as f64 * map.scale,
            );

            for beacon_source in &user.beacon_tofs {
                let beacon_loc = screen_space(
                    map,
                    beacon_source.location.x * map.scale,
                    beacon_source.location.y * map.scale,
                );
                let diff = stdweb::web::Date::now() - user.last_active.timestamp_millis() as f64;
                let grad = Gradient::new(vec![
                    LinSrgb::new(1.0f64, 0.0, 1.0),
                    LinSrgb::new(0.0, 0.0, 1.0),
                    LinSrgb::new(0.0, 1.0, 0.0),
                ]);
                let freshness = grad.get(num::clamp(diff / 10000.0, 0.0, 1.0));
                let color_string = color_to_hex(&freshness);
                self.context.set_fill_style_color(&color_string);
                self.context.begin_path();
                self.context.arc(user_pos.x, user_pos.y, USER_RADIUS, 0.0, std::f64::consts::PI * 2.0, true);
                self.context.fill(FillRule::NonZero);
                match &show_distance {
                    Some(tag_mac) if &user.addr == tag_mac => {
                        self.context.set_fill_style_color("#00000034");
                        self.context.begin_path();
                        self.context.arc(beacon_loc.x, beacon_loc.y, beacon_source.distance_to_tag * map.scale, 0.0, std::f64::consts::PI * 2.0, true);
                        self.context.fill(FillRule::NonZero);
                    },
                    _ => { }
                }
            }
        }
        self.context.restore();
    }
}
