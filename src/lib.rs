#[cfg(feature = "egui21")]
use egui21 as egui;
#[cfg(feature = "egui22")]
use egui22 as egui;
#[cfg(feature = "egui23")]
use egui23 as egui;
#[cfg(feature = "egui24")]
use egui24 as egui;
#[cfg(feature = "egui25")]
use egui25 as egui;
#[cfg(feature = "egui26")]
use egui26 as egui;
#[cfg(feature = "egui27")]
use egui27 as egui;

use egui::layers::ShapeIdx;
use egui::*;

mod exui;
mod ui_wrapper;
pub use exui::*;

#[derive(Clone, Debug, Default, PartialEq)]
/// Configures [`ExGrid`] Layout
/// - `Traditional` - will result in traditional grid layout (row is in single line)
/// - `CompactWidth` - will result in not grid layout (width-compact, `egui::Group` based layout)
// /// - `Auto0Wrap` - decision between above will be taken based on available and content width(assuming that in grid mode content will not wrap)
// /// - `AutoOptimalWrap` - similar to `Auto0Wrap` but in grid mode cell content(if necessary) will wrap to some extent
pub enum GridMode {
    // Auto0Wrap,
    // AutoOptimalWrap,
    #[default]
    CompactWidth,
    Traditional,
}

// ----------------------------------------------------------------------------

/// ExGrid- drop-in replacement for [`egui::Grid`] with superpowers:
///
/// - alternative/improved layout mode dedicated to narrow windows (it's not grid there, but rather some group based layout)
/// - supports "subdata" (rows that are indented, hidden behind collapsible, but columns are still aligned with top grid)
///
/// If you want to add multiple widgets to a cell you need to group them with
/// [`Ui::horizontal`], [`Ui::vertical`] etc.
///
/// ```
/// # egui::__run_test_ui(|ui| {
/// ExGrid::new("some_unique_id").show(ui, |ui| {
///     ui.label("First row, first column");
///     ui.label("First row, second column");
///     ui.end_row();
///
///     ui.label("Second row, first column");
///     ui.label("Second row, second column");
///     ui.label("Second row, third column");
///     ui.end_row();
///
///     ui.horizontal(|ui| { ui.label("Same"); ui.label("cell"); });
///     ui.label("Third row, second column");
///     ui.end_row();
/// });
/// # });
/// ```
#[must_use = "You should call .show()"]
pub struct ExGrid {
    grid: Grid,
    mode: GridMode,
}

impl ExGrid {
    /// Create a new [`ExGrid`] with a locally unique identifier.
    pub fn new(id_source: impl std::hash::Hash) -> Self {
        Self {
            grid: Grid::new(id_source),
            mode: Default::default(),
        }
    }

    /// Setting this will allow for dynamic coloring of rows of the grid object
    /// Matters only in Grid view
    #[inline]
    pub fn with_row_color<F>(mut self, color_picker: F) -> Self
    where
        F: Send + Sync + Fn(usize, &Style) -> Option<Color32> + 'static,
    {
        self.grid = self.grid.with_row_color(color_picker);
        self
    }

    /// Setting this will allow the last column to expand to take up the rest of the space of the parent [`Ui`].
    /// Matters only in Grid view
    #[inline]
    pub fn num_columns(mut self, num_columns: usize) -> Self {
        self.grid = self.grid.num_columns(num_columns);
        self
    }

    /// If `true`, add a subtle background color to every other row.
    ///
    /// This can make a table easier to read.
    /// Default is whatever is in [`crate::Visuals::striped`].
    /// Matters only in Grid view
    pub fn striped(mut self, striped: bool) -> Self {
        self.grid = self.grid.striped(striped);
        self
    }

    /// Set minimum width of each column.
    /// Default: [`crate::style::Spacing::interact_size`]`.x`.
    /// Matters only in Grid view
    #[inline]
    pub fn min_col_width(mut self, min_col_width: f32) -> Self {
        self.grid = self.grid.min_col_width(min_col_width);
        self
    }

    /// Set minimum height of each row.
    /// Default: [`crate::style::Spacing::interact_size`]`.y`.
    /// Matters only in Grid view
    #[inline]
    pub fn min_row_height(mut self, min_row_height: f32) -> Self {
        self.grid = self.grid.min_row_height(min_row_height);
        self
    }

    /// Set soft maximum width (wrapping width) of each column.
    /// Matters only in Grid view
    #[inline]
    pub fn max_col_width(mut self, max_col_width: f32) -> Self {
        self.grid = self.grid.max_col_width(max_col_width);
        self
    }

    /// Set spacing between columns/rows.
    /// Default: [`crate::style::Spacing::item_spacing`].
    /// Matters only in Grid view
    #[inline]
    pub fn spacing(mut self, spacing: impl Into<Vec2>) -> Self {
        self.grid = self.grid.spacing(spacing);
        self
    }

    /// Change which row number the grid starts on.
    /// This can be useful when you have a large [`ExGrid`] inside of [`ScrollArea::show_rows`].
    /// Matters only in Grid view
    #[inline]
    pub fn start_row(mut self, start_row: usize) -> Self {
        self.grid = self.grid.start_row(start_row);
        self
    }
}
impl ExGrid {
    /// Change how grid will be shown
    #[inline]
    pub fn mode(mut self, mode: GridMode) -> Self {
        self.mode = mode;
        self
    }
}

