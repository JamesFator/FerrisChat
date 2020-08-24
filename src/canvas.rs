use ferris_chat::components::*;
use ferris_chat::map::{Map, TileType};
use specs::prelude::*;
use stdweb::traits::*;
use stdweb::unstable::TryInto;
use stdweb::web::html_element::CanvasElement;
use stdweb::web::{document, CanvasRenderingContext2d, FillRule};

pub struct DrawSystem {}

impl<'a> System<'a> for DrawSystem {
    type SystemData = (
        Entities<'a>,
        ReadExpect<'a, Canvas>,
        ReadExpect<'a, Map>,
        ReadStorage<'a, Location>,
        ReadStorage<'a, Renderable>,
        ReadStorage<'a, TextRenderable>,
        ReadStorage<'a, ChatRenderable>,
        ReadStorage<'a, GraphicRenderable>,
        ReadStorage<'a, Disappearing>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            canvas,
            map,
            locations,
            renderable,
            text_renders,
            chat_renders,
            graphic_renders,
            disappearings,
        ) = data;

        // Clear the canvas to draw again
        canvas.draw_blank_map(&map);

        let mut draw_data = (&entities, &locations, &renderable)
            .join()
            .collect::<Vec<_>>();
        draw_data.sort_by(|&a, &b| b.2.render_order.cmp(&a.2.render_order));
        for (entity, location, _renderable) in draw_data.iter() {
            let alpha = match disappearings.get(*entity) {
                None => 1f64,
                Some(disappearing) => {
                    (disappearing.ticks_left as f64 / disappearing.total_ticks as f64).min(1.0)
                }
            };
            match text_renders.get(*entity) {
                None => {}
                Some(text_render) => {
                    canvas.draw_text(alpha, &location, &text_render);
                }
            };
            match chat_renders.get(*entity) {
                None => {}
                Some(chat_render) => {
                    canvas.draw_chat_bubble(alpha, &location, &chat_render);
                }
            };
            match graphic_renders.get(*entity) {
                None => {}
                Some(graphic_render) => {
                    canvas.draw_graphic(alpha, &location, &graphic_render);
                }
            };
        }
    }
}

pub struct Canvas {
    pub canvas: CanvasElement,
    pub ctx: CanvasRenderingContext2d,
    pub scaled_width: f64,
    pub scaled_height: f64,
    width: u32,
    height: u32,
}

const WATER_TOP_COLOR: &str = "#67bde0";
const WATER_LEFT_COLOR: &str = "#41add8";
const WATER_RIGHT_COLOR: &str = "#95d1e9";
const SAND_TOP_COLOR: &str = "#fae7c9";
const SAND_LEFT_COLOR: &str = "#f4cc8a";
const SAND_RIGHT_COLOR: &str = "#fdf5e8";
const GRASS_TOP_COLOR: &str = "#56b000";
const GRASS_LEFT_COLOR: &str = "#3e8000";
const GRASS_RIGHT_COLOR: &str = "#6fe600";

impl Canvas {
    pub fn new(attr_id: &str, width: u32, height: u32) -> Canvas {
        let canvas: CanvasElement = document()
            .query_selector(attr_id)
            .unwrap()
            .unwrap()
            .try_into()
            .unwrap();

        let ctx: CanvasRenderingContext2d = canvas.get_context().unwrap();

        let scaled_width = canvas.width() as f64 / width as f64 * 2.0;
        // x0.99 because bottom was being trimmed and I'm a shit programmer.
        let scaled_height = canvas.height() as f64 / height as f64 * 0.99;

        Canvas {
            canvas,
            ctx,
            scaled_width,
            scaled_height,
            width,
            height,
        }
    }

    pub fn convert_from_screen(
        &self,
        x: f64,
        y: f64,
        canvas_buffer_top: f64,
        canvas_buffer_left: f64,
    ) -> (f64, f64) {
        let relative_x = (x - self.width as f64 * 4.0 - canvas_buffer_left) / self.scaled_width;
        let relative_y = (y - self.scaled_height - canvas_buffer_top) / self.scaled_height;
        let x = (relative_y + relative_x).floor();
        let y = (relative_y - relative_x).floor();
        (x, y)
    }

    pub fn convert_to_isometric(&self, x: i32, y: i32) -> (f64, f64) {
        let screen_x = (x - y) as f64 * self.scaled_width / 2.0 + (self.width as f64 * 4.0);
        let screen_y = (x + y) as f64 * self.scaled_height / 2.0 + self.scaled_height;

        (screen_x, screen_y)
    }

