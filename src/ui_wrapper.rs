use egui::epaint::Hsva;
use maybe_owned::MaybeOwnedMut;
use std::hash::Hash;

use crate::*;

impl<'a, 'b> ExUi<'a, 'b> {
    // ------------------------------------------------------------------------
    // Creation:

    /// Create a new [`ExUi`].
    ///
    /// Normally you would not use this directly, but instead use
    /// [`ExGrid::show`].
    #[inline]
    pub fn new(ctx: Context, layer_id: LayerId, id: Id, max_rect: Rect, clip_rect: Rect) -> Self {
        ExUi(
            MaybeOwnedMut::Owned(Ui::new(ctx, layer_id, id, max_rect, clip_rect)),
            Default::default(),
        )
    }

    /// Create a new [`ExUi`] at a specific region.
    pub fn child_ui(&mut self, max_rect: Rect, layout: Layout) -> Self {
        self.child_ui_with_id_source(max_rect, layout, "child")
    }

    /// Create a new [`ExUi`]
    pub fn ui(&mut self) -> Self {
        self.child_ui(self.max_rect(), self.layout().clone())
    }

    /// Create a new [`ExUi`] at a specific region with a specific id.
    #[inline]
    pub fn child_ui_with_id_source(
        &mut self,
        max_rect: Rect,
        layout: Layout,
        id_source: impl Hash,
    ) -> Self {
        ExUi(
            MaybeOwnedMut::Owned(Ui::child_ui_with_id_source(
                &mut self.0,
                max_rect,
                layout,
                id_source,
            )),
            Default::default(),
        )
    }
}
// ------------------------------------------------------------------------

/// # [`Id`] creation
impl<'a, 'b> ExUi<'a, 'b> {
    /// A unique identity of this [`ExUi`].
    /// Differently to [`egui::UI::id`], it changes as widgets are added to it (it is calculated based on row number on each nesting level).
    #[inline]
    pub fn id(&self) -> Id {
        self.0.id().with(self.1.row_cursor.as_slice())
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
        self.add_ex(|ui| widget.ui(ui))
    }

    /// Add a [`Widget`] to this [`Ui`] with a given size.
    /// The widget will attempt to fit within the given size, but some widgets may overflow.
    ///
    /// To fill all remaining area, use `ui.add_sized(ui.available_size(), widget);`
    ///
    /// See also [`Self::add`] and [`Self::put`].
    ///
    /// ```
    /// # let mut my_value = 42;
    /// # egui::__run_test_ui(|ui| {
    /// ui.add_sized([40.0, 20.0], egui::DragValue::new(&mut my_value));
    /// # });
    /// ```
    pub fn add_sized(&mut self, max_size: impl Into<Vec2>, widget: impl Widget) -> Response {
        //TODO
        // TODO(emilk): configure to overflow to main_dir instead of centered overflow
        // to handle the bug mentioned at https://github.com/emilk/egui/discussions/318#discussioncomment-627578
        // and fixed in https://github.com/emilk/egui/commit/035166276322b3f2324bd8b97ffcedc63fa8419f
        //
        // Make sure we keep the same main direction since it changes e.g. how text is wrapped:
        let layout = Layout::centered_and_justified(self.layout().main_dir());
        self.allocate_ui_with_layout(max_size.into(), layout, |ui| ui.add(widget))
            .inner
    }

    /// Add a single [`Widget`] that is possibly disabled, i.e. greyed out and non-interactive.
    ///
    /// If you call `add_enabled` from within an already disabled [`Ui`],
    /// the widget will always be disabled, even if the `enabled` argument is true.
    ///
    /// See also [`Self::add_enabled_ui`] and [`Self::is_enabled`].
    ///
    /// ```
    /// # egui::__run_test_ui(|ui| {
    /// ui.add_enabled(false, egui::Button::new("Can't click this"));
    /// # });
    /// ```
    pub fn add_enabled(&mut self, enabled: bool, widget: impl Widget) -> Response {
        //TODO
        self.0.add_enabled(enabled, widget)
    }

