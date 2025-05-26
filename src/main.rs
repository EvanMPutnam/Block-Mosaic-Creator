use std::cmp::Ordering;
use std::env;
use std::fs::File;
use std::io::Read;
use image::imageops::FilterType;
use image::{GenericImageView, ImageReader};
use nannou::prelude::*;
use nannou::rand::prelude::SliceRandom;
use nannou::rand::thread_rng;
use serde::Deserialize;
use serde_json;

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
    count: u64
}

impl ColorConfig {
    fn decrement(&mut self) {
        self.count -= 1;
    }
}

#[derive(Debug, Deserialize)]
struct Color {
    r: u8,
    g: u8,
    b: u8,
    x: u64,
    y: u64
}

fn main() {
    nannou::app(model).simple_window(view).update(update).run();
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    draw.background().color(BLACK);
    draw_square(&app, &draw, &model);
    draw.to_frame(app, &frame).expect("Unable to draw to frame.");
}

fn update(_app: &App, _model: &mut Model, _update: Update) {}

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
    let img_resized = img.resize_exact(X_SIZE as u32, Y_SIZE as u32,
                                          FilterType::Nearest);

    let mut file = File::open(color_data).expect("Could not open color data file.");
    let mut buff = String::new();
    file.read_to_string(&mut buff).expect("Unable to read color data file.");

    let mut color_configs: ColorConfigs = serde_json::from_str(buff.as_str())
        .expect("JSON not parseable.");

    let mut colors: Vec<Color> = Vec::new();
    for y in 0..Y_SIZE {
        for x in 0..X_SIZE {
            let pixel = img_resized.get_pixel(x as u32, (Y_SIZE - y - 1) as u32);
            colors.push(Color{
                r: pixel.0[0] as u8,
                g: pixel.0[1] as u8,
                b: pixel.0[2] as u8,
                x,
                y
            })
        }
    }
    colors.shuffle(&mut thread_rng());

    let mut colors: Vec<Color> = colors.iter().map(|original_color| -> Color {
        let nearest_color = calculate_closest_color(&color_configs, original_color);
        let selected_config = color_configs.colors
            .get_mut(nearest_color)
            .expect("Color configs should have value within index range");
        selected_config.decrement();
        Color {
            r: selected_config.r,
            g: selected_config.g,
            b: selected_config.b,
            x: original_color.x,
            y: original_color.y
        }
    }).collect();
    colors.sort_by(|a, b| match a.y.cmp(&b.y) {
        Ordering::Equal => a.x.cmp(&b.x),
        other => other,
    });

    Model {
        pixels: colors,
    }
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
    let window_size = app.main_window().inner_size_pixels();
    let sf = app.main_window().scale_factor().abs() as u64;

    let x_width = window_size.0 as u64 / X_SIZE / sf;
    let y_height = window_size.1 as u64 / Y_SIZE / sf;

    // Ensures we align completely with the grid.
    let x_offset: i32 = ((window_size.0 / 2 / (sf as u32)) as i32) - (x_width as i32 / 2);
    let y_offset: i32 = ((window_size.1 / 2 / (sf as u32)) as i32) - (y_height as i32 / 2);

    let mut count = 0;
    for y in 0..Y_SIZE {
        for x in 0..X_SIZE {
            let x_f: f32 = (x * x_width) as f32 - x_offset as f32;
            let y_f: f32 = (y * y_height) as f32 - y_offset as f32;
            let color = model.pixels.get(count).expect("Model should have appropriate pixel count.");
            draw.rect().xy(Point2::new(x_f, y_f))
                .color(srgb8(color.r, color.g, color.b))
                .width(x_width as f32 - 1.0)
                .height(y_height as f32 - 1.0);
            count += 1;
        }
    }
}