use std::{borrow::Borrow, sync::Arc};

use derivative::Derivative;
use gilrs::Gilrs;
use strum::IntoEnumIterator;
use winit::{
    event::{Event, KeyEvent, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
    window::Window,
};

use crate::{input, render, GameError};

#[derive(Debug)]
pub struct Game<'a> {
    score: i32,
    input: input::Input,
    pub render: render::Render<'a>,
}

impl<'a> Game<'a> {
    pub async fn new(window: Arc<Window>) -> Result<Self, GameError> {
        Ok(Self {
            score: 0,
            input: input::Input::new(),
            render: render::Render::new(window).await?,
        })
    }

    pub fn update(&mut self) {
        if self.input.get_bool(input::KeyboardButton::Space) {
            self.score += 1;
        }
    }

    pub fn draw(&mut self, window: &Arc<Window>) -> Result<(), GameError> {
        window.set_title(&format!("Score: {}", self.score));
        self.render.render(window, &self.input)
    }

    pub fn handle_event(&mut self, event: &Event<()>) -> bool {
        let mut continue_render = true;
        self.input.event(event);

        if self.input.get_bool(input::KeyboardButton::Escape) {
            continue_render = false;
        }

        match event {
            Event::WindowEvent {
                event: ref window_event,
                ..
            } => match window_event {
                WindowEvent::CloseRequested => continue_render = false,
                WindowEvent::Resized(size) => {
                    self.render.resize(*size);
                }
                _ => (),
            },
            _ => (),
        }

        continue_render
    }
}
