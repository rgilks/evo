pub mod controls;
pub mod renderer;
pub mod wasm_bridge;

use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

#[wasm_bindgen]
pub struct WebRenderer {
    canvas: HtmlCanvasElement,
    ctx: CanvasRenderingContext2d,
    width: u32,
    height: u32,
}

#[wasm_bindgen]
impl WebRenderer {
    #[wasm_bindgen(constructor)]
    pub fn new(canvas_id: &str) -> Result<WebRenderer, JsValue> {
        let window = web_sys::window().ok_or("No window")?;
        let document = window.document().ok_or("No document")?;
        let canvas = document
            .get_element_by_id(canvas_id)
            .ok_or("Canvas not found")?
            .dyn_into::<HtmlCanvasElement>()?;

        let ctx = canvas
            .get_context("2d")?
            .ok_or("Failed to get 2d context")?
            .dyn_into::<CanvasRenderingContext2d>()?;

        let width = canvas.width();
        let height = canvas.height();

        Ok(WebRenderer {
            canvas,
            ctx,
            width,
            height,
        })
    }

    pub fn render(&self, entities: &JsValue) -> Result<(), JsValue> {
        // Parse entities from JS
        let entities: Vec<(f32, f32, f32, f32, f32, f32)> =
            serde_wasm_bindgen::from_value(entities.clone())?;

        // Clear canvas
        self.ctx
            .clear_rect(0.0, 0.0, self.width as f64, self.height as f64);

        // Calculate center offset to center the simulation world
        let center_x = self.width as f64 / 2.0;
        let center_y = self.height as f64 / 2.0;

        // Render each entity
        for (x, y, radius, r, g, b) in entities {
            self.ctx.begin_path();
            self.ctx.arc(
                center_x + x as f64,
                center_y + y as f64,
                (radius * 0.1) as f64, // Make entities 10x smaller
                0.0,
                2.0 * std::f64::consts::PI,
            )?;

            self.ctx.set_fill_style(&JsValue::from_str(&format!(
                "rgba({}, {}, {}, 0.8)",
                (r * 255.0) as u8,
                (g * 255.0) as u8,
                (b * 255.0) as u8
            )));
            self.ctx.fill();
        }

        Ok(())
    }
}
