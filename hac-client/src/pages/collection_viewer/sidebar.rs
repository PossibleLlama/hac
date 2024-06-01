use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use hac_core::collection::types::{Request, RequestKind, RequestMethod};

use crate::pages::collection_viewer::collection_store::{CollectionStore, CollectionStoreAction};
use crate::pages::{Eventful, Page};

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, RwLock};

use ratatui::layout::Rect;
use ratatui::style::{Style, Styled, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use super::collection_viewer::PaneFocus;

#[derive(Debug)]
pub struct Sidebar<'sbar> {
    colors: &'sbar hac_colors::Colors,
    is_focused: bool,
    is_selected: bool,
    lines: Vec<Paragraph<'static>>,
    collection_store: Rc<RefCell<CollectionStore>>,
}

impl<'sbar> Sidebar<'sbar> {
    pub fn new(
        colors: &'sbar hac_colors::Colors,
        is_focused: bool,
        is_selected: bool,
        collection_store: Rc<RefCell<CollectionStore>>,
    ) -> Self {
        let mut sidebar = Self {
            colors,
            is_focused,
            is_selected,
            lines: vec![],
            collection_store,
        };

        sidebar.rebuild_tree_view();

        sidebar
    }

    pub fn maybe_select(&mut self, selected_pane: Option<&PaneFocus>) {
        self.is_selected = selected_pane.is_some_and(|pane| pane.eq(&PaneFocus::Sidebar));
    }

    pub fn maybe_focus(&mut self, focused_pane: &PaneFocus) {
        self.is_focused = focused_pane.eq(&PaneFocus::Sidebar);
    }

    pub fn rebuild_tree_view(&mut self) {
        let mut collection_store = self.collection_store.borrow_mut();
        self.lines = build_lines(
            collection_store.get_requests(),
            0,
            collection_store.get_selected_request(),
            collection_store.get_hovered_request(),
            collection_store.get_dirs_expanded().unwrap().clone(),
            self.colors,
        );
    }
}

impl<'sbar> Page for Sidebar<'sbar> {
    fn draw(&mut self, frame: &mut Frame, size: Rect) -> anyhow::Result<()> {
        let mut requests_size = Rect::new(size.x + 1, size.y, size.width.saturating_sub(2), 1);

        let block_border = match (self.is_focused, self.is_selected) {
            (true, false) => Style::default().fg(self.colors.bright.blue),
            (true, true) => Style::default().fg(self.colors.normal.red),
            (false, _) => Style::default().fg(self.colors.bright.black),
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .title(vec![
                "R".fg(self.colors.normal.red).bold(),
                "equests".fg(self.colors.bright.black),
            ])
            .border_style(block_border);

        frame.render_widget(block, size);

        self.lines.clone().into_iter().for_each(|req| {
            requests_size.y += 1;
            frame.render_widget(req, requests_size);
        });

        Ok(())
    }

    fn resize(&mut self, _new_size: Rect) {}
}

#[derive(Debug)]
pub enum SidebarEvent {
    CreateRequest,
    Quit,
}

impl<'a> Eventful for Sidebar<'a> {
    type Result = SidebarEvent;