    fn draw_tile(&self, orig_x: usize, orig_y: usize, tiles: &Vec<Vec<TileType>>) {
        let tile_type = tiles[orig_x][orig_y];
        if tile_type == TileType::Void || tile_type == TileType::Water {
            return; // Shouldn't render void tiles
        }
        let (height_scale, top_color, left_color, right_color) = match tile_type {
            TileType::Water => (0.25, WATER_TOP_COLOR, WATER_LEFT_COLOR, WATER_RIGHT_COLOR),
            TileType::Sand => (0.0, SAND_TOP_COLOR, SAND_LEFT_COLOR, SAND_RIGHT_COLOR),
            TileType::Grass => (-0.5, GRASS_TOP_COLOR, GRASS_LEFT_COLOR, GRASS_RIGHT_COLOR),
            TileType::Void => (0.0, "", "", ""),
        };
        let paint_left_face = !(orig_y + 1 < (self.height - 1) as usize)
            || (tile_type as i32) > tiles[orig_x][orig_y + 1] as i32;
        let paint_right_face = !(orig_x + 1 < (self.width - 1) as usize)
            || (tile_type as i32) > tiles[orig_x + 1][orig_y] as i32;
        let height_modifier = height_scale * self.scaled_height;

        // Draw the isometric tile
        let (x, y) = self.convert_to_isometric(orig_x as i32, orig_y as i32);

        // Draw the tile top
        // --------------------------------------------
        //    step 1  |  step 2  |  step 3  |  step 4
        // --------------------------------------------
        //    /       |  /       |  /       |  /\
        //            |  \       |  \/      |  \/
        // --------------------------------------------
        self.ctx.begin_path();
        self.ctx
            .move_to(x - self.scaled_width / 2.0, height_modifier + y);
        self.ctx.line_to(
            x - self.scaled_width,
            height_modifier + y + self.scaled_height / 2.0,
        );
        self.ctx.line_to(
            x - self.scaled_width / 2.0,
            height_modifier + y + self.scaled_height,
        );
        self.ctx
            .line_to(x, height_modifier + y + self.scaled_height / 2.0);
        self.ctx
            .line_to(x - self.scaled_width / 2.0, height_modifier + y);

        // self.ctx.stroke();
        self.ctx.set_fill_style_color(top_color);
        self.ctx.fill(FillRule::NonZero);

        if paint_left_face {
            // --------------------------------------------
            //    step 1  |  step 2  |  step 3  |  step 4
            // --------------------------------------------
            //     \      |    \     |    \     |    |\
            //            |    |     |   \|     |    \|
            // --------------------------------------------
            self.ctx.begin_path();
            self.ctx.move_to(
                x - self.scaled_width,
                height_modifier + y + self.scaled_height / 2.0,
            );
            self.ctx.line_to(
                x - self.scaled_width / 2.0,
                height_modifier + y + self.scaled_height,
            );
            self.ctx
                .line_to(x - self.scaled_width / 2.0, y + self.scaled_width);
            self.ctx
                .line_to(x - self.scaled_width, y + self.scaled_height * 1.5);
            self.ctx.line_to(
                x - self.scaled_width,
                height_modifier + y + self.scaled_height / 2.0,
            );

            // self.ctx.stroke();
            self.ctx.set_fill_style_color(left_color);
            self.ctx.fill(FillRule::NonZero);
        }

        if paint_right_face {
            // --------------------------------------------
            //    step 1  |  step 2  |  step 3  |  step 4
            // --------------------------------------------
            //     /      |     /    |     /    |      /|
            //            |    |     |    |/    |     |/
            // --------------------------------------------
            self.ctx.begin_path();
            self.ctx
                .move_to(x, height_modifier + y + self.scaled_height / 2.0);
            self.ctx.line_to(
                x - self.scaled_width / 2.0,
                height_modifier + y + self.scaled_height,
            );
            self.ctx
                .line_to(x - self.scaled_width / 2.0, y + self.scaled_width);
            self.ctx.line_to(x, y + self.scaled_height * 1.5);
            self.ctx
                .line_to(x, height_modifier + y + self.scaled_height / 2.0);

            // self.ctx.stroke();
            self.ctx.set_fill_style_color(right_color);
            self.ctx.fill(FillRule::NonZero);
        }
    }

