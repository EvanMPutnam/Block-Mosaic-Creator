use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::Read;
use image::imageops::FilterType;
use image::{GenericImageView, ImageReader};
use nannou::prelude::*;
use serde::Deserialize;
use serde_json;

const X_SIZE: u64 = 48;
const Y_SIZE: u64 = 48;

struct Model {
    pixels: Colors
}

#[derive(Debug, Deserialize)]
struct Colors {
    colors: Vec<Color>,
}

#[derive(Debug, Deserialize)]
struct Color {
    name: String,
    r: u8,
    g: u8,
    b: u8,
}

fn main() {
    nannou::app(model).simple_window(view).update(update).run();
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    draw.background().color(BLACK);
    draw_square(&app, &draw, &model);
    draw.to_frame(app, &frame).unwrap();
}

fn update(_app: &App, _model: &mut Model, _update: Update) {}

fn model(_app: &App) -> Model {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        panic!("Need to provide file paths for picture and color config")
    }
    let picture_path = args.get(1).unwrap();
    let color_data = args.get(2).unwrap();
    if !color_data.ends_with(".json") {
        panic!("Need to provide filepath for color.json file")
    }

    let img = ImageReader::open(picture_path).unwrap().decode();
    let img_resized = img.unwrap().resize_exact(X_SIZE as u32, Y_SIZE as u32,
                                          FilterType::Nearest);

    let mut file = File::open(color_data).unwrap();
    let mut buff = String::new();
    file.read_to_string(&mut buff).unwrap();

    let colors: Colors = serde_json::from_str(buff.as_str()).unwrap();
    let mut color_arr: Vec<Color> = vec![];

    let mut used_colors: HashMap<String, u64> = HashMap::new();

    for y in 0..Y_SIZE {
        for x in 0..X_SIZE {
            let pixel = img_resized.get_pixel(x as u32, (Y_SIZE - y - 1) as u32);
            let closest_color = calculate_closest_color(&colors,
                                                        pixel.0[0],
                                                        pixel.0[1] as u8,
                                                        pixel.0[2] as u8);
            if closest_color == usize::MAX {
                panic!("Need to provide at least 1 color to color json file.")
            }

            let color = colors.colors.get(closest_color).unwrap();
            color_arr.push(Color {
                name: color.name.clone(),
                r: color.r,
                g: color.g,
                b: color.b,
            });

            if used_colors.get(color.name.as_str()).is_none() {
                used_colors.insert(color.name.to_string(), 0);
            }
            used_colors.insert(color.name.to_string(),
                               used_colors.get(color.name.as_str()).unwrap() + 1);

        }
    }

    let mut total = 0;
    for (key, value) in used_colors.into_iter() {
        println!("{} - {} pieces", key, value);
        total += value
    }
    println!("Total Pieces: {}", total);

    Model {
        pixels: Colors { colors: color_arr },
    }
}

fn calculate_closest_color(colors: &Colors, r: u8, g: u8, b: u8) -> usize {
    let mut closest_dist: f32 = f32::MAX;
    let mut closest_index = usize::MAX;
    let mut count = 0;
    for color in colors.colors.iter() {
        let mut r_dist = (color.r as f32 - r as f32) * 0.3;
        r_dist = r_dist * r_dist;
        let mut g_dist = (color.g as f32 - g as f32) * 0.59;
        g_dist = g_dist * g_dist;
        let mut b_dist = (color.b as f32 - b as f32) * 0.11;
        b_dist = b_dist * b_dist;

        let dist = r_dist + g_dist + b_dist;
        if dist < closest_dist {
            closest_dist = dist;
            closest_index = count;
        }
        count += 1;
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
            let color = model.pixels.colors.get(count).unwrap();
            draw.rect().xy(Point2::new(x_f, y_f))
                .color(srgb8(color.r, color.g, color.b))
                .width(x_width as f32 - 1.0)
                .height(y_height as f32 - 1.0);
            count += 1;
        }
    }
}