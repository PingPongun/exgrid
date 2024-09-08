use maybe_owned::MaybeOwnedMut;
use std::hash::Hash;

use crate::*;

impl<'a, 'b: 'a> ExUi<'a, 'b> {
    // ------------------------------------------------------------------------
    // Creation:

    /// Create a new [`ExUi`].
    ///
    /// Normally you would not use this directly, but instead use
    /// [`ExGrid::show`].
    #[inline]
    pub fn new(ctx: Context, layer_id: LayerId, id: Id, max_rect: Rect, clip_rect: Rect) -> Self {
        ExUi {
            ui: MaybeOwnedMut::Owned(Ui::new(
                ctx,
                layer_id,
                id,
                max_rect,
                clip_rect,
                #[cfg(feature = "egui28")]
                Default::default(),
            )),
            state: Default::default(),
            keep_cell: None,
            temp_ui: None,
        }
    }

    /// Create a new [`ExUi`] at a specific region.
    pub fn child_ui(&mut self, max_rect: Rect, layout: Layout) -> Self {
        self.child_ui_with_id_source(max_rect, layout, "child")
    }

    /// Create a new [`ExUi`]
    pub fn simple_child(&mut self) -> Self {
        self.child_ui(self.available_rect_before_wrap(), self.layout().clone())
    }

    /// Create a new [`ExUi`] at a specific region with a specific id.
    #[inline]
    pub fn child_ui_with_id_source(
        &mut self,
        max_rect: Rect,
        layout: Layout,
        id_source: impl Hash,
    ) -> Self {
        ExUi {
            ui: MaybeOwnedMut::Owned(self.ui.child_ui_with_id_source(
                max_rect,
                layout,
                id_source,
                #[cfg(feature = "egui28")]
                None,
            )),
            state: Default::default(),
            keep_cell: None,
            temp_ui: None,
        }
    }
}
// ------------------------------------------------------------------------

/// # [`Id`] creation
impl<'a, 'b> ExUi<'a, 'b> {
    /// A unique identity of this [`ExUi`].
    /// Differently to [`egui::UI::id`], it changes as widgets are added to it (it is calculated based on row number on each nesting level).
    #[inline]
    pub fn id(&self) -> Id {
        self.ui.id().with(self.state.row_cursor.as_slice())
    }
    /// Use this to generate widget ids for widgets that have persistent state in [`Memory`].
    pub fn make_persistent_id<IdSource>(&self, id_source: IdSource) -> Id
    where
        IdSource: Hash,
    {
        self.id().with(&id_source)
    }
}
// ------------------------------------------------------------------------

/// # Adding widgets
impl<'a, 'b> ExUi<'a, 'b> {
    /// Add a [`Widget`] to this [`Ui`] at a location dependent on the current [`Layout`].
    ///
    /// The returned [`Response`] can be used to check for interactions,
    /// as well as adding tooltips using [`Response::on_hover_text`].
    ///
    /// See also [`Self::add_sized`] and [`Self::put`].
    ///
    /// ```
    /// # egui::__run_test_ui(|ui| {
    /// # let mut my_value = 42;
    /// let response = ui.add(egui::Slider::new(&mut my_value, 0..=100));
    /// response.on_hover_text("Drag me!");
    /// # });
    /// ```
    #[inline]
    pub fn add(&mut self, widget: impl Widget) -> Response {
        self.add_ex_opt(|ui| widget.ui(ui))
            .unwrap_or_else(|| self.dummy_response())
    }
}