impl ExGrid {
    /// Show `ExGrid` in supplied `ui` and add `add_contents` to it.
    pub fn show<R>(
        self,
        ui: &mut Ui,
        add_contents: impl FnOnce(&mut ExUi) -> R,
    ) -> InnerResponse<R> {
        if self.mode == GridMode::Traditional {
            let add_contents = |ui: &mut Ui| {
                let id = ui.id();
                let mut ex = ui.into();
                let ret = add_contents(&mut ex);
                ex.data_mut(|d| d.insert_temp(id, ex.state.width_max));
                ret
            };
            self.grid.show(ui, add_contents)
        } else {
            let add_contents = |ui: &mut Ui| {
                let id = ui.id();
                let mut ex: ExUi<'_, '_> = ui.into();
                ex.state.mode = ExUiMode::Compact {
                    ui_row: vec![FrameRun::begin(Frame::group(ex.ui.style()), 0, &mut ex.ui)],
                    ui_columns: None,
                };
                let ret = add_contents(&mut ex);
                if ex.state.column != 0 {
                    ex.end_row()
                }
                ex.data_mut(|d| d.insert_temp(id, ex.state.width_max));
                ret
            };
            self.grid.num_columns(1).show(ui, add_contents)
        }
    }
}

/// Similar to [`egui::Widget`], but for use with [`ExUi`]
///
/// Anything implementing Widget can be added to a [`ExUi`] with [`ExUi::add`], but this trait is if you prefer to use `widget.ui_ex(ui)`.
///
/// This trait is implemented for all types that implement [`egui::Widget`]
#[must_use = "You should put this widget in an ui with `ui.add(widget);`"]
pub trait ExWidget {
    /// Allocate space, interact, paint, and return a [`Response`].
    ///
    /// Note that this consumes `self`.
    /// This is because most widgets ([`Button`], [`TextEdit`] etc) are
    /// [builders](https://doc.rust-lang.org/1.0.0/style/ownership/builders.html)
    ///
    /// Tip: you can `impl Widget for &mut YourObject { }`.
    fn ui_ex(self, ex_ui: &mut ExUi) -> Response;
}
impl<T: Widget> ExWidget for T {
    fn ui_ex(self, ex: &mut ExUi) -> Response {
        ex.add_ex_opt(|ui| self.ui(ui))
            .unwrap_or_else(|| ex.dummy_response())
    }
}

/// Exactly the same as [`ExWidget`], but for convenience function is named as in [`egui::Widget`]
#[must_use = "You should put this widget in an ui with `ui.add(widget);`"]
pub trait ExWidgetConvenience {
    /// Allocate space, interact, paint, and return a [`Response`].
    ///
    /// Note that this consumes `self`.
    /// This is because most widgets ([`Button`], [`TextEdit`] etc) are
    /// [builders](https://doc.rust-lang.org/1.0.0/style/ownership/builders.html)
    ///
    /// Tip: you can `impl Widget for &mut YourObject { }`.
    fn ui(self, ex_ui: &mut ExUi) -> Response;
}
impl<T: ExWidget> ExWidgetConvenience for T {
    fn ui(self, ex: &mut ExUi) -> Response {
        self.ui_ex(ex)
    }
}

/// Similar to `egui::containers::frame::Prepared`, but this is necessary as:
/// - `frame` module is private
/// - `Prepared::end(..)` requires owned self
pub(crate) struct FrameRun {
    empty: bool,
    pub frame: Frame,
    where_to_put_background: ShapeIdx,
    indent: usize,
    pub content_ui: Ui,
    pub parent_width: f32,
}

impl FrameRun {
    pub fn begin(frame: Frame, indent: usize, ui: &mut Ui) -> FrameRun {
        let where_to_put_background = ui.painter().add(Shape::Noop);
        let outer_rect_bounds = ui.available_rect_before_wrap();

        let mut inner_rect =
            (frame.inner_margin + frame.outer_margin).shrink_rect(outer_rect_bounds);
        if indent > 1 {
            inner_rect.min.x += ui.style().spacing.indent;
        }

        // Make sure we don't shrink to the negative:
        inner_rect.max.x = inner_rect.max.x.max(inner_rect.min.x);
        inner_rect.max.y = inner_rect.max.y.max(inner_rect.min.y);

        let content_ui = ui.child_ui(inner_rect, Layout::top_down_justified(Align::LEFT));

        FrameRun {
            empty: true,
            frame,
            where_to_put_background,
            indent,
            content_ui,
            parent_width: ui.min_rect().max.y,
        }
    }

    fn paint_rect(&self) -> Rect {
        let mut rect = self.content_ui.min_rect();
        rect.max.x = rect.max.x.max(self.parent_width);
        self.frame.inner_margin.expand_rect(rect)
    }

    fn content_with_margin(&self) -> Rect {
        (self.frame.total_margin() + egui::Margin::same(self.frame.stroke.width))
            .expand_rect(self.content_ui.min_rect())
    }

    pub fn end(&mut self, max_x: f32, advance_before: Rect) {
        let width_from_previous = max_x
            - ((self.indent.wrapping_sub(1)) as f32
                * (self.frame.total_margin().right + self.frame.stroke.width));
        self.content_ui.advance_cursor_after_rect(advance_before);
        self.content_ui
            .expand_to_include_rect(self.content_ui.min_rect().with_max_x(width_from_previous));
        if !self.empty {
            let paint_rect = self.paint_rect();

            let FrameRun {
                frame,
                where_to_put_background,
                ..
            } = self;

            if self.content_ui.is_rect_visible(paint_rect) {
                let shape = frame.paint(paint_rect);
                self.content_ui
                    .painter()
                    .set(*where_to_put_background, shape);
            }

            self.content_ui
                .advance_cursor_after_rect(self.content_with_margin());
        }
    }

    pub fn ui(&mut self) -> &mut Ui {
        self.empty = false;
        &mut self.content_ui
    }
}