    pub fn draw_blank_map(&self, map: &Map) {
        // Fill background with water color
        self.ctx.set_fill_style_color(WATER_TOP_COLOR);
        self.ctx.fill_rect(
            0.0,
            0.0,
            self.width as f64 * self.scaled_width,
            self.height as f64 * self.scaled_height,
        );

        // Iterate through the tiles, drawing the color for each
        for x in 0..map.width - 1 {
            for y in 0..map.width - 1 {
                self.draw_tile(x as usize, y as usize, &map.tiles);
            }
        }
    }

    pub fn draw_graphic(
        &self,
        alpha: f64,
        location: &Location,
        graphic_renderable: &GraphicRenderable,
    ) {
        self.ctx.set_global_alpha(alpha);

        let graphic_width = self.scaled_width * 5_f64;
        let graphic_height = self.scaled_height * 10_f64;

        let (iso_x, iso_y) = self.convert_to_isometric(location.x as i32, location.y as i32);
        let x = iso_x + graphic_renderable.offset_x * self.scaled_width - graphic_width / 2.0;
        let y = iso_y + graphic_renderable.offset_y * self.scaled_height - graphic_height / 2.0;

        let img_element: stdweb::web::html_element::ImageElement = stdweb::web::document()
            .get_element_by_id(&graphic_renderable.image_name)
            .unwrap()
            .try_into()
            .unwrap();
        self.ctx
            .draw_image_d(img_element, x, y, graphic_width, graphic_height)
            .expect("draw_image_d failed");

        self.ctx.set_global_alpha(1f64);
    }

    pub fn draw_text(&self, alpha: f64, location: &Location, text_renderable: &TextRenderable) {
        self.ctx.set_global_alpha(alpha);

        let text_height = text_renderable.font_size;
        self.ctx.set_font(&format!("{}px helvetica", text_height));

        let text_width = self
            .ctx
            .measure_text(&text_renderable.text)
            .expect("Canvas measure_text failed")
            .get_width();

        let (iso_x, iso_y) = self.convert_to_isometric(location.x as i32, location.y as i32);
        let x = iso_x - (text_width / 2_f64) + text_renderable.offset_x * self.scaled_width;
        let y = iso_y + text_renderable.offset_y * self.scaled_height;

        self.ctx.set_fill_style_color("black");
        self.ctx
            .fill_text(&text_renderable.text, x, y, Some(2000f64));

        self.ctx.set_global_alpha(1f64);
    }

    pub fn draw_chat_bubble(
        &self,
        alpha: f64,
        location: &Location,
        chat_renderable: &ChatRenderable,
    ) {
        self.ctx.set_font("20px helvetica");
        let (iso_x, iso_y) = self.convert_to_isometric(location.x as i32, location.y as i32);
        let x = iso_x as f64 + chat_renderable.offset_x * self.scaled_width;
        let y = iso_y as f64 + (1f64 + chat_renderable.offset_y) * self.scaled_height;
        let w = self
            .ctx
            .measure_text(&chat_renderable.text)
            .expect("Canvas measure_text failed")
            .get_width()
            + 14_f64;
        let h = 45_f64;
        let r = x + w;
        let b = y + h;
        let radius = 10_f64;

        self.ctx.set_global_alpha(alpha);

        self.ctx.begin_path();
        self.ctx.set_fill_style_color("white");
        self.ctx.fill(FillRule::NonZero);
        self.ctx.set_stroke_style_color("black");
        self.ctx.set_line_width(1_f64);
        self.ctx.move_to(x + radius, y);

        self.ctx.line_to(r - radius, y);
        self.ctx.quadratic_curve_to(r, y, r, y + radius);
        self.ctx.line_to(r, y + h - radius);
        self.ctx.quadratic_curve_to(r, b, r - radius, b);
        self.ctx.line_to(x + radius, b);
        self.ctx.quadratic_curve_to(x, b, x, b - radius);
        self.ctx.line_to(x, y + radius);
        self.ctx.quadratic_curve_to(x, y, x + radius, y);
        self.ctx.fill(FillRule::NonZero);
        self.ctx.stroke();
        self.ctx.set_fill_style_color("black");
        self.ctx
            .fill_text(&chat_renderable.text, x + 7_f64, y + 28_f64, None);

        self.ctx.set_global_alpha(1f64);
    }
}
