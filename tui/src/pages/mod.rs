/*
    SPDX-License-Identifier: AGPL-3.0-or-later
    SPDX-FileCopyrightText: 2025 Shomy
*/
pub mod device;
pub mod welcome;
pub use device::DevicePage;
pub use welcome::WelcomePage;

use crate::app::AppCtx;
use ratatui::Frame;
use ratatui::crossterm::event::KeyEvent;

#[async_trait::async_trait]
pub trait Page {
    fn render(&mut self, frame: &mut Frame<'_>, ctx: &mut AppCtx);
    async fn handle_input(&mut self, ctx: &mut AppCtx, key: KeyEvent);
    async fn on_enter(&mut self, _ctx: &mut AppCtx) {}
    async fn on_exit(&mut self, _ctx: &mut AppCtx) {}
    async fn update(&mut self, _ctx: &mut AppCtx) {}
}
