use egui::*;
use maybe_owned::MaybeOwnedMut;
use std::borrow::BorrowMut;
use std::ops::{Deref, DerefMut};

use crate::*;

pub enum ExUiMode {
    Compact {
        ui_row: Vec<FrameRun>,
        ui_columns: Option<Ui>,
    },

    Grid {},
}

impl Default for ExUiMode {
    fn default() -> Self {
        ExUiMode::Grid {}
    }
}
pub struct ExUiInner {
    pub(crate) column: usize,
    pub(crate) start_collapsed: bool,
    pub(crate) row_cursor: Vec<usize>,
    pub(crate) width_max_prev: f32,
    pub(crate) width_max: f32,

    pub(crate) mode: ExUiMode,
    /// at frame start initialized with `widths_used` value from previous frame(is always larger or egual `widths_used`)
    pub(crate) widths_max: Vec<f32>,
    /// at frame start initialized with zero
    pub(crate) widths_used: Vec<f32>,
}

impl Default for ExUiInner {
    fn default() -> Self {
        Self {
            column: 0,
            start_collapsed: false,
            width_max_prev: 0.0,
            width_max: 0.0,
            row_cursor: vec![0],
            mode: Default::default(),
            widths_max: Default::default(),
            widths_used: Default::default(),
        }
    }
}

pub struct ExUi<'a, 'b>(
    pub MaybeOwnedMut<'a, Ui>,
    pub(crate) MaybeOwnedMut<'b, ExUiInner>,
);

impl<'a, 'b> Deref for ExUi<'a, 'b> {
    type Target = Ui;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<'a, 'b> DerefMut for ExUi<'a, 'b> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl<'a, 'b> From<&'a mut Ui> for ExUi<'a, 'b> {
    fn from(ui: &'a mut Ui) -> Self {
        let mut inner: ExUiInner = Default::default();
        inner.width_max_prev = ui.data_mut(|d| d.get_temp_mut_or(ui.id(), 0.0).clone());
        ExUi(MaybeOwnedMut::Borrowed(ui), MaybeOwnedMut::Owned(inner))
    }
}

impl<'a, 'b> ExUi<'a, 'b> {
    /// Move to the next line in a wrapping layout (GridMode::CompactWidth).
    /// In table mode does nothing.
    pub fn end_row_weak(&mut self) {
        if let ExUiMode::Compact {
            ref mut ui_columns, ..
        } = self.1.mode
        {
            ui_columns.as_mut().map(|x| x.end_row());
        }
    }

    /// Add empty row in a grid layout or wrapping layout.
    /// Otherwise does nothing.
    pub fn empty_row(&mut self) {
        self.end_row();
        self.1.column = 1;
        self.end_row();
    }

    /// Move to the next row in a grid layout or wrapping layout.
    /// Otherwise does nothing.
    /// No-op if we are already at the begining of the new row
    pub fn end_row(&mut self) {
        if self.1.column != 0 {
            *self.1.row_cursor.last_mut().unwrap() += 1;
        }
        let indent = self.1.row_cursor.len();
        let mut width_max = self.1.width_max;
        let width_max_prev = self.1.width_max_prev;
        if let ExUiMode::Compact {
            ref mut ui_row,
            ref mut ui_columns,
        } = self.1.mode
        {
            let mut rect_columns = ui_columns.as_ref().map_or(Rect::NOTHING, |u| u.min_rect());
            *ui_columns = None;
            width_max = width_max.max(rect_columns.max.x);
            if ui_row.len() >= indent {
                //indent kept at the same level
                let mut row_poped = ui_row.pop().unwrap();
                row_poped.end(width_max.max(width_max_prev), rect_columns);
                rect_columns = row_poped.ui().min_rect();
            }

            let ui = ui_row.last_mut().map_or(self.0.borrow_mut(), |x| x.ui());
            ui.advance_cursor_after_rect(rect_columns);
            ui.end_row();
            let fr = FrameRun::begin(Frame::group(ui.style()), indent, ui);
            ui_row.push(fr);

            //TODO add frame configuration
        } else if self.1.column != 0 {
            self.0.end_row()
        }
        self.1.column = 0;
        self.1.width_max = width_max;
    }

    fn start_collapsing(&mut self) {
        if !self.1.start_collapsed {
            self.1.start_collapsed = true;
            self.1.row_cursor.push(0);
        }
    }

    fn stop_collapsing(&mut self) {
        self.1.start_collapsed = false;
        if self.1.row_cursor.len() > 1 {
            self.1.row_cursor.pop();
            let mut width_max = self.1.width_max;
            let width_max_prev = self.1.width_max_prev;
            *self.1.row_cursor.last_mut().unwrap() += 1;

            if let ExUiMode::Compact {
                ref mut ui_row,
                ref mut ui_columns,
            } = self.1.mode
            {
                let rect_columns = ui_columns.as_ref().map_or(Rect::NOTHING, |u| u.min_rect());
                width_max = width_max.max(rect_columns.max.x);
                let mut row_poped = ui_row.pop().unwrap();
                row_poped.end(width_max.max(width_max_prev), rect_columns);
                ui_row
                    .last_mut()
                    .map(|u| u.ui().advance_cursor_after_rect(row_poped.ui().min_rect()));
            }
            self.1.width_max = width_max;
        }
    }

