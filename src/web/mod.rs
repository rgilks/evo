use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

#[derive(Serialize, Deserialize)]
struct EntityData {
    x: f32,
    y: f32,
    radius: f32,
    r: f32,
    g: f32,
    b: f32,
}

#[wasm_bindgen]
pub struct WebRenderer {
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

        Ok(WebRenderer { ctx, width, height })
    }

    pub fn render(&self, entities: &JsValue) -> Result<(), JsValue> {
        // Parse entities from JS
        let entities: Vec<EntityData> = serde_wasm_bindgen::from_value(entities.clone())?;

        // Clear canvas
        self.ctx
            .clear_rect(0.0, 0.0, self.width as f64, self.height as f64);

        // Calculate center offset to center the simulation world
        let center_x = self.width as f64 / 2.0;
        let center_y = self.height as f64 / 2.0;

        // Render each entity
        for entity in entities {
            self.ctx.begin_path();
            self.ctx.arc(
                center_x + entity.x as f64,
                center_y + entity.y as f64,
                (entity.radius * 0.1) as f64, // Make entities 10x smaller
                0.0,
                2.0 * std::f64::consts::PI,
            )?;

            let fill_style = format!(
                "rgba({}, {}, {}, 0.8)",
                (entity.r * 255.0) as u8,
                (entity.g * 255.0) as u8,
                (entity.b * 255.0) as u8
            );
            self.ctx.set_fill_style(&JsValue::from_str(&fill_style));
            self.ctx.fill();
        }

        Ok(())
    }
}
