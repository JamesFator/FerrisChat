use super::*;
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

impl Canvas {
    pub fn new(attr_id: &str, width: u32, height: u32) -> Canvas {
        let canvas: CanvasElement = document()
            .query_selector(attr_id)
            .unwrap()
            .unwrap()
            .try_into()
            .unwrap();

        let ctx: CanvasRenderingContext2d = canvas.get_context().unwrap();

        let scaled_width = canvas.width() as f64 / width as f64;
        let scaled_height = canvas.height() as f64 / height as f64;

        Canvas {
            canvas,
            ctx,
            scaled_width,
            scaled_height,
            width,
            height,
        }
    }

    pub fn draw_blank_map(&self, map: &Map) {
        let water_color = "#67bde0";
        let sand_color = "#fae7c9";
        let grass_color = "#56b000";
        // Fill background with water color
        self.ctx.set_fill_style_color(water_color);
        self.ctx.fill_rect(
            0.0,
            0.0,
            self.width as f64 * self.scaled_width,
            self.height as f64 * self.scaled_height,
        );

        // Iterate through the tiles, drawing the color for each
        for p in map.iter.iter() {
            match map.tiles[p.x][p.y] {
                TileType::Water => {
                    self.ctx.set_fill_style_color(water_color);
                }
                TileType::Sand => {
                    self.ctx.set_fill_style_color(sand_color);
                }
                TileType::Grass => {
                    self.ctx.set_fill_style_color(grass_color);
                }
            };
            self.ctx.fill_rect(
                p.x as f64 * self.scaled_width,
                p.y as f64 * self.scaled_height,
                self.scaled_width,
                self.scaled_height,
            );
        }
    }

    pub fn draw_graphic(
        &self,
        alpha: f64,
        location: &Location,
        graphic_renderable: &GraphicRenderable,
    ) {
        self.ctx.set_global_alpha(alpha);

        let x = (location.x as f64 * self.scaled_width)
            + (self.scaled_width * graphic_renderable.offset_x);
        let y = (location.y as f64 * self.scaled_height)
            + (self.scaled_height * graphic_renderable.offset_y);

        let img_element: stdweb::web::html_element::ImageElement = stdweb::web::document()
            .get_element_by_id(&graphic_renderable.image_name)
            .unwrap()
            .try_into()
            .unwrap();
        self.ctx
            .draw_image_d(
                img_element,
                x,
                y,
                self.scaled_width * 10_f64,
                self.scaled_height * 10_f64,
            )
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

        let x = (location.x as f64 * self.scaled_width) - (text_width / 2_f64)
            + (self.scaled_width * text_renderable.offset_x);
        let y = (location.y as f64 * self.scaled_height) - (text_height / 2_f64)
            + (self.scaled_height * (1f64 + text_renderable.offset_y));

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
        let x =
            location.x as f64 * self.scaled_width + self.scaled_width * chat_renderable.offset_x;
        let y = location.y as f64 * self.scaled_height
            + self.scaled_height * (1f64 + chat_renderable.offset_y);
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
