use egui::mutex::Mutex;
use maybe_owned::MaybeOwnedMut;
use once_cell::sync::Lazy;
use std::any::Any;
use std::borrow::BorrowMut;
use std::ops::{Deref, DerefMut};

use crate::*;

pub(crate) enum ExUiMode {
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
pub(crate) struct ExUiInner {
    pub(crate) column: usize,
    pub(crate) collapsing_header: bool,
    pub(crate) row_cursor: Vec<usize>,
    pub(crate) width_max_prev: f32,
    pub(crate) width_max: f32,
    /// 0 means widgets will be added in enabled state,
    /// higher numbers indicate number of `Self::start_disabled` not matched by `Self::stop_disabled`
    pub(crate) disabled: usize,

    pub(crate) mode: ExUiMode,
    pub(crate) collapsed: Vec<bool>,
}

impl Default for ExUiInner {
    fn default() -> Self {
        Self {
            column: 0,
            collapsing_header: false,
            width_max_prev: 0.0,
            width_max: 0.0,
            disabled: 0,
            row_cursor: vec![0],
            mode: Default::default(),
            collapsed: vec![false],
        }
    }
}
/// Wrapper for [`egui::Ui`] (most functions will work exactly the same; `ExUi` derefs to `Ui` for all not implemented functions),
/// but with some additional state & functions to manage adding widgets inside [`ExGrid`].
pub struct ExUi<'a, 'b> {
    pub ui: MaybeOwnedMut<'a, Ui>,
    pub(crate) state: MaybeOwnedMut<'b, ExUiInner>,
    /// (cell ui, widgets in cell)
    pub(crate) keep_cell: Option<(MaybeOwnedMut<'a, Ui>, usize)>,
    pub(crate) temp_ui: Option<MaybeOwnedMut<'a, Ui>>,
}

impl<'a, 'b> Deref for ExUi<'a, 'b> {
    type Target = Ui;