    fn handle_key_event(&mut self, key_event: KeyEvent) -> anyhow::Result<Option<Self::Result>> {
        assert!(
            self.is_selected,
            "handled an event to the sidebar while it was not selected"
        );

        if let (KeyCode::Char('c'), KeyModifiers::CONTROL) = (key_event.code, key_event.modifiers) {
            return Ok(Some(SidebarEvent::Quit));
        }

        let mut store = self.collection_store.borrow_mut();
        match key_event.code {
            KeyCode::Enter => {
                if store.get_hovered_request().is_some() && store.get_requests().is_some() {
                    let request = store.find_hovered_request();
                    match request {
                        RequestKind::Nested(_) => {
                            store
                                .dispatch(CollectionStoreAction::ToggleDirectory(request.get_id()));
                            drop(store);
                            self.rebuild_tree_view();
                        }
                        RequestKind::Single(req) => {
                            store.dispatch(CollectionStoreAction::SetSelectedRequest(Some(req)));
                            drop(store);
                            self.rebuild_tree_view();
                        }
                    }
                }
            }
            KeyCode::Char('j') | KeyCode::Down => {
                if store.get_hovered_request().is_some() && store.get_requests().is_some() {
                    store.dispatch(CollectionStoreAction::HoverNext);
                    drop(store);
                    self.rebuild_tree_view();
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if store.get_hovered_request().is_some() && store.get_requests().is_some() {
                    store.dispatch(CollectionStoreAction::HoverPrev);
                    drop(store);
                    self.rebuild_tree_view();
                }
            }

            KeyCode::Char('n') => return Ok(Some(SidebarEvent::CreateRequest)),
            _ => {}
        }

        Ok(None)
    }
}

pub fn build_lines(
    requests: Option<Arc<RwLock<Vec<RequestKind>>>>,
    level: usize,
    selected_request: Option<Arc<RwLock<Request>>>,
    hovered_request: Option<String>,
    dirs_expanded: Rc<RefCell<HashMap<String, bool>>>,
    colors: &hac_colors::Colors,
) -> Vec<Paragraph<'static>> {
    requests
        .unwrap_or(Arc::new(RwLock::new(vec![])))
        .read()
        .unwrap()
        .iter()
        .flat_map(|item| match item {
            RequestKind::Nested(dir) => {
                let is_hovered = hovered_request
                    .as_ref()
                    .is_some_and(|id| id.eq(&item.get_id()));
                let mut dirs = dirs_expanded.borrow_mut();
                let is_expanded = dirs.entry(dir.id.to_string()).or_insert(false);

                let dir_style = match is_hovered {
                    true => Style::default()
                        .fg(colors.normal.white)
                        .bg(colors.primary.hover)
                        .bold(),
                    false => Style::default().fg(colors.normal.white).bold(),
                };

                let gap = " ".repeat(level * 2);
                let chevron = if *is_expanded { "v" } else { ">" };
                let line = vec![Paragraph::new(format!(
                    "{}{} {}/",
                    gap,
                    chevron,
                    dir.name.to_lowercase().replace(' ', "-")
                ))
                .set_style(dir_style)];

                let nested_lines = if *is_expanded {
                    build_lines(
                        Some(dir.requests.clone()),
                        level + 1,
                        selected_request.clone(),
                        hovered_request.clone(),
                        dirs_expanded.clone(),
                        colors,
                    )
                } else {
                    vec![]
                };
                line.into_iter().chain(nested_lines).collect::<Vec<_>>()
            }
            RequestKind::Single(req) => {
                let gap = " ".repeat(level * 2);
                let is_selected = selected_request.as_ref().is_some_and(|selected| {
                    selected.read().unwrap().id.eq(&req.read().unwrap().id)
                });
                let is_hovered = hovered_request
                    .as_ref()
                    .is_some_and(|id| id.eq(&item.get_id()));

                let req_style = match (is_selected, is_hovered) {
                    (true, true) => Style::default()
                        .fg(colors.normal.yellow)
                        .bg(colors.normal.blue),
                    (true, _) => Style::default()
                        .fg(colors.normal.white)
                        .bg(colors.normal.blue),
                    (_, true) => Style::default()
                        .fg(colors.normal.white)
                        .bg(colors.primary.hover),
                    (false, false) => Style::default().fg(colors.normal.white),
                };

                let line: Line<'_> = vec![
                    Span::from(gap.clone()),
                    colored_method(req.read().unwrap().method.clone(), colors),
                    Span::from(format!(" {}", req.read().unwrap().name.clone())),
                ]
                .into();

                vec![Paragraph::new(line).set_style(req_style)]
            }
        })
        .collect()
}

fn colored_method(method: RequestMethod, colors: &hac_colors::Colors) -> Span<'static> {
    match method {
        RequestMethod::Get => "GET   ".fg(colors.normal.green).bold(),
        RequestMethod::Post => "POST  ".fg(colors.normal.magenta).bold(),
        RequestMethod::Put => "PUT   ".fg(colors.normal.yellow).bold(),
        RequestMethod::Patch => "PATCH ".fg(colors.normal.orange).bold(),
        RequestMethod::Delete => "DELETE".fg(colors.normal.red).bold(),
    }
}