/*
    SPDX-License-Identifier: AGPL-3.0-or-later
    SPDX-FileCopyrightText: 2025 Shomy
*/
use crate::pages::{DevicePage, Page, WelcomePage};
use penumbra::da::DAFile;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyModifiers};
use ratatui::{DefaultTerminal, Frame};
use std::{io::Result, time::Duration};

#[derive(PartialEq, Clone)]
pub enum AppPage {
    Welcome,
    DevicePage,
}

pub struct AppCtx {
    pub loader: Option<DAFile>,
    pub exit: bool,
    pub current_page_id: AppPage,
}

pub struct App {
    current_page: Box<dyn Page + Send>,
    pub context: AppCtx,
}

impl App {
    pub fn new() -> App {
        let context = AppCtx {
            loader: None,
            exit: false,
            current_page_id: AppPage::Welcome,
        };
        App {
            current_page: Box::new(WelcomePage::new()),
            context,
        }
    }

    pub async fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        self.current_page.on_enter(&mut self.context).await;
        let mut previous_page = self.context.current_page_id.clone();
        while !self.context.exit {
            if previous_page != self.context.current_page_id {
                self.switch_to(self.context.current_page_id.clone()).await;
                previous_page = self.context.current_page_id.clone();
            }

            self.current_page.update(&mut self.context).await;
            terminal.draw(|f: &mut Frame<'_>| self.draw(f))?;

            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if key.code == KeyCode::Delete && key.modifiers.contains(KeyModifiers::CONTROL)
                    {
                        self.quit();
                        continue;
                    }

                    self.current_page.handle_input(&mut self.context, key).await;
                }
            }
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame<'_>) {
        self.current_page.render(frame, &mut self.context);
    }

    pub fn quit(&mut self) {
        self.context.exit = true;
    }

    pub async fn switch_to(&mut self, page: AppPage) {
        self.current_page.on_exit(&mut self.context).await;

        let new_page: Box<dyn Page + Send> = match self.context.current_page_id {
            AppPage::Welcome => Box::new(WelcomePage::new()),
            AppPage::DevicePage => Box::new(DevicePage::new()),
        };

        self.current_page = new_page;
        self.current_page.on_enter(&mut self.context).await;
    }
}
