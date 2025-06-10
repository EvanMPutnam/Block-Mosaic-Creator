use image::imageops::FilterType;
use image::{GenericImageView, ImageReader};
use nannou::prelude::real::Real;
use nannou::prelude::*;
use nannou::rand::prelude::SliceRandom;
use nannou::rand::thread_rng;
use serde::Deserialize;
use serde_json;
use std::cmp::Ordering;
use std::env;
use std::fs::File;
use std::io::Read;

const X_SIZE: u64 = 48;
const Y_SIZE: u64 = 48;

struct Model {
    pixels: Vec<Color>,
}

#[derive(Debug, Deserialize)]
struct ColorConfigs {
    colors: Vec<ColorConfig>,
}

#[derive(Debug, Deserialize)]
struct ColorConfig {
    name: String,
    r: u8,
    g: u8,
    b: u8,
    count: u64,
}

impl ColorConfig {
    fn decrement(&mut self) {
        self.count -= 1;
    }
}

#[derive(Debug, Deserialize, Clone)]
struct Color {
    r: u8,
    g: u8,
    b: u8,
    x: u64,
    y: u64,
}

fn main() {
    nannou::app(model).simple_window(view).update(update).run();
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    draw.background().color(BLACK);
    draw_square(&app, &draw, &model);
    draw.to_frame(app, &frame)
        .expect("Unable to draw to frame.");
}

fn update(_app: &App, _model: &mut Model, _update: Update) {
    let pressed_mouse = _app.mouse.buttons.left().is_down();
    if !pressed_mouse {
        return;
    }

    let pixels = _app.main_window().inner_size_points();

    let x_width = pixels.0.abs();
    let y_height = pixels.1.abs();

    // Ensures we align completely with the grid.
    let x_offset = x_width / 2.0;
    let y_offset = y_height / 2.0;
    let x = _app.mouse.x + x_offset;
    let y = _app.mouse.y + y_offset;

    // Scale x from screen width to X_SIZE and truncate result to int.
    let x_scaled = ((x as f64 / x_width as f64) * X_SIZE as f64) as u64;
    let y_scaled = ((y as f64 / y_height as f64) * Y_SIZE as f64) as u64;

    let index = y_scaled * X_SIZE + x_scaled;
    let color = _model.pixels[index as usize].clone();

    let rgb_str = format!("Selected Color: rgb({r}, {g}, {b}), Position: xy({x}, {y})",
                          r = color.r, g = color.g, b = color.b, x = color.x, y = color.y);
    _app.main_window().set_title(rgb_str.as_str());
}

fn model(_app: &App) -> Model {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        panic!("Need to provide file paths for picture and color config")
    }
    let picture_path = args.get(1).expect("Please specify path to picture.");
    let color_data = args.get(2).expect("Please specify path to color data.");
    if !color_data.ends_with(".json") {
        panic!("Need to provide filepath for color.json file")
    }

    let img = ImageReader::open(picture_path)
        .expect("Image failed to parse.")
        .decode()
        .expect("Failed to decode image.");
    let img_resized = img.resize_exact(X_SIZE as u32, Y_SIZE as u32, FilterType::Nearest);

    let mut file = File::open(color_data).expect("Could not open color data file.");
    let mut buff = String::new();
    file.read_to_string(&mut buff)
        .expect("Unable to read color data file.");

    let mut color_configs: ColorConfigs =
        serde_json::from_str(buff.as_str()).expect("JSON not parseable.");

    let mut colors: Vec<Color> = Vec::new();
    for y in 0..Y_SIZE {
        for x in 0..X_SIZE {
            let pixel = img_resized.get_pixel(x as u32, (Y_SIZE - y - 1) as u32);
            colors.push(Color {
                r: pixel.0[0] as u8,
                g: pixel.0[1] as u8,
                b: pixel.0[2] as u8,
                x,
                y,
            })
        }
    }
    colors.shuffle(&mut thread_rng());

    let mut colors: Vec<Color> = colors
        .iter()
        .map(|original_color| -> Color {
            let nearest_color = calculate_closest_color(&color_configs, original_color);
            let selected_config = color_configs
                .colors
                .get_mut(nearest_color)
                .expect("Color configs should have value within index range");
            selected_config.decrement();
            Color {
                r: selected_config.r,
                g: selected_config.g,
                b: selected_config.b,
                x: original_color.x,
                y: original_color.y,
            }
        })
        .collect();
    colors.sort_by(|a, b| match a.y.cmp(&b.y) {
        Ordering::Equal => a.x.cmp(&b.x),
        other => other,
    });

    Model { pixels: colors }
}

fn calculate_closest_color(color_configs: &ColorConfigs, original_color: &Color) -> usize {
    let mut closest_dist: f32 = f32::MAX;
    let mut closest_index = usize::MAX;
    let mut count = 0;
    let mut has_available_color = false;
    for color_config in color_configs.colors.iter() {
        if color_config.count == 0 {
            count += 1;
            continue;
        }
        let mut r_dist = (color_config.r as f32 - original_color.r as f32) * 0.3;
        let mut g_dist = (color_config.g as f32 - original_color.g as f32) * 0.59;
        let mut b_dist = (color_config.b as f32 - original_color.b as f32) * 0.11;
        r_dist = r_dist * r_dist;
        g_dist = g_dist * g_dist;
        b_dist = b_dist * b_dist;

        let dist = r_dist + g_dist + b_dist;
        if dist < closest_dist {
            closest_dist = dist;
            closest_index = count;
        }
        has_available_color = true;
        count += 1;
    }
    if !has_available_color || closest_dist == f32::MAX {
        panic!("Invalid configuration of colors.  Not enough colors present.")
    }
    closest_index
}

fn draw_square(app: &App, draw: &Draw, model: &Model) {
    let window_size = app.main_window().inner_size_points();

    let x_width = window_size.0.abs() / X_SIZE as f32;
    let y_height = window_size.1.abs() / Y_SIZE as f32;

    // Ensures we align completely with the grid.
    let x_offset: f32 = (window_size.0.abs() / 2.0) - (x_width / 2.0);
    let y_offset: f32 = (window_size.1.abs() / 2.0) - (y_height / 2.0);

    let mut count = 0;
    for y in 0..Y_SIZE {
        for x in 0..X_SIZE {
            let x_f: f32 = (x as f32 * x_width) - x_offset;
            let y_f: f32 = (y as f32 * y_height) - y_offset;
            let color = model
                .pixels
                .get(count)
                .expect("Model should have appropriate pixel count.");
            draw.rect()
                .xy(Point2::new(x_f, y_f))
                .color(srgb8(color.r, color.g, color.b))
                .width(x_width as f32 - 1.0)
                .height(y_height as f32 - 1.0);
            count += 1;
        }
    }
}