    fn deref(&self) -> &Self::Target {
        &self.ui
    }
}
impl<'a, 'b> ExUi<'a, 'b> {
    fn _ui(&mut self) -> &mut Ui {
        if let ExUiMode::Compact {
            ref mut ui_row,
            ref mut ui_columns,
        } = self.state.mode
        {
            if let Some(ref mut col) = ui_columns {
                return col;
            } else {
                return &mut ui_row.last_mut().unwrap().content_ui;
            }
        } else {
            return self.ui.as_mut();
        }
    }
    fn advance_temp_rect(&mut self) {
        if self.collapsed() || self.keep_cell.is_some() {
            return;
        }
        let temp_rect = self
            .temp_ui
            .as_ref()
            .map(|ui| {
                if ui.is_visible() {
                    Some(ui.min_rect())
                } else {
                    None
                }
            })
            .flatten();
        if let Some(rect) = temp_rect {
            self._ui().advance_cursor_after_rect(rect);
        }
        self.temp_ui = None;
    }
}
fn disable_ui<'a: 'd, 'b: 'd, 'c: 'd, 'd>(
    temp_ui: &'c mut Option<MaybeOwnedMut<'a, Ui>>,
    ui: &'b mut Ui,
    disabled: usize,
) -> &'d mut Ui {
    if disabled != 0 {
        let mut ui = simpleui(ui);
        ui.disable();
        *temp_ui = Some(MaybeOwnedMut::Owned(ui));
        return temp_ui.as_mut().unwrap();
    } else {
        return ui;
    }
}
impl<'a, 'b> DerefMut for ExUi<'a, 'b> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        if self.collapsed() {
            let ctx = self.ctx().clone();
            let rect = self._ui().min_rect();
            self.state.column += 1;
            if let Some(ui) = &mut self.temp_ui {
                ui.skip_ahead_auto_ids(1);
            }
            #[cfg(feature = "egui29")]
            let u: &mut MaybeOwnedMut<'a, Ui> = self.temp_ui.get_or_insert_with(|| {
                MaybeOwnedMut::Owned(Ui::new(
                    ctx,
                    LayerId::debug(),
                    "dummy".into(),
                    UiBuilder {
                        max_rect: Some(rect),
                        invisible: true,
                        ..Default::default()
                    },
                ))
            });
            #[cfg(not(feature = "egui29"))]
            let u: &mut MaybeOwnedMut<'a, Ui> = self.temp_ui.get_or_insert_with(|| {
                MaybeOwnedMut::Owned(Ui::new(
                    ctx,
                    LayerId::debug(),
                    "dummy".into(),
                    rect,
                    Rect::NOTHING,
                    #[cfg(feature = "egui28")]
                    Default::default(),
                ))
            });

            #[cfg(feature = "egui28")]
            u.set_invisible();
            #[cfg(not(any(feature = "egui28", feature = "egui29")))]
            u.set_visible(false);
            return u;
        }
        self.advance_temp_rect();
        let id = self.id();
        if let Some((ui, wic)) = &mut self.keep_cell {
            *wic += 1;
            disable_ui(&mut self.temp_ui, ui, self.state.disabled)
        } else {
            self.state.column += 1;
            let ExUiInner {
                column,
                collapsing_header,
                row_cursor,
                mode,
                disabled,
                ..
            } = self.state.as_mut();

            if let ExUiMode::Compact {
                ref mut ui_row,
                ref mut ui_columns,
            } = mode
            {
                match column {
                    1 => {
                        let mut ui = simpleui(ui_row.last_mut().unwrap().ui());
                        if *collapsing_header {
                            let collapsed =
                                self.ui.data_mut(|d| d.get_temp_mut_or(id, false).clone());
                            let icon = if collapsed { "⏵" } else { "⏷" };
                            if ui.add(Button::new(icon).frame(false).small()).clicked() {
                                self.ui.data_mut(|d| d.insert_temp(id, !collapsed));
                            }
                        };
                        if *disabled != 0 {
                            ui.disable();
                        }
                        self.temp_ui = Some(MaybeOwnedMut::Owned(ui));
                        return self.temp_ui.as_mut().unwrap();
                    }
                    2 => {
                        let ui = &mut ui_row.last_mut().unwrap().ui();
                        let indent = ui.spacing().indent;
                        let mut child_rect = ui.available_rect_before_wrap();
                        child_rect.min.x += indent;

                        #[cfg(feature = "egui29")]
                        {
                            *ui_columns = Some(ui.new_child(UiBuilder {
                                id_salt: Some("indent".into()),
                                max_rect: Some(child_rect),
                                layout: Some(
                                    Layout::left_to_right(Align::TOP).with_main_wrap(true),
                                ),
                                ..Default::default()
                            }));
                        }
                        #[cfg(not(feature = "egui29"))]
                        {
                            *ui_columns = Some(ui.child_ui_with_id_source(
                                child_rect,
                                Layout::left_to_right(Align::TOP).with_main_wrap(true),
                                "indent",
                                #[cfg(feature = "egui28")]
                                None,
                            ));
                        }

                        return disable_ui(
                            &mut self.temp_ui,
                            ui_columns.as_mut().unwrap(),
                            *disabled,
                        );
                    }
                    _ => {
                        if let Some(ref mut col) = ui_columns {
                            col.separator();
                            return disable_ui(&mut self.temp_ui, col, *disabled);
                        } else {
                            unreachable!()
                        }
                    }
                }
            } else {
                if *collapsing_header && *column == 1 {
                    let mut ui = simpleui(self.ui.as_mut());
                    for _ in 0..row_cursor.len() - 2 {
                        ui.separator();
                    }
                    let collapsed = ui.data_mut(|d| d.get_temp_mut_or(id, false).clone());
                    let icon = if collapsed { "⏵" } else { "⏷" };
                    if ui.add(Button::new(icon).frame(false).small()).clicked() {
                        ui.data_mut(|d| d.insert_temp(id, !collapsed));
                    }
                    if *disabled != 0 {
                        ui.disable();
                    }
                    self.temp_ui = Some(MaybeOwnedMut::Owned(ui));
                    return self.temp_ui.as_mut().unwrap();
                } else if row_cursor.len() > 1 {
                    //if rows are collapsed, we should not reach here(reaching here should be stopped by `collapsing_rows_body`)
                    if *column == 1 {
                        let mut ui = simpleui(self.ui.as_mut());
                        for _ in 0..row_cursor.len() - 1 {
                            ui.separator();
                        }
                        if *disabled != 0 {
                            ui.disable();
                        }
                        self.temp_ui = Some(MaybeOwnedMut::Owned(ui));
                        return self.temp_ui.as_mut().unwrap();
                    } else {
                        return disable_ui(&mut self.temp_ui, self.ui.as_mut(), *disabled);
                    }
                } else {
                    return disable_ui(&mut self.temp_ui, self.ui.as_mut(), *disabled);
                }
            };
        }
    }
}
impl<'a, 'b> From<&'a mut Ui> for ExUi<'a, 'b> {
    fn from(ui: &'a mut Ui) -> Self {
        let mut inner: ExUiInner = Default::default();
        inner.width_max_prev = ui.data_mut(|d| d.get_temp_mut_or(ui.id(), 0.0).clone());
        ExUi {
            ui: MaybeOwnedMut::Borrowed(ui),
            state: MaybeOwnedMut::Owned(inner),
            keep_cell: None,
            temp_ui: None,
        }
    }
}