    /// Add a section that is possibly disabled, i.e. greyed out and non-interactive.
    ///
    /// If you call `add_enabled_ui` from within an already disabled [`Ui`],
    /// the result will always be disabled, even if the `enabled` argument is true.
    ///
    /// See also [`Self::add_enabled`] and [`Self::is_enabled`].
    ///
    /// ### Example
    /// ```
    /// # egui::__run_test_ui(|ui| {
    /// # let mut enabled = true;
    /// ui.checkbox(&mut enabled, "Enable subsection");
    /// ui.add_enabled_ui(enabled, |ui| {
    ///     if ui.button("Button that is not always clickable").clicked() {
    ///         /* … */
    ///     }
    /// });
    /// # });
    /// ```
    pub fn add_enabled_ui<R>(
        &mut self,
        enabled: bool,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> InnerResponse<R> {
        self.scope(|ui| {
            ui.set_enabled(enabled);
            add_contents(ui)
        })
    }

    /// Add a single [`Widget`] that is possibly invisible.
    ///
    /// An invisible widget still takes up the same space as if it were visible.
    ///
    /// If you call `add_visible` from within an already invisible [`Ui`],
    /// the widget will always be invisible, even if the `visible` argument is true.
    ///
    /// See also [`Self::add_visible_ui`], [`Self::set_visible`] and [`Self::is_visible`].
    ///
    /// ```
    /// # egui::__run_test_ui(|ui| {
    /// ui.add_visible(false, egui::Label::new("You won't see me!"));
    /// # });
    /// ```
    pub fn add_visible(&mut self, visible: bool, widget: impl Widget) -> Response {
        //TODO
        self.0.add_visible(visible, widget)
    }

    /// Add a section that is possibly invisible, i.e. greyed out and non-interactive.
    ///
    /// An invisible ui still takes up the same space as if it were visible.
    ///
    /// If you call `add_visible_ui` from within an already invisible [`Ui`],
    /// the result will always be invisible, even if the `visible` argument is true.
    ///
    /// See also [`Self::add_visible`], [`Self::set_visible`] and [`Self::is_visible`].
    ///
    /// ### Example
    /// ```
    /// # egui::__run_test_ui(|ui| {
    /// # let mut visible = true;
    /// ui.checkbox(&mut visible, "Show subsection");
    /// ui.add_visible_ui(visible, |ui| {
    ///     ui.label("Maybe you see this, maybe you don't!");
    /// });
    /// # });
    /// ```
    pub fn add_visible_ui<R>(
        &mut self,
        visible: bool,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> InnerResponse<R> {
        self.scope(|ui| {
            ui.set_visible(visible);
            add_contents(ui)
        })
    }
}

/// # Adding Containers / Sub-uis:
impl<'a, 'b> ExUi<'a, 'b> {
    /// Put into a [`Frame::group`], visually grouping the contents together
    ///
    /// ```
    /// # egui::__run_test_ui(|ui| {
    /// ui.group(|ui| {
    ///     ui.label("Within a frame");
    /// });
    /// # });
    /// ```
    ///
    /// See also [`Self::scope`].
    pub fn group<R>(&mut self, add_contents: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        self.add_ex(|ui| ui.group(add_contents))
    }

    /// Create a child Ui with an explicit [`Id`].
    ///
    /// ```
    /// # egui::__run_test_ui(|ui| {
    /// for i in 0..10 {
    ///     // ui.collapsing("Same header", |ui| { }); // this will cause an ID clash because of the same title!
    ///
    ///     ui.push_id(i, |ui| {
    ///         ui.collapsing("Same header", |ui| { }); // this is fine!
    ///     });
    /// }
    /// # });
    /// ```
    pub fn push_id<R>(
        &mut self,
        id_source: impl Hash,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> InnerResponse<R> {
        self.add_ex(|ui| ui.push_id(id_source, add_contents))
    }

    /// Create a scoped child ui.
    ///
    /// You can use this to temporarily change the [`Style`] of a sub-region, for instance:
    ///
    /// ```
    /// # egui::__run_test_ui(|ui| {
    /// ui.scope(|ui| {
    ///     ui.spacing_mut().slider_width = 200.0; // Temporary change
    ///     // …
    /// });
    /// # });
    /// ```
    pub fn scope<R>(&mut self, add_contents: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        self.push_id("child", add_contents)
    }

    /// Redirect shapes to another paint layer.
    pub fn with_layer_id<R>(
        &mut self,
        layer_id: LayerId,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> InnerResponse<R> {
        self.add_ex(|ui| ui.with_layer_id(layer_id, add_contents))
    }

    /// A [`CollapsingHeader`] that starts out collapsed.
    pub fn collapsing<R>(
        &mut self,
        heading: impl Into<WidgetText>,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> CollapsingResponse<R> {
        self.add_ex(|ui| ui.collapsing(heading, add_contents))
    }

    /// Create a child ui which is indented to the right.
    ///
    /// The `id_source` here be anything at all.
    // TODO(emilk): remove `id_source` argument?
    #[inline]
    pub fn indent<R>(
        &mut self,
        id_source: impl Hash,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> InnerResponse<R> {
        self.add_ex(|ui| ui.indent(id_source, add_contents))
    }

    /// Start a ui with horizontal layout.
    /// After you have called this, the function registers the contents as any other widget.
    ///
    /// Elements will be centered on the Y axis, i.e.
    /// adjusted up and down to lie in the center of the horizontal layout.
    /// The initial height is `style.spacing.interact_size.y`.
    /// Centering is almost always what you want if you are
    /// planning to to mix widgets or use different types of text.
    ///
    /// If you don't want the contents to be centered, use [`Self::horizontal_top`] instead.
    ///
    /// The returned [`Response`] will only have checked for mouse hover
    /// but can be used for tooltips (`on_hover_text`).
    /// It also contains the [`Rect`] used by the horizontal layout.
    ///
    /// ```
    /// # egui::__run_test_ui(|ui| {
    /// ui.horizontal(|ui| {
    ///     ui.label("Same");
    ///     ui.label("row");
    /// });
    /// # });
    /// ```
    ///
    /// See also [`Self::with_layout`] for more options.
    #[inline]
    pub fn horizontal<R>(&mut self, add_contents: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        self.add_ex(|ui| ui.horizontal(add_contents))
    }

    /// Like [`Self::horizontal`], but allocates the full vertical height and then centers elements vertically.
    pub fn horizontal_centered<R>(
        &mut self,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> InnerResponse<R> {
        self.add_ex(|ui| ui.horizontal_centered(add_contents))
    }

    /// Like [`Self::horizontal`], but aligns content with top.
    pub fn horizontal_top<R>(
        &mut self,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> InnerResponse<R> {
        self.add_ex(|ui| ui.horizontal_top(add_contents))
    }

    /// Start a ui with horizontal layout that wraps to a new row
    /// when it reaches the right edge of the `max_size`.
    /// After you have called this, the function registers the contents as any other widget.
    ///
    /// Elements will be centered on the Y axis, i.e.
    /// adjusted up and down to lie in the center of the horizontal layout.
    /// The initial height is `style.spacing.interact_size.y`.
    /// Centering is almost always what you want if you are
    /// planning to to mix widgets or use different types of text.
    ///
    /// The returned [`Response`] will only have checked for mouse hover
    /// but can be used for tooltips (`on_hover_text`).
    /// It also contains the [`Rect`] used by the horizontal layout.
    ///
    /// See also [`Self::with_layout`] for more options.
    pub fn horizontal_wrapped<R>(
        &mut self,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> InnerResponse<R> {
        self.add_ex(|ui| ui.horizontal_wrapped(add_contents))
    }

    /// Start a ui with vertical layout.
    /// Widgets will be left-justified.
    ///
    /// ```
    /// # egui::__run_test_ui(|ui| {
    /// ui.vertical(|ui| {
    ///     ui.label("over");
    ///     ui.label("under");
    /// });
    /// # });
    /// ```
    ///
    /// See also [`Self::with_layout`] for more options.
    #[inline]
    pub fn vertical<R>(&mut self, add_contents: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        self.add_ex(|ui| ui.vertical(add_contents))
    }

    /// Start a ui with vertical layout.
    /// Widgets will be horizontally centered.
    ///
    /// ```
    /// # egui::__run_test_ui(|ui| {
    /// ui.vertical_centered(|ui| {
    ///     ui.label("over");
    ///     ui.label("under");
    /// });
    /// # });
    /// ```
    #[inline]
    pub fn vertical_centered<R>(
        &mut self,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> InnerResponse<R> {
        self.add_ex(|ui| ui.vertical_centered(add_contents))
    }

    /// Start a ui with vertical layout.
    /// Widgets will be horizontally centered and justified (fill full width).
    ///
    /// ```
    /// # egui::__run_test_ui(|ui| {
    /// ui.vertical_centered_justified(|ui| {
    ///     ui.label("over");
    ///     ui.label("under");
    /// });
    /// # });
    /// ```
    pub fn vertical_centered_justified<R>(
        &mut self,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> InnerResponse<R> {
        self.add_ex(|ui| ui.vertical_centered_justified(add_contents))
    }

    /// The new layout will take up all available space.
    ///
    /// ```
    /// # egui::__run_test_ui(|ui| {
    /// ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
    ///     ui.label("world!");
    ///     ui.label("Hello");
    /// });
    /// # });
    /// ```
    ///
    /// If you don't want to use up all available space, use [`Self::allocate_ui_with_layout`].
    ///
    /// See also the helpers [`Self::horizontal`], [`Self::vertical`], etc.
    #[inline]
    pub fn with_layout<R>(
        &mut self,
        layout: Layout,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> InnerResponse<R> {
        self.add_ex(|ui| ui.with_layout(layout, add_contents))
    }

    /// This will make the next added widget centered and justified in the available space.
    ///
    /// Only one widget may be added to the inner `Ui`!
    pub fn centered_and_justified<R>(
        &mut self,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> InnerResponse<R> {
        self.add_ex(|ui| ui.centered_and_justified(add_contents))
    }

    #[inline]
    /// Create a menu button that when clicked will show the given menu.
    ///
    /// If called from within a menu this will instead create a button for a sub-menu.
    ///
    /// ```
    /// # egui::__run_test_ui(|ui| {
    /// ui.menu_button("My menu", |ui| {
    ///     ui.menu_button("My sub-menu", |ui| {
    ///         if ui.button("Close the menu").clicked() {
    ///             ui.close_menu();
    ///         }
    ///     });
    /// });
    /// # });
    /// ```
    ///
    /// See also: [`Self::close_menu`] and [`Response::context_menu`].
    pub fn menu_button<R>(
        &mut self,
        title: impl Into<WidgetText>,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> InnerResponse<Option<R>> {
        self.add_ex(|ui| ui.menu_button(title, add_contents))
    }

    /// Create a menu button with an image that when clicked will show the given menu.
    ///
    /// If called from within a menu this will instead create a button for a sub-menu.
    ///
    /// ```ignore
    /// let img = egui::include_image!("../assets/ferris.png");
    ///
    /// ui.menu_image_button(img, |ui| {
    ///     ui.menu_button("My sub-menu", |ui| {
    ///         if ui.button("Close the menu").clicked() {
    ///             ui.close_menu();
    ///         }
    ///     });
    /// });
    /// ```
    ///
    /// See also: [`Self::close_menu`] and [`Response::context_menu`].
    #[inline]
    pub fn menu_image_button<'c, R>(
        &mut self,
        image: impl Into<Image<'c>>,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> InnerResponse<Option<R>> {
        self.add_ex(|ui| ui.menu_image_button(image, add_contents))
    }
}

//Widget wrappers
impl<'a, 'b> ExUi<'a, 'b> {
    /// Show some text.
    ///
    /// Shortcut for `add(Label::new(text))`
    ///
    /// See also [`Label`].
    ///
    /// ### Example
    /// ```
    /// # egui::__run_test_ui(|ui| {
    /// use egui::{RichText, FontId, Color32};
    /// ui.label("Normal text");
    /// ui.label(RichText::new("Large text").font(FontId::proportional(40.0)));
    /// ui.label(RichText::new("Red text").color(Color32::RED));
    /// # });
    /// ```
    #[inline]
    pub fn label(&mut self, text: impl Into<WidgetText>) -> Response {
        Label::new(text).ui_ex(self)
    }

    /// Show colored text.
    ///
    /// Shortcut for `ui.label(RichText::new(text).color(color))`
    pub fn colored_label(
        &mut self,
        color: impl Into<Color32>,
        text: impl Into<RichText>,
    ) -> Response {
        Label::new(text.into().color(color)).ui_ex(self)
    }

    /// Show large text.
    ///
    /// Shortcut for `ui.label(RichText::new(text).heading())`
    pub fn heading(&mut self, text: impl Into<RichText>) -> Response {
        Label::new(text.into().heading()).ui_ex(self)
    }

    /// Show monospace (fixed width) text.
    ///
    /// Shortcut for `ui.label(RichText::new(text).monospace())`
    pub fn monospace(&mut self, text: impl Into<RichText>) -> Response {
        Label::new(text.into().monospace()).ui_ex(self)
    }

    /// Show text as monospace with a gray background.
    ///
    /// Shortcut for `ui.label(RichText::new(text).code())`
    pub fn code(&mut self, text: impl Into<RichText>) -> Response {
        Label::new(text.into().code()).ui_ex(self)
    }

    /// Show small text.
    ///
    /// Shortcut for `ui.label(RichText::new(text).small())`
    pub fn small(&mut self, text: impl Into<RichText>) -> Response {
        Label::new(text.into().small()).ui_ex(self)
    }

    /// Show text that stand out a bit (e.g. slightly brighter).
    ///
    /// Shortcut for `ui.label(RichText::new(text).strong())`
    pub fn strong(&mut self, text: impl Into<RichText>) -> Response {
        Label::new(text.into().strong()).ui_ex(self)
    }

    /// Show text that is weaker (fainter color).
    ///
    /// Shortcut for `ui.label(RichText::new(text).weak())`
    pub fn weak(&mut self, text: impl Into<RichText>) -> Response {
        Label::new(text.into().weak()).ui_ex(self)
    }

    /// Looks like a hyperlink.
    ///
    /// Shortcut for `add(Link::new(text))`.
    ///
    /// ```
    /// # egui::__run_test_ui(|ui| {
    /// if ui.link("Documentation").clicked() {
    ///     // …
    /// }
    /// # });
    /// ```
    ///
    /// See also [`Link`].
    #[must_use = "You should check if the user clicked this with `if ui.link(…).clicked() { … } "]
    pub fn link(&mut self, text: impl Into<WidgetText>) -> Response {
        Link::new(text).ui_ex(self)
    }

    /// Link to a web page.
    ///
    /// Shortcut for `add(Hyperlink::new(url))`.
    ///
    /// ```
    /// # egui::__run_test_ui(|ui| {
    /// ui.hyperlink("https://www.egui.rs/");
    /// # });
    /// ```
    ///
    /// See also [`Hyperlink`].
    pub fn hyperlink(&mut self, url: impl ToString) -> Response {
        Hyperlink::new(url).ui_ex(self)
    }

    /// Shortcut for `add(Hyperlink::from_label_and_url(label, url))`.
    ///
    /// ```
    /// # egui::__run_test_ui(|ui| {
    /// ui.hyperlink_to("egui on GitHub", "https://www.github.com/emilk/egui/");
    /// # });
    /// ```
    ///
    /// See also [`Hyperlink`].
    pub fn hyperlink_to(&mut self, label: impl Into<WidgetText>, url: impl ToString) -> Response {
        Hyperlink::from_label_and_url(label, url).ui_ex(self)
    }

    /// No newlines (`\n`) allowed. Pressing enter key will result in the [`TextEdit`] losing focus (`response.lost_focus`).
    ///
    /// See also [`TextEdit`].
    pub fn text_edit_singleline<S: widgets::text_edit::TextBuffer>(
        &mut self,
        text: &mut S,
    ) -> Response {
        TextEdit::singleline(text).ui_ex(self)
    }

    /// A [`TextEdit`] for multiple lines. Pressing enter key will create a new line.
    ///
    /// See also [`TextEdit`].
    pub fn text_edit_multiline<S: widgets::text_edit::TextBuffer>(
        &mut self,
        text: &mut S,
    ) -> Response {
        TextEdit::multiline(text).ui_ex(self)
    }

    /// A [`TextEdit`] for code editing.
    ///
    /// This will be multiline, monospace, and will insert tabs instead of moving focus.
    ///
    /// See also [`TextEdit::code_editor`].
    pub fn code_editor<S: widgets::text_edit::TextBuffer>(&mut self, text: &mut S) -> Response {
        self.add(TextEdit::multiline(text).code_editor())
    }

    /// Usage: `if ui.button("Click me").clicked() { … }`
    ///
    /// Shortcut for `add(Button::new(text))`
    ///
    /// See also [`Button`].
    ///
    /// ```
    /// # egui::__run_test_ui(|ui| {
    /// if ui.button("Click me!").clicked() {
    ///     // …
    /// }
    ///
    /// # use egui::{RichText, Color32};
    /// if ui.button(RichText::new("delete").color(Color32::RED)).clicked() {
    ///     // …
    /// }
    /// # });
    /// ```
    #[must_use = "You should check if the user clicked this with `if ui.button(…).clicked() { … } "]
    #[inline]
    pub fn button(&mut self, text: impl Into<WidgetText>) -> Response {
        Button::new(text).ui_ex(self)
    }

    /// A button as small as normal body text.
    ///
    /// Usage: `if ui.small_button("Click me").clicked() { … }`
    ///
    /// Shortcut for `add(Button::new(text).small())`
    #[must_use = "You should check if the user clicked this with `if ui.small_button(…).clicked() { … } "]
    pub fn small_button(&mut self, text: impl Into<WidgetText>) -> Response {
        Button::new(text).small().ui_ex(self)
    }

    /// Show a checkbox.
    ///
    /// See also [`Self::toggle_value`].
    #[inline]
    pub fn checkbox(&mut self, checked: &mut bool, text: impl Into<WidgetText>) -> Response {
        Checkbox::new(checked, text).ui_ex(self)
    }

    /// Acts like a checkbox, but looks like a [`SelectableLabel`].
    ///
    /// Click to toggle to bool.
    ///
    /// See also [`Self::checkbox`].
    pub fn toggle_value(&mut self, selected: &mut bool, text: impl Into<WidgetText>) -> Response {
        let mut response = self.selectable_label(*selected, text);
        if response.clicked() {
            *selected = !*selected;
            response.mark_changed();
        }
        response
    }

    /// Show a [`RadioButton`].
    /// Often you want to use [`Self::radio_value`] instead.
    #[must_use = "You should check if the user clicked this with `if ui.radio(…).clicked() { … } "]
    #[inline]
    pub fn radio(&mut self, selected: bool, text: impl Into<WidgetText>) -> Response {
        RadioButton::new(selected, text).ui_ex(self)
    }

    /// Show a [`RadioButton`]. It is selected if `*current_value == selected_value`.
    /// If clicked, `selected_value` is assigned to `*current_value`.
    ///
    /// ```
    /// # egui::__run_test_ui(|ui| {
    ///
    /// #[derive(PartialEq)]
    /// enum Enum { First, Second, Third }
    /// let mut my_enum = Enum::First;
    ///
    /// ui.radio_value(&mut my_enum, Enum::First, "First");
    ///
    /// // is equivalent to:
    ///
    /// if ui.add(egui::RadioButton::new(my_enum == Enum::First, "First")).clicked() {
    ///     my_enum = Enum::First
    /// }
    /// # });
    /// ```
    pub fn radio_value<Value: PartialEq>(
        &mut self,
        current_value: &mut Value,
        alternative: Value,
        text: impl Into<WidgetText>,
    ) -> Response {
        let mut response = self.radio(*current_value == alternative, text);
        if response.clicked() && *current_value != alternative {
            *current_value = alternative;
            response.mark_changed();
        }
        response
    }

    /// Show a label which can be selected or not.
    ///
    /// See also [`SelectableLabel`] and [`Self::toggle_value`].
    #[must_use = "You should check if the user clicked this with `if ui.selectable_label(…).clicked() { … } "]
    pub fn selectable_label(&mut self, checked: bool, text: impl Into<WidgetText>) -> Response {
        SelectableLabel::new(checked, text).ui_ex(self)
    }

    /// Show selectable text. It is selected if `*current_value == selected_value`.
    /// If clicked, `selected_value` is assigned to `*current_value`.
    ///
    /// Example: `ui.selectable_value(&mut my_enum, Enum::Alternative, "Alternative")`.
    ///
    /// See also [`SelectableLabel`] and [`Self::toggle_value`].
    pub fn selectable_value<Value: PartialEq>(
        &mut self,
        current_value: &mut Value,
        selected_value: Value,
        text: impl Into<WidgetText>,
    ) -> Response {
        let mut response = self.selectable_label(*current_value == selected_value, text);
        if response.clicked() && *current_value != selected_value {
            *current_value = selected_value;
            response.mark_changed();
        }
        response
    }

    /// Shortcut for `add(Separator::default())`
    ///
    /// See also [`Separator`].
    #[inline]
    pub fn separator(&mut self) -> Response {
        Separator::default().ui_ex(self)
    }

    /// Shortcut for `add(Spinner::new())`
    ///
    /// See also [`Spinner`].
    #[inline]
    pub fn spinner(&mut self) -> Response {
        Spinner::new().ui_ex(self)
    }

    /// Modify an angle. The given angle should be in radians, but is shown to the user in degrees.
    /// The angle is NOT wrapped, so the user may select, for instance 720° = 2𝞃 = 4π
    pub fn drag_angle(&mut self, radians: &mut f32) -> Response {
        let mut degrees = radians.to_degrees();
        let mut response = self.add(DragValue::new(&mut degrees).speed(1.0).suffix("°"));

        // only touch `*radians` if we actually changed the degree value
        if degrees != radians.to_degrees() {
            *radians = degrees.to_radians();
            response.changed = true;
        }

        response
    }

    /// Modify an angle. The given angle should be in radians,
    /// but is shown to the user in fractions of one Tau (i.e. fractions of one turn).
    /// The angle is NOT wrapped, so the user may select, for instance 2𝞃 (720°)
    pub fn drag_angle_tau(&mut self, radians: &mut f32) -> Response {
        use std::f32::consts::TAU;

        let mut taus = *radians / TAU;
        let mut response = self.add(DragValue::new(&mut taus).speed(0.01).suffix("τ"));

        if self.style().explanation_tooltips {
            response =
                response.on_hover_text("1τ = one turn, 0.5τ = half a turn, etc. 0.25τ = 90°");
        }

        // only touch `*radians` if we actually changed the value
        if taus != *radians / TAU {
            *radians = taus * TAU;
            response.changed = true;
        }

        response
    }

    /// Show an image available at the given `uri`.
    ///
    /// ⚠ This will do nothing unless you install some image loaders first!
    /// The easiest way to do this is via [`egui_extras::install_image_loaders`](https://docs.rs/egui_extras/latest/egui_extras/fn.install_image_loaders.html).
    ///
    /// The loaders handle caching image data, sampled textures, etc. across frames, so calling this is immediate-mode safe.
    ///
    /// ```
    /// # egui::__run_test_ui(|ui| {
    /// ui.image("https://picsum.photos/480");
    /// ui.image("file://assets/ferris.png");
    /// ui.image(egui::include_image!("../assets/ferris.png"));
    /// ui.add(
    ///     egui::Image::new(egui::include_image!("../assets/ferris.png"))
    ///         .max_width(200.0)
    ///         .rounding(10.0),
    /// );
    /// # });
    /// ```
    ///
    /// Using [`include_image`] is often the most ergonomic, and the path
    /// will be resolved at compile-time and embedded in the binary.
    /// When using a "file://" url on the other hand, you need to make sure
    /// the files can be found in the right spot at runtime!
    ///
    /// See also [`crate::Image`], [`crate::ImageSource`].
    #[inline]
    pub fn image<'c>(&mut self, source: impl Into<ImageSource<'c>>) -> Response {
        Image::new(source).ui_ex(self)
    }
}

/// # Colors //TODO PP
impl<'a, 'b> ExUi<'a, 'b> {
    /// Shows a button with the given color.
    /// If the user clicks the button, a full color picker is shown.
    pub fn color_edit_button_srgba(&mut self, srgba: &mut Color32) -> Response {
        self.add(|ui: &mut Ui| ui.color_edit_button_srgba(srgba))
    }

    /// Shows a button with the given color.
    /// If the user clicks the button, a full color picker is shown.
    pub fn color_edit_button_hsva(&mut self, hsva: &mut Hsva) -> Response {
        self.add(|ui: &mut Ui| ui.color_edit_button_hsva(hsva))
    }

    /// Shows a button with the given color.
    /// If the user clicks the button, a full color picker is shown.
    /// The given color is in `sRGB` space.
    pub fn color_edit_button_srgb(&mut self, srgb: &mut [u8; 3]) -> Response {
        self.add(|ui: &mut Ui| ui.color_edit_button_srgb(srgb))
    }

    /// Shows a button with the given color.
    /// If the user clicks the button, a full color picker is shown.
    /// The given color is in linear RGB space.
    pub fn color_edit_button_rgb(&mut self, rgb: &mut [f32; 3]) -> Response {
        self.add(|ui: &mut Ui| ui.color_edit_button_rgb(rgb))
    }

    /// Shows a button with the given color.
    /// If the user clicks the button, a full color picker is shown.
    /// The given color is in `sRGBA` space with premultiplied alpha
    pub fn color_edit_button_srgba_premultiplied(&mut self, srgba: &mut [u8; 4]) -> Response {
        self.add(|ui: &mut Ui| ui.color_edit_button_srgba_premultiplied(srgba))
    }

    /// Shows a button with the given color.
    /// If the user clicks the button, a full color picker is shown.
    /// The given color is in `sRGBA` space without premultiplied alpha.
    /// If unsure, what "premultiplied alpha" is, then this is probably the function you want to use.
    pub fn color_edit_button_srgba_unmultiplied(&mut self, srgba: &mut [u8; 4]) -> Response {
        self.add(|ui: &mut Ui| ui.color_edit_button_srgba_unmultiplied(srgba))
    }

    /// Shows a button with the given color.
    /// If the user clicks the button, a full color picker is shown.
    /// The given color is in linear RGBA space with premultiplied alpha
    pub fn color_edit_button_rgba_premultiplied(&mut self, rgba_premul: &mut [f32; 4]) -> Response {
        self.add(|ui: &mut Ui| ui.color_edit_button_rgba_premultiplied(rgba_premul))
    }

    /// Shows a button with the given color.
    /// If the user clicks the button, a full color picker is shown.
    /// The given color is in linear RGBA space without premultiplied alpha.
    /// If unsure, what "premultiplied alpha" is, then this is probably the function you want to use.
    pub fn color_edit_button_rgba_unmultiplied(&mut self, rgba_unmul: &mut [f32; 4]) -> Response {
        self.add(|ui: &mut Ui| ui.color_edit_button_rgba_unmultiplied(rgba_unmul))
    }
}
