pub mod game;
pub mod input;
pub mod render;

pub type Result<T> = std::result::Result<T, GameError>;

use game::Game;
use serde_json::json;
use std::sync::Arc;
use thiserror::Error;

use game_loop::game_loop;
use winit::{
    error::{EventLoopError, OsError},
    event::{self, *},
    event_loop::{ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::WindowBuilder,
};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() -> Result<()> {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
        } else {
            // env_logger::init();
        }
    }

    let event_loop = EventLoop::new()?;
    let window_size = winit::dpi::PhysicalSize::new(450, 400);
    let window = Arc::new(
        WindowBuilder::new()
            .with_inner_size(window_size)
            .build(&event_loop)?,
    );

    #[cfg(target_arch = "wasm32")]
    {
        // Winit prevents sizing with CSS, so we have to set
        // the size manually when on web.
        window.set_min_inner_size(Some(window_size));

        use winit::platform::web::WindowExtWebSys;
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                let dst = doc.get_element_by_id("game")?;
                let canvas = window.canvas()?;
                dst.append_child(&canvas).ok()?;
                Some(())
            })
            .expect("Couldn't append canvas to document body.");
    }

    // ControlFlow::Poll continuously runs the event loop, even if the OS hasn't
    // dispatched any events. This is ideal for games and similar applications.
    event_loop.set_control_flow(ControlFlow::Poll);

    let game = Game::new(window.clone()).await?;

    game_loop(
        event_loop,
        window,
        game,
        240,
        0.1,
        |g| {
            g.game.update(g.last_frame_time());
        },
        |g| match g.game.draw(&g.window) {
            Ok(_) => (),
            Err(GameError::SurfaceError(surface_error)) => match surface_error {
                // Reconfigure the surface if lost
                wgpu::SurfaceError::Lost => {
                    log::warn!("Surface lost, recreating");
                    g.game.render.resize(g.window.inner_size());
                }
                // The system is out of memory, we should probably quit
                wgpu::SurfaceError::OutOfMemory => {
                    log::error!("Surface out of memory, exiting");
                    g.exit();
                }
                // All other errors (Outdated, Timeout) should be resolved by the next frame
                _ => {
                    log::error!("Surface error: {:?}", surface_error);
                }
            },
            Err(err) => {
                log::error!("{}", err);
            }
        },
        |g, event| {
            if !g.game.handle_event(event) {
                g.exit();
            }
        },
    )
    .map_err(|err| err.into())
}

#[derive(Error, Debug)]
pub enum GameError {
    #[error("Event loop error: {0}")]
    EventLoopError(#[from] EventLoopError),
    #[error("OS error: {0}")]
    OsError(#[from] OsError),
    #[cfg(target_arch = "wasm32")]
    #[error("JS value error: {0}")]
    JsValue(String),
    #[error("No adapter found")]
    NoAdapter,
    #[error("Request adapter error: {0}")]
    RequestDeviceError(#[from] wgpu::RequestDeviceError),
    #[error("No surface format found")]
    NoSurfaceFormat,
    #[error("Surface error: {0}")]
    SurfaceError(#[from] wgpu::SurfaceError),
    #[error("Image error: {0}")]
    ImageError(#[from] image::ImageError),
}

#[cfg(target_arch = "wasm32")]
impl From<JsValue> for GameError {
    fn from(value: JsValue) -> Self {
        let pot = value.as_string().unwrap_or("Unknown".to_string());
        GameError::JsValue(
            serde_wasm_bindgen::from_value(value)
                .unwrap_or(pot)
                .to_string(),
        )
    }
}

#[cfg(target_arch = "wasm32")]
impl Into<JsValue> for GameError {
    fn into(self) -> JsValue {
        JsValue::from_str(&self.to_string())
    }
}