impl<'a, 'b> ExUi<'a, 'b> {
    /// Move to the next line in a wrapping layout (GridMode::CompactWidth).
    /// In table mode does nothing.
    pub fn end_row_weak(&mut self) {
        if let ExUiMode::Compact {
            ref mut ui_columns, ..
        } = self.state.mode
        {
            ui_columns.as_mut().map(|x| x.end_row());
        }
    }

    /// Add empty row.
    pub fn empty_row(&mut self) {
        self.end_row();
        self.state.column = 1;
        self.end_row();
    }

    /// Move to the next row.
    /// No-op if we are already at the beginning of the new row.
    pub fn end_row(&mut self) {
        self.keep_cell_stop();
        self.advance_temp_rect();
        if self.state.collapsing_header {
            self.state.collapsing_header = false;
            let id = self.id();
            if !self.collapsed() {
                self.state
                    .collapsed
                    .push(self.ui.data_mut(|d| d.get_temp_mut_or(id, false).clone()));
            }
        }
        if self.state.column != 0 {
            *self.state.row_cursor.last_mut().unwrap() += 1;
        }
        let indent = self.state.row_cursor.len();
        let mut width_max = self.state.width_max;
        let width_max_prev = self.state.width_max_prev;
        let collapsed = self.collapsed();
        if let ExUiMode::Compact {
            ref mut ui_row,
            ref mut ui_columns,
        } = self.state.mode
        {
            let mut rect_columns = ui_columns.as_ref().map_or(
                ui_row
                    .last_mut()
                    .map_or(Rect::NOTHING, |fr| fr.content_ui.min_rect()),
                |u| u.min_rect(),
            );
            *ui_columns = None;
            width_max = width_max.max(rect_columns.max.x);
            if ui_row.len() >= indent {
                //indent kept at the same level
                let mut row_popped = ui_row.pop().unwrap();
                if !collapsed {
                    row_popped.end(width_max.max(width_max_prev), rect_columns);
                    rect_columns = row_popped.ui().min_rect();
                }
            }

            let ui = ui_row.last_mut().map_or(self.ui.borrow_mut(), |x| x.ui());
            ui.advance_cursor_after_rect(rect_columns);
            ui.end_row();
            let fr = FrameRun::begin(Frame::group(ui.style()), indent, ui);
            ui_row.push(fr);

            //TODO add frame configuration
        } else if self.state.column != 0 {
            self.ui.end_row()
        }
        self.state.column = 0;
        self.state.width_max = width_max;
    }

    pub fn start_collapsing(&mut self) {
        if !self.state.collapsing_header {
            self.state.collapsing_header = true;
            self.state.row_cursor.push(0);
        }
    }

