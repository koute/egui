use crate::{
    layout::{CellDirection, CellSize, Layout},
    sizing::Sizing,
    Size,
};
use egui::{Response, Ui};

/// Builder for creating a new [`Strip`].
pub struct StripBuilder<'a> {
    ui: &'a mut Ui,
    sizing: Sizing,
}

impl<'a> StripBuilder<'a> {
    /// Create new strip builder.
    ///
    /// In contrast to normal egui behavior, strip cells do *not* grow with its children!
    ///
    /// After adding size hints with `[Self::column]`/`[Self::columns]` the strip can be build with `[Self::horizontal]`/`[Self::vertical]`.
    ///
    /// ### Example
    /// ```
    /// # egui::__run_test_ui(|ui| {
    /// use egui_extras::{StripBuilder, Size};
    /// StripBuilder::new(ui)
    ///     .size(Size::RemainderMinimum(100.0))
    ///     .size(Size::Absolute(40.0))
    ///     .vertical(|mut strip| {
    ///         strip.strip(|builder| {
    ///             builder.sizes(Size::Remainder, 2).horizontal(|mut strip| {
    ///                 strip.cell(|ui| {
    ///                     ui.label("Top Left");
    ///                 });
    ///                 strip.cell(|ui| {
    ///                     ui.label("Top Right");
    ///                 });
    ///             });
    ///         });
    ///         strip.cell(|ui| {
    ///             ui.label("Fixed");
    ///         });
    ///     });
    /// # });
    /// ```
    pub fn new(ui: &'a mut Ui) -> Self {
        let sizing = Sizing::new();

        Self { ui, sizing }
    }

    /// Add size hint for column/row
    pub fn size(mut self, size: Size) -> Self {
        self.sizing.add(size);
        self
    }

    /// Add size hint for columns/rows `count` times
    pub fn sizes(mut self, size: Size, count: usize) -> Self {
        for _ in 0..count {
            self.sizing.add(size);
        }
        self
    }

    /// Build horizontal strip: Cells are positions from left to right.
    /// Takes the available horizontal width, so there can't be anything right of the strip or the container will grow slowly!
    ///
    /// Returns a `[egui::Response]` for hover events.
    pub fn horizontal<F>(self, strip: F) -> Response
    where
        F: for<'b> FnOnce(Strip<'a, 'b>),
    {
        let widths = self.sizing.into_lengths(
            self.ui.available_rect_before_wrap().width() - self.ui.spacing().item_spacing.x,
            self.ui.spacing().item_spacing.x,
        );
        let mut layout = Layout::new(self.ui, CellDirection::Horizontal);
        strip(Strip {
            layout: &mut layout,
            direction: CellDirection::Horizontal,
            sizes: widths,
        });
        layout.set_rect()
    }

    /// Build vertical strip: Cells are positions from top to bottom.
    /// Takes the full available vertical height, so there can't be anything below of the strip or the container will grow slowly!
    ///
    /// Returns a `[egui::Response]` for hover events.
    pub fn vertical<F>(self, strip: F) -> Response
    where
        F: for<'b> FnOnce(Strip<'a, 'b>),
    {
        let heights = self.sizing.into_lengths(
            self.ui.available_rect_before_wrap().height() - self.ui.spacing().item_spacing.y,
            self.ui.spacing().item_spacing.y,
        );
        let mut layout = Layout::new(self.ui, CellDirection::Vertical);
        strip(Strip {
            layout: &mut layout,
            direction: CellDirection::Vertical,
            sizes: heights,
        });
        layout.set_rect()
    }
}

/// A Strip of cells which go in one direction. Each cell has a fixed size.
/// In contrast to normal egui behavior, strip cells do *not* grow with its children!
pub struct Strip<'a, 'b> {
    layout: &'b mut Layout<'a>,
    direction: CellDirection,
    sizes: Vec<f32>,
}

impl<'a, 'b> Strip<'a, 'b> {
    fn next_cell_size(&mut self) -> (CellSize, CellSize) {
        match self.direction {
            CellDirection::Horizontal => (
                CellSize::Absolute(self.sizes.remove(0)),
                CellSize::Remainder,
            ),
            CellDirection::Vertical => (
                CellSize::Remainder,
                CellSize::Absolute(self.sizes.remove(0)),
            ),
        }
    }

    /// Add empty cell
    pub fn empty(&mut self) {
        assert!(
            !self.sizes.is_empty(),
            "Tried using more strip cells then available."
        );

        let (width, height) = self.next_cell_size();
        self.layout.empty(width, height);
    }

    fn _cell(&mut self, clip: bool, add_contents: impl FnOnce(&mut Ui)) {
        assert!(
            !self.sizes.is_empty(),
            "Tried using more strip cells then available."
        );

        let (width, height) = self.next_cell_size();
        self.layout.add(width, height, clip, add_contents);
    }

    /// Add cell, content is wrapped
    pub fn cell(&mut self, add_contents: impl FnOnce(&mut Ui)) {
        self._cell(false, add_contents);
    }

    /// Add cell, content is clipped
    pub fn cell_clip(&mut self, add_contents: impl FnOnce(&mut Ui)) {
        self._cell(true, add_contents);
    }

    fn _strip(&mut self, clip: bool, strip_builder: impl FnOnce(StripBuilder<'_>)) {
        self._cell(clip, |ui| {
            strip_builder(StripBuilder::new(ui));
        });
    }
    /// Add strip as cell
    pub fn strip(&mut self, strip_builder: impl FnOnce(StripBuilder<'_>)) {
        self._strip(false, strip_builder);
    }

    /// Add strip as cell, content is clipped
    pub fn strip_noclip(&mut self, strip_builder: impl FnOnce(StripBuilder<'_>)) {
        self._strip(true, strip_builder);
    }
}

impl<'a, 'b> Drop for Strip<'a, 'b> {
    fn drop(&mut self) {
        while !self.sizes.is_empty() {
            self.empty();
        }
    }
}