    pub(crate) fn add_ex<R>(&mut self, add_contents: impl FnOnce(&mut Ui) -> R) -> R {
        self.1.column += 1;
        let column = self.1.column;
        let mut start_collapsed = self.1.start_collapsed;
        let id = self.id();
        let ret = if let ExUiMode::Compact {
            ref mut ui_row,
            ref mut ui_columns,
        } = self.1.mode
        {
            match column {
                1 => {
                    ui_row
                        .last_mut()
                        .unwrap()
                        .ui()
                        .horizontal(|ui| {
                            if start_collapsed {
                                let collapsed =
                                    self.0.data_mut(|d| d.get_temp_mut_or(id, false).clone());
                                let icon = if collapsed { "⏵" } else { "⏷" };
                                if ui.add(Button::new(icon).frame(false).small()).clicked() {
                                    self.0.data_mut(|d| d.insert_temp(id, !collapsed));
                                }
                                start_collapsed = false;
                            };
                            add_contents(ui)
                        })
                        .inner
                }
                2 => {
                    let ui = &mut ui_row.last_mut().unwrap().ui();
                    let indent = ui.spacing().indent;
                    let mut child_rect = ui.available_rect_before_wrap();
                    child_rect.min.x += indent;

                    *ui_columns = Some(ui.child_ui_with_id_source(
                        child_rect,
                        // Layout::top_down_justified(Align::LEFT),
                        // It would be better to use wrapping-horizontal here, but eguis wrapping isn't smart enough
                        Layout::left_to_right(Align::TOP).with_main_wrap(true),
                        // .with_main_justify(true)
                        // .with_main_align(Align::RIGHT),
                        "indent",
                    ));

                    add_contents(ui_columns.as_mut().unwrap())
                }
                _ => {
                    if let Some(ref mut col) = ui_columns {
                        col.separator();
                        add_contents(col)
                    } else {
                        unreachable!()
                    }
                }
            }
        } else {
            if start_collapsed && column == 1 {
                self.0
                    .horizontal(|ui| {
                        for _ in 0..self.1.row_cursor.len() - 2 {
                            ui.separator();
                        }
                        let collapsed = ui.data_mut(|d| d.get_temp_mut_or(id, false).clone());
                        let icon = if collapsed { "⏵" } else { "⏷" };
                        if ui.add(Button::new(icon).frame(false).small()).clicked() {
                            ui.data_mut(|d| d.insert_temp(id, !collapsed));
                        }
                        start_collapsed = false;
                        add_contents(ui)
                    })
                    .inner
            } else if self.1.row_cursor.len() > 1 {
                //if rows are collapsed, we should not reach here(reaching here should be stoped by `collapsing_rows_body`)
                if column == 1 {
                    self.0
                        .horizontal(|ui| {
                            for _ in 0..self.1.row_cursor.len() - 1 {
                                ui.separator();
                            }

                            add_contents(ui)
                        })
                        .inner
                } else {
                    add_contents(&mut self.0)
                }
            } else {
                add_contents(&mut self.0)
            }
        };
        self.1.start_collapsed = start_collapsed;
        ret
    }

    /// Set initial collapse state of this level collapsible rows. Function will be executed once for each collapsible
    pub fn collapsing_rows_initial_state(&mut self, start_collapsed: impl FnOnce() -> bool) {
        self.start_collapsing();
        let id = self.id();
        self.0
            .data_mut(|d| d.get_temp_mut_or(id, start_collapsed()).clone());
    }
    /// Add row with subdata, subdata rows are hidden behind collapsible
    pub fn collapsing_rows_header<T>(&mut self, header_row: impl FnOnce(&mut ExUi) -> T) -> T {
        self.start_collapsing();
        let ret = header_row(self);
        ret
    }

    /// Add row with subdata, subdata rows are hidden behind collapsible
    pub fn collapsing_rows_body<T>(
        &mut self,
        collapsing_rows: impl FnOnce(&mut ExUi) -> T,
    ) -> Option<T> {
        let id = self.id();
        let collapsed = self.0.data_mut(|d| d.get_temp_mut_or(id, false).clone());
        let body_response;
        self.end_row();
        if collapsed {
            body_response = None;
            if let ExUiMode::Compact { ref mut ui_row, .. } = self.1.mode {
                let ui = &mut ui_row.last_mut().unwrap().content_ui;

                let mut child = ui.child_ui(
                    ui.available_rect_before_wrap(),
                    Layout::left_to_right(Align::TOP),
                );
                child.label("⚫⚫⚫");
                ui.expand_to_include_rect(child.min_rect())
            }
        } else {
            body_response = Some(collapsing_rows(self));
        }
        self.stop_collapsing();
        self.end_row();
        body_response
    }

    /// In grid mode this is the same as `Self::label`, in compact mode it uses larger font in first column (currently `Self::heading`).
    pub fn extext(&mut self, text: impl Into<RichText>) -> Response {
        if matches!(self.1.mode, ExUiMode::Compact { .. }) && self.1.column == 0 {
            self.heading(text)
        } else {
            self.label(text.into())
        }
    }

    pub fn skip_cell(&mut self) {}
}