    pub fn stop_collapsing(&mut self) {
        self.temp_ui = None;
        self.state.collapsing_header = false;
        if self.state.row_cursor.len() > 1 {
            self.state.row_cursor.pop();
            let len = self.state.row_cursor.len();
            self.state.collapsed.truncate(len);
            let mut width_max = self.state.width_max;
            let width_max_prev = self.state.width_max_prev;
            let collapsed = self.collapsed();
            *self.state.row_cursor.last_mut().unwrap() += 1;

            if let ExUiMode::Compact {
                ref mut ui_row,
                ref mut ui_columns,
            } = self.state.mode
            {
                let rect_columns = ui_columns.as_ref().map_or(
                    ui_row
                        .last_mut()
                        .map_or(Rect::NOTHING, |fr| fr.content_ui.min_rect()),
                    |u| u.min_rect(),
                );
                width_max = width_max.max(rect_columns.max.x);
                let mut row_popped = ui_row.pop().unwrap();
                if !collapsed {
                    row_popped.end(width_max.max(width_max_prev), rect_columns);
                }
                ui_row
                    .last_mut()
                    .map(|u| u.ui().advance_cursor_after_rect(row_popped.ui().min_rect()));
            }
            self.state.width_max = width_max;
        }
        self.end_row();
    }

    pub fn add_ex_opt<R>(&mut self, add_contents: impl FnOnce(&mut Ui) -> R) -> Option<R> {
        if self.collapsed() {
            None
        } else {
            Some(add_contents(self))
        }
    }

    pub fn collapsed(&self) -> bool {
        Some(true) == self.state.collapsed.last().copied()
    }

    /// Adds collapsible rows header
    pub fn collapsing_rows(
        &mut self,
        header_row: impl FnOnce(&mut ExUi) -> Response,
    ) -> CollapsingRows<'a, 'b, '_> {
        self.maybe_collapsing_rows(true, header_row)
    }
    /// If `collapsible` == true, adds collapsible rows header,
    /// otherwise simply adds `header_row` (without collapse/uncollapse button)
    pub fn maybe_collapsing_rows(
        &mut self,
        collapsible: bool,
        header_row: impl FnOnce(&mut ExUi) -> Response,
    ) -> CollapsingRows<'a, 'b, '_> {
        if collapsible {
            self.start_collapsing()
        }
        CollapsingRows {
            header_response: header_row(self),
            exui: self,
            collapsible,
        }
    }

    /// In grid mode this is the same as `Self::label`, in compact mode it uses larger font in first column (currently `Self::heading`).
    pub fn extext(&mut self, text: impl Into<RichText>) -> Response {
        if matches!(self.state.mode, ExUiMode::Compact { .. }) && self.state.column == 0 {
            self.add_ex_opt(|ui| ui.heading(text))
        } else {
            self.add_ex_opt(|ui| ui.label(text.into()))
        }
        .unwrap_or(self.dummy_response())
    }
    pub fn dummy_response(&mut self) -> Response {
        self.interact(
            egui::Rect::NOTHING,
            "dummy".into(),
            egui::Sense {
                click: false,
                drag: false,
                focusable: false,
            },
        )
    }
    pub fn get_column(&self) -> usize {
        self.state.column
    }
    pub fn get_widgets_in_cell(&self) -> Option<usize> {
        self.keep_cell.as_ref().map(|(_, wic)| *wic)
    }
    /// Creates sub-ui which will be used when adding next widgets
    /// (following widgets will stay in the same grid cell)
    /// Consecutive calls will stay with the same sub-ui (no-op)
    /// Sub-ui will be exited when [`Self::keep_cell_stop`] or [`Self::end_row`] is called
    pub fn keep_cell_start(&mut self) {
        if self.keep_cell.is_none() {
            let ui: &mut Ui = &mut *self;
            self.keep_cell = Some((MaybeOwnedMut::Owned(simpleui(ui)), 0));
        }
    }
    pub fn keep_cell_stop(&mut self) {
        let rect = self.keep_cell.as_ref().map(|ui| ui.0.min_rect());

        if let Some(rect) = rect {
            self._ui().advance_cursor_after_rect(rect);
            self.temp_ui = None;
            self.keep_cell = None
        }
    }
}

impl<'a, 'b> ExUi<'a, 'b> {
    /// Until `Self::stop_disabled` all widgets will be added in disabled state
    /// Nested calls are allowed and require matching number of `Self::stop_disabled` to reenable Ui
    pub fn start_disabled(&mut self) {
        self.state.disabled += 1
    }
    /// If Ui has been disabled with `Self::start_disabled`, reenable it
    pub fn stop_disabled(&mut self) {
        self.state.disabled.overflowing_sub(1).0;
    }
}
#[allow(nonstandard_style)]
static __egui_struct_mut_prior_add__: Lazy<
    Mutex<std::collections::HashMap<Id, Box<dyn Any + Send>>>,
