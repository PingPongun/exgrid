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
#[derive(Default)]
pub struct ExUiInner {
    pub(crate) column: usize,
    pub(crate) row: usize,
    pub(crate) indent: usize,
    pub(crate) start_collapsed: bool,
    pub(crate) indent_cursor: Vec<usize>,

    pub(crate) mode: ExUiMode,
    /// at frame start initialized with `widths_used` value from previous frame(is always larger or egual `widths_used`)
    pub(crate) widths_max: Vec<f32>,
    /// at frame start initialized with zero
    pub(crate) widths_used: Vec<f32>,
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
    fn from(value: &'a mut Ui) -> Self {
        ExUi(MaybeOwnedMut::Borrowed(value), Default::default())
    }
}
pub struct CollapsingRowsResponse<H, B> {
    pub header_response: H,
    pub body_response: Option<B>,
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

    /// Move to the next row in a grid layout or wrapping layout.
    /// Otherwise does nothing.
    pub fn end_row(&mut self) {
        self.1.row += 1;
        self.1.column = 0;
        let indent = self.1.indent;
        if let ExUiMode::Compact {
            ref mut ui_row,
            ref mut ui_columns,
        } = self.1.mode
        {
            let mut rect_columns = ui_columns.as_ref().map_or(Rect::NOTHING, |u| u.min_rect());
            *ui_columns = None;
            if ui_row.len() >= indent + 1 {
                //indent kept at the same level
                let mut row_poped = ui_row.pop().unwrap();
                row_poped.content_ui.advance_cursor_after_rect(rect_columns);
                row_poped.end();
                rect_columns = row_poped.content_ui.min_rect();
            }

            let ui = ui_row
                .last_mut()
                .map_or(self.0.borrow_mut(), |x| &mut x.content_ui);
            ui.advance_cursor_after_rect(rect_columns);
            ui.end_row();
            let fr = FrameRun::begin(Frame::group(ui.style()), indent > 0, ui);
            ui_row.push(fr);

            //TODO add frame configuration
        } else {
            self.0.end_row()
        }
    }

    fn start_collapsing(&mut self) {
        self.1.indent += 1;
        self.1.start_collapsed = true;
        let row = self.1.row;
        self.1.indent_cursor.push(row);
    }
    fn stop_collapsing(&mut self) {
        self.1.indent -= 1;
        self.1.start_collapsed = false;
        self.1.indent_cursor.pop();

        if let ExUiMode::Compact {
            ref mut ui_row,
            ref mut ui_columns,
        } = self.1.mode
        {
            let rect_columns = ui_columns.as_ref().map_or(Rect::NOTHING, |u| u.min_rect());
            let mut row_poped = ui_row.pop().unwrap();
            row_poped.content_ui.advance_cursor_after_rect(rect_columns);
            row_poped.end();
            ui_row.last_mut().map(|u| {
                u.content_ui
                    .advance_cursor_after_rect(row_poped.content_ui.min_rect())
            });
        }
    }

    pub(crate) fn add_ex<R>(&mut self, add_contents: impl FnOnce(&mut Ui) -> R) -> R {
        self.1.column += 1;
        let column = self.1.column;
        let mut start_collapsed = self.1.start_collapsed;
        let id = if start_collapsed {
            self.0.id().with(self.1.indent_cursor.as_slice())
        } else {
            self.0.id()
        };
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
                        .content_ui
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
                    let ui = &mut ui_row.last_mut().unwrap().content_ui;
                    let indent = ui.spacing().indent;
                    let mut child_rect = ui.available_rect_before_wrap();
                    child_rect.min.x += indent;

                    *ui_columns = Some(ui.child_ui_with_id_source(
                        child_rect,
                        Layout::top_down_justified(Align::RIGHT),
                        // It would be better to use wrapping-horizontal here, but eguis wrapping isn't smart enough
                        // Layout::left_to_right(Align::TOP)
                        //     .with_main_wrap(true)
                        //     .with_main_justify(true)
                        //     .with_main_align(Align::RIGHT),
                        "indent",
                    ));

                    add_contents(ui_columns.as_mut().unwrap())
                }
                _ => {
                    if let Some(ref mut col) = ui_columns {
                        // col.separator();
                        add_contents(col)
                    } else {
                        unreachable!()
                    }
                }
            }
        } else {
            if self.1.start_collapsed && column == 1 {
                self.0
                    .horizontal(|ui| {
                        for _ in 0..self.1.indent - 1 {
                            ui.separator();
                        }
                        let collapsed = ui.data_mut(|d| d.get_temp_mut_or(id, false).clone());
                        let icon = if collapsed { "⏵" } else { "⏷" };
                        if ui.add(Button::new(icon).frame(false).small()).clicked() {
                            ui.data_mut(|d| d.insert_temp(id, !collapsed));
                        }
                        self.1.start_collapsed = false;
                        add_contents(ui)
                    })
                    .inner
            } else if self.1.indent > 0 {
                //if rows are collapsed, we should not reach here(reaching here should be stoped by `add_collapsing``)
                if column == 1 {
                    self.0
                        .horizontal(|ui| {
                            for _ in 0..self.1.indent {
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

    /// Add row with subdata, subdata rows are hidden behind collapsible
    pub fn collapsing_rows<H, B>(
        &mut self,
        header_row: impl FnOnce(&mut ExUi) -> H,
        collapsing_rows: impl FnOnce(&mut ExUi) -> B,
    ) -> CollapsingRowsResponse<H, B> {
        self.start_collapsing();
        let header_response = header_row(self);
        let id = self.0.id().with(self.1.indent_cursor.as_slice());
        let collapsed = self.0.data_mut(|d| d.get_temp_mut_or(id, false).clone());
        let body_response;
        if collapsed {
            body_response = None;
        } else {
            self.end_row();
            body_response = Some(collapsing_rows(self));
        }
        self.stop_collapsing();
        self.end_row();
        CollapsingRowsResponse {
            header_response,
            body_response,
        }
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
