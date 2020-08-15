use super::{
    ChatRenderable, Disappearing, GraphicRenderable, Location, Renderable, TextRenderable,
};
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
            locations,
            renderable,
            text_renders,
            chat_renders,
            graphic_renders,
            disappearings,
        ) = data;

        // Clear the canvas to draw again
        canvas.clear_all();

        let mut draw_data = (&entities, &locations, &renderable)
            .join()
            .collect::<Vec<_>>();
        draw_data.sort_by(|&a, &b| b.2.render_order.cmp(&a.2.render_order));
        for (entity, location, _renderable) in draw_data.iter() {
            let alpha = match disappearings.get(*entity) {
                None => 1f64,
                Some(disappearing) => {
                    disappearing.ticks_left as f64 / disappearing.total_ticks as f64
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

    pub fn clear_all(&self) {
        self.ctx.set_fill_style_color("#fae7c9");
        self.ctx.fill_rect(
            0.0,
            0.0,
            f64::from(self.width * self.scaled_width),
            f64::from(self.height * self.scaled_height),
        );
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
        let img_element: stdweb::web::html_element::ImageElement = stdweb::web::document()
            .get_element_by_id("rustacean")
            .unwrap()
            .try_into()
            .unwrap();
        self.ctx
            .draw_image_d(
                img_element,
                x,
                y,
                f64::from(self.scaled_width),
                f64::from(self.scaled_height),
            )
            .expect("draw_image_d failed");

        self.ctx.set_global_alpha(1f64);
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

    pub fn draw_chat_bubble(
        &self,
        alpha: f64,
        location: &Location,
        chat_renderable: &ChatRenderable,
    ) {
        let x = (location.x as f64 * self.scaled_width as f64)
            + (self.scaled_width as f64 * chat_renderable.offset_x);
        let y = (location.y as f64 * self.scaled_height as f64)
            + (self.scaled_height as f64 * (1f64 + chat_renderable.offset_y));
        let w = self
            .ctx
            .measure_text(&chat_renderable.text)
            .expect("Canvas measure_text failed")
            .get_width()
            + 14_f64;
        let h = self.scaled_width as f64 * 1_f64;
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
