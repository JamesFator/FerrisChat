use super::{Disappearing, GraphicRenderable, Location, Renderable, TextRenderable};
use specs::prelude::*;
use stdweb::traits::*;
use stdweb::unstable::TryInto;
use stdweb::web::html_element::CanvasElement;
use stdweb::web::{document, CanvasRenderingContext2d};

pub struct DrawSystem {}

impl<'a> System<'a> for DrawSystem {
    type SystemData = (
        Entities<'a>,
        ReadExpect<'a, Canvas>,
        ReadStorage<'a, Location>,
        ReadStorage<'a, Renderable>,
        ReadStorage<'a, Disappearing>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, canvas, locations, renderables, disappearings) = data;

        // Clear the canvas to draw again
        canvas.clear_all();

        let mut draw_data = (&entities, &locations, &renderables)
            .join()
            .collect::<Vec<_>>();
        draw_data.sort_by(|&a, &b| b.2.render_order.cmp(&a.2.render_order));
        for (entity, location, renderable) in draw_data.iter() {
            let alpha = match disappearings.get(*entity) {
                None => 1f64,
                Some(disappearing) => {
                    disappearing.ticks_left as f64 / disappearing.total_ticks as f64
                }
            };
            match &renderable.text_renderable {
                None => {}
                Some(text_render) => {
                    canvas.draw_text(alpha, &location, &text_render);
                }
            };
            match &renderable.graphic_renderable {
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
    scaled_width: u32,
    scaled_height: u32,
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

        let scaled_width = canvas.width() / width;
        let scaled_height = canvas.height() / height;

        Canvas {
            canvas,
            ctx,
            scaled_width,
            scaled_height,
            width,
            height,
        }
    }

    pub fn draw_text(&self, alpha: f64, location: &Location, text_renderable: &TextRenderable) {
        self.ctx.set_global_alpha(alpha);

        let x = (location.x as f64 * self.scaled_width as f64)
            + (self.scaled_width as f64 * text_renderable.offset_x);
        let y = (location.y as f64 * self.scaled_height as f64)
            + (self.scaled_height as f64 * (1f64 + text_renderable.offset_y));

        self.ctx.set_fill_style_color("black");
        self.ctx.set_font("20px helvetica");
        self.ctx
            .fill_text(&text_renderable.text, x, y, Some(2000f64));

        self.ctx.set_global_alpha(1f64);
    }

    pub fn draw_graphic(
        &self,
        alpha: f64,
        location: &Location,
        graphic_renderable: &GraphicRenderable,
    ) {
        self.ctx.set_global_alpha(alpha);

        let x = (location.x as f64 * self.scaled_width as f64)
            + (self.scaled_width as f64 * graphic_renderable.offset_x);
        let y = (location.y as f64 * self.scaled_height as f64)
            + (self.scaled_height as f64 * graphic_renderable.offset_y);

        self.ctx.set_fill_style_color(&graphic_renderable.color);
        self.ctx.fill_rect(
            x,
            y,
            f64::from(self.scaled_width),
            f64::from(self.scaled_height),
        );

        self.ctx.set_global_alpha(1f64);
    }

    pub fn clear_all(&self) {
        self.ctx.set_fill_style_color("#fae7c9");
        self.ctx.fill_rect(
            0.0,
            0.0,
            f64::from(self.width * self.scaled_width),
            f64::from(self.height * self.scaled_height),
        );
    }
}