> = Lazy::new(|| Default::default());

impl<'a, 'b> ExUi<'a, 'b> {
    /// Insert a value that will not be persisted. (Similar function to `egui::UI::data_mut`, but less strict bounds)
    #[inline]
    pub fn data_store<T: Any + Send>(&mut self, id: Id, value: Box<T>) {
        __egui_struct_mut_prior_add__.lock().insert(id, value);
    }
    /// Remove data from storage.
    #[inline]
    pub fn data_remove<T: Any + Send>(&mut self, id: Id) -> Option<Box<T>> {
        __egui_struct_mut_prior_add__
            .lock()
            .remove(&id)
            .map(|x| x.downcast().ok())
            .flatten()
    }
}

fn simpleui(ui: &mut Ui) -> Ui {
    let max_rect = ui.available_rect_before_wrap();
    let layout = Layout::left_to_right(Default::default());
    #[cfg(feature = "egui29")]
    {
        ui.new_child(UiBuilder {
            max_rect: Some(max_rect),
            layout: Some(layout),
            ..Default::default()
        })
    }
    #[cfg(not(feature = "egui29"))]
    {
        ui.child_ui(
            max_rect,
            layout,
            #[cfg(feature = "egui28")]
            None,
        )
    }
}
#[must_use = "Call [`Self::body(..)`] or [`Self::body_simple(..)`]"]
pub struct CollapsingRows<'a, 'b, 'c> {
    exui: &'c mut ExUi<'a, 'b>,
    header_response: Response,
    collapsible: bool,
}
impl<'a, 'b, 'c> CollapsingRows<'a, 'b, 'c> {
    /// Set initial collapse state of this level collapsible rows. Function will be executed once for each collapsible
    pub fn initial_state(self, start_collapsed: impl FnOnce() -> bool) -> Self {
        let id = self.exui.id();
        self.exui
            .ui
            .data_mut(|d| d.get_temp_mut_or_insert_with(id, start_collapsed).clone());

        self
    }
    /// Add rows with subdata, subdata rows are hidden behind collapsible
    pub fn body(
        self,
        collapsing_rows: impl FnOnce(&mut ExUi) -> Response,
    ) -> CollapsingResponse<()> {
        let id = self.exui.id();
        let collapsed = self
            .exui
            .ui
            .data_mut(|d| d.get_temp_mut_or(id, false).clone());
        let mut ret = CollapsingResponse {
            header_response: self.header_response.clone(),
            body_response: None,
            body_returned: None,
            openness: !collapsed as usize as f32,
        };
        self.exui.end_row();
        if self.collapsible {
            if collapsed {
                if let ExUiMode::Compact { ref mut ui_row, .. } = self.exui.state.mode {
                    let ui = &mut ui_row.last_mut().unwrap().content_ui;
                    #[cfg(feature = "egui29")]
                    let mut child = ui.new_child(UiBuilder {
                        max_rect: Some(ui.available_rect_before_wrap()),
                        layout: Some(Layout::left_to_right(Align::TOP)),
                        ..Default::default()
                    });
                    #[cfg(not(feature = "egui29"))]
                    let mut child = ui.child_ui(
                        ui.available_rect_before_wrap(),
                        Layout::left_to_right(Align::TOP),
                        #[cfg(feature = "egui28")]
                        None,
                    );
                    child.label("⚫⚫⚫");
                    ui.expand_to_include_rect(child.min_rect())
                }
            } else {
                ret.body_response = Some(collapsing_rows(self.exui));
            }
            self.exui.stop_collapsing();
        }
        ret
    }
    /// Same as [`Self::body`] but returns summed responses from body & header
    pub fn body_simple(self, collapsing_rows: impl FnOnce(&mut ExUi) -> Response) -> Response {
        let cr = self.body(collapsing_rows);
        if let Some(body) = cr.body_response {
            body | cr.header_response
        } else {
            cr.header_response
        }
    }
}
