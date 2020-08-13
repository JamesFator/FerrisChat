use stdweb::traits::*;
use stdweb::unstable::TryInto;
use stdweb::web::html_element::CanvasElement;
use stdweb::web::{document, CanvasRenderingContext2d};

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

    pub fn draw(&self, x: u32, y: u32, color: &str, name: &str, alpha: f64) {
        assert!(x < self.width);
        assert!(y < self.height);

        self.ctx.set_global_alpha(alpha);

        self.ctx.set_fill_style_color(color);

        let x = x * self.scaled_width;
        let y = y * self.scaled_height;

        if color != "" {
            self.ctx.fill_rect(
                f64::from(x),
                f64::from(y),
                f64::from(self.scaled_width),
                f64::from(self.scaled_height),
            );
        }

        // let name_y = (y + 1) * self.scaled_height;
        self.ctx.set_fill_style_color("black");
        self.ctx.set_font("20px helvetica");
        self.ctx.fill_text(
            name,
            f64::from(x),
            y as f64 + self.scaled_height as f64 * 1.5,
            Some(2000f64),
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
