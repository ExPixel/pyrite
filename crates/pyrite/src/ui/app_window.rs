use std::sync::Arc;

use ahash::HashSet;
use egui::{ViewportBuilder, ViewportId};
use parking_lot::Mutex;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum AppWindowCategory {
    Egui,
    Gba,
}

pub trait AppWindow {
    type State: Send + 'static;

    fn ui(state: &mut Self::State, ctx: &egui::Context);
    fn save(_state: &mut Self::State, _storage: &mut dyn eframe::Storage) {}
    fn title() -> String;
    fn viewport_id() -> ViewportId;
    fn viewport_builder() -> ViewportBuilder {
        ViewportBuilder {
            title: Some(Self::title()),
            ..Default::default()
        }
    }
    fn category() -> AppWindowCategory;
}

pub struct AppWindowWrapper {
    #[allow(clippy::type_complexity)]
    show_viewport_deferred_fn: Box<dyn Fn(&egui::Context)>,
    #[allow(clippy::type_complexity)]
    save_fn: Box<dyn Fn(&mut dyn eframe::Storage)>,
    viewport_id_fn: Box<dyn Fn() -> ViewportId>,
    category_fn: Box<dyn Fn() -> AppWindowCategory>,
    set_visibility_fn: Box<dyn Fn(bool)>,
    visible_fn: Box<dyn Fn() -> bool>,
    #[allow(clippy::type_complexity)]
    visible_fast_fn: Box<dyn Fn(&HashSet<ViewportId>) -> bool>,
    title_fn: Box<dyn Fn() -> String>,
}

impl AppWindowWrapper {
    pub fn new<T: AppWindow>(windows: Arc<Mutex<HashSet<ViewportId>>>, state: T::State) -> Self {
        let ui_state = Arc::new(Mutex::new(state));
        let save_state = ui_state.clone();
        Self {
            show_viewport_deferred_fn: {
                let windows = windows.clone();
                Box::new(move |ctx| {
                    let ui_state = ui_state.clone();
                    let windows = windows.clone();
                    ctx.show_viewport_deferred(
                        T::viewport_id(),
                        T::viewport_builder(),
                        move |ctx, _| {
                            let mut state = ui_state.lock();
                            T::ui(&mut state, ctx);
                            ctx.input(|input| {
                                if input.viewport().close_requested() {
                                    windows.lock().remove(&T::viewport_id());
                                }
                            });
                        },
                    )
                })
            },
            save_fn: Box::new(move |storage| {
                let mut state = save_state.lock();
                T::save(&mut state, storage);
            }),
            viewport_id_fn: Box::new(|| T::viewport_id()),
            category_fn: Box::new(|| T::category()),
            set_visibility_fn: {
                let windows = windows.clone();
                Box::new(move |visible| {
                    if visible {
                        windows.lock().insert(T::viewport_id());
                    } else {
                        windows.lock().remove(&T::viewport_id());
                    }
                })
            },
            visible_fn: {
                let windows = windows.clone();
                Box::new(move || windows.lock().contains(&T::viewport_id()))
            },
            visible_fast_fn: Box::new(|windows| windows.contains(&T::viewport_id())),
            title_fn: Box::new(|| T::title()),
        }
    }

    pub fn new_default<T>(windows: Arc<Mutex<HashSet<ViewportId>>>) -> Self
    where
        T: AppWindow,
        T::State: Default,
    {
        Self::new::<T>(windows, T::State::default())
    }

    pub fn show_viewport_deferred(&self, ctx: &egui::Context) {
        (self.show_viewport_deferred_fn)(ctx);
    }

    pub fn save(&self, storage: &mut dyn eframe::Storage) {
        (self.save_fn)(storage);
    }

    pub fn viewport_id(&self) -> ViewportId {
        (self.viewport_id_fn)()
    }

    pub fn category(&self) -> AppWindowCategory {
        (self.category_fn)()
    }

    pub fn set_visibility(&self, visible: bool) {
        (self.set_visibility_fn)(visible);
    }

    #[allow(dead_code)]
    pub fn visible(&self) -> bool {
        (self.visible_fn)()
    }

    pub fn visible_fast(&self, windows: &HashSet<ViewportId>) -> bool {
        (self.visible_fast_fn)(windows)
    }

    pub fn title(&self) -> String {
        (self.title_fn)()
    }
}
