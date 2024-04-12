use egui::*;
use maybe_owned::MaybeOwnedMut;
use std::ops::{Deref, DerefMut};

use crate::*;

pub enum ExUiMode {
    Compact {
        ui_row: Option<FrameRun>,
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
        self.1.column = 0;
        if let ExUiMode::Compact {
            ref mut ui_row,
            ref mut ui_columns,
        } = self.1.mode
        {
            let rect_columns = ui_columns.as_ref().map_or(Rect::NOTHING, |u| u.min_rect());
            {
                let ui_row = ui_row.as_mut().unwrap();
                ui_row
                    .content_ui
                    .allocate_rect(rect_columns, Sense::hover());

                ui_row.end(&mut self.0);
            }
            self.0.end_row();
            *ui_row = Some(FrameRun::begin(Frame::group(self.0.style()), &mut self.0));
            *ui_columns = None;
            //TODO add frame configuration
        } else {
            self.0.end_row()
        }
    }

    pub(crate) fn add_ex<R>(&mut self, add_contents: impl FnOnce(&mut Ui) -> R) -> R {
        self.1.column += 1;
        let column = self.1.column;
        if let ExUiMode::Compact {
            ref mut ui_row,
            ref mut ui_columns,
        } = self.1.mode
        {
            match column {
                1 => add_contents(&mut ui_row.as_mut().unwrap().content_ui),
                2 => {
                    let ui = &mut ui_row.as_mut().unwrap().content_ui;
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
            add_contents(&mut self.0)
        }
    }

    /// In grid mode this is the same as `Self::label`, in compact mode it uses larger font (currently `Self::heading`).
    pub fn extext(&mut self, text: impl Into<RichText>) -> Response {
        if matches!(self.1.mode, ExUiMode::Compact { .. }) && self.1.column == 0 {
            self.heading(text)
        } else {
            self.label(text.into())
        }
    }

    /// Add row with subdata, subdata rows are hidden behind collapsible
    /// Returns (collapsing button response, header/parrent_row response, body/childs response [None if closed])
    pub fn subdata<HeaderRet, BodyRet>(
        &mut self,
        parrent_row: impl FnOnce(&mut ExUi) -> HeaderRet,
        child_rows: impl FnOnce(&mut ExUi) -> BodyRet,
    ) -> (
        Response,
        InnerResponse<HeaderRet>,
        Option<InnerResponse<BodyRet>>,
    ) {
        (todo!(), todo!(), todo!())
    }
    pub fn skip_cell(&mut self) {}
}
