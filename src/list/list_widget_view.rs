//! This file is part of the xilem_extras project.
//! (c) 2026 by Jacek Wisniowski
//!
//! This project was released as open source under the
//! Apache License, Version 2.0: http://www.apache.org/licenses/LICENSE-2.0
//! (compatible with the Xilem licence).

//! High-performance virtualized list view for efficient rendering of large datasets.
//!
//! # Architecture Overview
//!
//! This module implements a virtualized list using Xilem's View/Widget pattern:
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │ ListView (Xilem View layer)                                     │
//! │   - Declarative API for building lists                          │
//! │   - Manages row view lifecycle (build/rebuild/teardown)         │
//! │   - Routes messages to child row views                          │
//! │   - Handles ListAction for user interactions                    │
//! └────────────────────────┬────────────────────────────────────────┘
//!                          │ Creates & manages
//!                          ▼
//! ┌─────────────────────────────────────────────────────────────────┐
//! │ ListWidget (Masonry Widget layer)                               │
//! │   - Internal scroll state management                            │
//! │   - Computes visible range with buffer zones                    │
//! │   - Submits ListRangeAction when range changes                  │
//! │   - Handles pointer events, scrollbar, keyboard navigation      │
//! └─────────────────────────────────────────────────────────────────┘
//! ```

use std::collections::HashMap;
use std::marker::PhantomData;

use xilem::core::{MessageCtx, MessageResult, Mut, View, ViewId, ViewMarker, ViewPathTracker};
use xilem::masonry::peniko::Color;
use xilem::{Pod, ViewCtx, WidgetView};

use super::widget::{ListRangeAction, ListWidget, ListWidgetAction};
use crate::traits::{Identifiable, SelectionModifiers, SelectionState};

/// Actions that can occur on list items.
#[derive(Debug, Clone, PartialEq)]
pub enum ListViewAction<Id> {
    /// Item selected with optional modifiers.
    Select(Id, SelectionModifiers),
    /// Item activated (double-click or Enter).
    Activate(Id),
}

/// Style configuration for the virtualized list.
#[derive(Debug, Clone)]
pub struct ListViewStyle {
    /// Row height in pixels.
    pub row_height: f64,
    /// Hover background color (used by row views).
    pub hover_bg: Color,
    /// Selected background color (used by row views).
    pub selected_bg: Color,
    /// Striped rows (alternating background).
    pub striped: bool,
    /// Stripe background color.
    pub stripe_bg: Color,
}

impl Default for ListViewStyle {
    fn default() -> Self {
        Self {
            row_height: 28.0,
            hover_bg: Color::from_rgba8(55, 53, 50, 255),
            selected_bg: Color::from_rgba8(65, 62, 58, 255),
            striped: false,
            stripe_bg: Color::from_rgba8(45, 43, 40, 255),
        }
    }
}

impl ListViewStyle {
    /// Creates a new style with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the row height.
    pub fn row_height(mut self, height: f64) -> Self {
        self.row_height = height;
        self
    }

    /// Sets the hover background color.
    pub fn hover_bg(mut self, color: Color) -> Self {
        self.hover_bg = color;
        self
    }

    /// Sets the selected background color.
    pub fn selected_bg(mut self, color: Color) -> Self {
        self.selected_bg = color;
        self
    }

    /// Enables striped rows.
    pub fn striped(mut self, striped: bool) -> Self {
        self.striped = striped;
        self
    }

    /// Sets the stripe background color.
    pub fn stripe_bg(mut self, color: Color) -> Self {
        self.stripe_bg = color;
        self
    }
}

/// Create the view id used for child row views.
const fn view_id_for_row(idx: usize) -> ViewId {
    ViewId::new(idx as u64)
}

/// Get the row index stored in the view id.
const fn row_index_for_view_id(id: ViewId) -> usize {
    id.routing_id() as usize
}

/// View state for each child row.
struct ChildState<View, ViewState> {
    view: View,
    state: ViewState,
}

/// Internal view state for ListView.
pub struct ListViewState<RowView, RowViewState> {
    /// Pending action from widget.
    pending_action: Option<ListRangeAction>,
    /// Per-row view states.
    children: HashMap<usize, ChildState<RowView, RowViewState>>,
}

/// High-performance virtualized list view.
pub struct ListView<State, R, RowView, F, H, Sel>
where
    R: Identifiable,
{
    phantom: PhantomData<fn() -> (State, RowView)>,
    /// Number of items in the list.
    item_count: usize,
    /// Style configuration.
    style: ListViewStyle,
    /// Function to build row view: (data_index, is_selected, is_striped) -> RowView.
    row_builder: F,
    /// Action handler.
    handler: H,
    /// Selection state for determining which rows are selected.
    selection_fn: Box<dyn Fn(usize) -> bool + Send + Sync>,
    /// ID getter for rows.
    id_getter: Box<dyn Fn(usize) -> R::Id + Send + Sync>,
    _sel: PhantomData<Sel>,
}

impl<State, R, RowView, F, H, Sel> ViewMarker for ListView<State, R, RowView, F, H, Sel> where
    R: Identifiable
{
}

impl<State, R, RowView, F, H, Sel> View<State, (), ViewCtx>
    for ListView<State, R, RowView, F, H, Sel>
where
    State: 'static,
    R: Identifiable + 'static,
    R::Id: Clone + Send + Sync + 'static,
    RowView: WidgetView<State, ()> + 'static,
    F: Fn(&mut State, usize, bool, bool) -> RowView + Send + Sync + 'static,
    H: Fn(&mut State, ListViewAction<R::Id>) + Clone + Send + Sync + 'static,
    Sel: SelectionState<R::Id> + 'static,
{
    type Element = Pod<ListWidget>;
    type ViewState = ListViewState<RowView, RowView::ViewState>;

    fn build(&self, ctx: &mut ViewCtx, _app_state: &mut State) -> (Self::Element, Self::ViewState) {
        // Create list widget
        let mut widget = ListWidget::new(self.style.row_height);
        widget.state_mut().set_item_count(self.item_count);

        let pod = Pod::new(widget);
        ctx.record_action_source(pod.new_widget.id());

        (
            pod,
            ListViewState {
                pending_action: None,
                children: HashMap::new(),
            },
        )
    }

    fn rebuild(
        &self,
        prev: &Self,
        view_state: &mut Self::ViewState,
        ctx: &mut ViewCtx,
        mut element: Mut<'_, Self::Element>,
        app_state: &mut State,
    ) {
        // Update item count if changed
        if self.item_count != prev.item_count {
            ListWidget::set_item_count(&mut element, self.item_count);
        }

        // Handle pending range action
        if let Some(pending_action) = view_state.pending_action.take() {
            ListWidget::will_handle_action(&mut element, &pending_action);

            // Teardown old rows not in target range
            for idx in pending_action.old_range.clone() {
                if !pending_action.target_range.contains(&idx) {
                    if let Some(mut child_state) = view_state.children.remove(&idx) {
                        ctx.with_id(view_id_for_row(idx), |ctx| {
                            if let Some(mut row_mut) = ListWidget::row_mut(&mut element, idx) {
                                child_state.view.teardown(&mut child_state.state, ctx, row_mut.downcast());
                            }
                            ListWidget::remove_row(&mut element, idx);
                        });
                    }
                }
            }

            // Build/rebuild rows in target range
            for idx in pending_action.target_range.clone() {
                let is_selected = (self.selection_fn)(idx);
                let is_striped = self.style.striped && idx % 2 == 1;

                if let Some(child) = view_state.children.get_mut(&idx) {
                    // Rebuild existing row
                    let next_view = (self.row_builder)(app_state, idx, is_selected, is_striped);
                    ctx.with_id(view_id_for_row(idx), |ctx| {
                        if let Some(mut row_mut) = ListWidget::row_mut(&mut element, idx) {
                            next_view.rebuild(
                                &child.view,
                                &mut child.state,
                                ctx,
                                row_mut.downcast(),
                                app_state,
                            );
                        }
                        child.view = next_view;
                    });
                } else {
                    // Build new row
                    let new_view = (self.row_builder)(app_state, idx, is_selected, is_striped);
                    ctx.with_id(view_id_for_row(idx), |ctx| {
                        let (new_element, child_state) = new_view.build(ctx, app_state);
                        ListWidget::add_row(&mut element, idx, new_element.new_widget.erased());
                        view_state.children.insert(
                            idx,
                            ChildState {
                                view: new_view,
                                state: child_state,
                            },
                        );
                    });
                }
            }

            ListWidget::did_handle_action(&mut element);
        } else {
            // No action, just rebuild existing rows
            for (&idx, child) in &mut view_state.children {
                let is_selected = (self.selection_fn)(idx);
                let is_striped = self.style.striped && idx % 2 == 1;
                let next_view = (self.row_builder)(app_state, idx, is_selected, is_striped);
                ctx.with_id(view_id_for_row(idx), |ctx| {
                    if let Some(mut row_mut) = ListWidget::row_mut(&mut element, idx) {
                        next_view.rebuild(
                            &child.view,
                            &mut child.state,
                            ctx,
                            row_mut.downcast(),
                            app_state,
                        );
                    }
                    child.view = next_view;
                });
            }
        }
    }

    fn teardown(
        &self,
        view_state: &mut Self::ViewState,
        ctx: &mut ViewCtx,
        mut element: Mut<'_, Self::Element>,
    ) {
        for (&idx, child) in &mut view_state.children {
            ctx.with_id(view_id_for_row(idx), |ctx| {
                if let Some(mut row_mut) = ListWidget::row_mut(&mut element, idx) {
                    child.view.teardown(&mut child.state, ctx, row_mut.downcast());
                }
            });
        }
        ctx.teardown_action_source(element);
    }

    fn message(
        &self,
        view_state: &mut Self::ViewState,
        message: &mut MessageCtx,
        mut element: Mut<'_, Self::Element>,
        app_state: &mut State,
    ) -> MessageResult<()> {
        // Check for child message routing
        if let Some(first) = message.take_first() {
            let child_idx = row_index_for_view_id(first);
            if let Some(target) = view_state.children.get_mut(&child_idx) {
                if let Some(mut row_mut) = ListWidget::row_mut(&mut element, child_idx) {
                    return target.view.message(
                        &mut target.state,
                        message,
                        row_mut.downcast(),
                        app_state,
                    );
                }
            }
            tracing::error!(
                "Message sent to unloaded view in `ListView::message`: {message:?}"
            );
            return MessageResult::Stale;
        }

        // Handle ListWidgetAction
        if let Some(action) = message.take_message::<ListWidgetAction>() {
            match *action {
                ListWidgetAction::RangeChanged(range_action) => {
                    view_state.pending_action = Some(range_action);
                    return MessageResult::RequestRebuild;
                }
                ListWidgetAction::RowSelect(ref click) => {
                    let id = (self.id_getter)(click.row_index);
                    let mods = SelectionModifiers {
                        shift: click.shift,
                        command: click.command,
                        alt: false,
                    };
                    (self.handler)(app_state, ListViewAction::Select(id, mods));
                    return MessageResult::Action(());
                }
                ListWidgetAction::RowActivate(ref click) => {
                    let id = (self.id_getter)(click.row_index);
                    (self.handler)(app_state, ListViewAction::Activate(id));
                    return MessageResult::Action(());
                }
            }
        }

        tracing::error!(?message, "Wrong message type in ListView::message");
        MessageResult::Stale
    }
}

/// Creates a high-performance virtualized list view for large datasets.
///
/// Only renders visible rows plus a buffer zone, making it efficient
/// for lists with thousands of rows.
///
/// # Arguments
///
/// * `data` - The collection of items (must implement `Identifiable`)
/// * `selection` - Selection state
/// * `row_builder` - Function that builds a view for each row: `(state, index, is_selected, is_striped) -> RowView`
/// * `handler` - Function that handles list actions
///
/// # Example
///
/// ```ignore
/// use xilem_extras::{list_view, ListViewAction, ListViewStyle};
///
/// list_view(
///     &model.contacts,
///     &model.selection,
///     |state, idx, is_selected, is_striped| {
///         let contact = &state.contacts[idx];
///         contact_row(contact, is_selected, is_striped)
///     },
///     |state, action| {
///         match action {
///             ListViewAction::Select(id, mods) => {
///                 state.selection.select(id, mods);
///             }
///             ListViewAction::Activate(id) => {
///                 // Handle double-click/Enter
///             }
///         }
///     },
/// )
/// ```
pub fn list_view<'a, State, R, RowView, Sel, F, H>(
    data: &'a [R],
    selection: &'a Sel,
    row_builder: F,
    handler: H,
) -> impl WidgetView<State, ()> + use<'a, State, R, RowView, Sel, F, H>
where
    State: 'static,
    R: Identifiable + Clone + 'static,
    R::Id: Clone + Send + Sync + 'static,
    RowView: WidgetView<State, ()> + 'static,
    Sel: SelectionState<R::Id> + Clone + Send + Sync + 'static,
    F: Fn(&mut State, usize, bool, bool) -> RowView + Send + Sync + 'static,
    H: Fn(&mut State, ListViewAction<R::Id>) + Clone + Send + Sync + 'static,
{
    list_view_styled(data, selection, ListViewStyle::default(), row_builder, handler)
}

/// Creates a high-performance virtualized list view with custom styling.
///
/// Same as [`list_view`] but accepts a [`ListViewStyle`] for customization.
pub fn list_view_styled<'a, State, R, RowView, Sel, F, H>(
    data: &'a [R],
    selection: &'a Sel,
    style: ListViewStyle,
    row_builder: F,
    handler: H,
) -> impl WidgetView<State, ()> + use<'a, State, R, RowView, Sel, F, H>
where
    State: 'static,
    R: Identifiable + Clone + 'static,
    R::Id: Clone + Send + Sync + 'static,
    RowView: WidgetView<State, ()> + 'static,
    Sel: SelectionState<R::Id> + Clone + Send + Sync + 'static,
    F: Fn(&mut State, usize, bool, bool) -> RowView + Send + Sync + 'static,
    H: Fn(&mut State, ListViewAction<R::Id>) + Clone + Send + Sync + 'static,
{
    let data_len = data.len();
    let data_for_id: Vec<R::Id> = data.iter().map(|r| r.id()).collect();
    let data_for_sel: Vec<R::Id> = data.iter().map(|r| r.id()).collect();

    let selection_clone = selection.clone();
    let selection_fn = Box::new(move |idx: usize| {
        if idx < data_for_sel.len() {
            selection_clone.is_selected(&data_for_sel[idx])
        } else {
            false
        }
    });

    let id_getter = Box::new(move |idx: usize| data_for_id[idx].clone());

    ListView::<State, R, RowView, F, H, Sel> {
        phantom: PhantomData,
        item_count: data_len,
        style,
        row_builder,
        handler,
        selection_fn,
        id_getter,
        _sel: PhantomData,
    }
}

// =============================================================================
// Simple list_navigable (legacy API for backwards compatibility)
// =============================================================================

/// Creates a widget-based list view with keyboard navigation.
///
/// This is a simplified list that handles keyboard navigation but requires
/// an external row builder for rendering. For full row lifecycle management,
/// use [`list_view`] instead.
///
/// # Arguments
///
/// * `item_count` - Number of items in the list
/// * `style` - Style configuration
/// * `on_action` - Callback for selection/activation events
pub fn list_navigable<State, Action, F>(
    item_count: usize,
    style: ListViewStyle,
    on_action: F,
) -> ListNavigableView<State, Action, F>
where
    F: Fn(&mut State, ListViewAction<usize>) + Send + Sync + 'static,
{
    ListNavigableView {
        item_count,
        style,
        on_action,
        _phantom: PhantomData,
    }
}

/// A navigable list view using the ListWidget (simple API).
pub struct ListNavigableView<State, Action, F> {
    item_count: usize,
    style: ListViewStyle,
    on_action: F,
    _phantom: PhantomData<fn(&mut State) -> Action>,
}

/// View state for ListNavigableView.
pub struct ListNavigableViewState {
    item_count: usize,
}

impl<State, Action, F> ViewMarker for ListNavigableView<State, Action, F> {}

impl<State, Action, F> View<State, Action, ViewCtx> for ListNavigableView<State, Action, F>
where
    State: 'static,
    Action: 'static,
    F: Fn(&mut State, ListViewAction<usize>) + Send + Sync + 'static,
{
    type Element = Pod<ListWidget>;
    type ViewState = ListNavigableViewState;

    fn build(
        &self,
        ctx: &mut ViewCtx,
        _app_state: &mut State,
    ) -> (Self::Element, Self::ViewState) {
        let mut widget = ListWidget::new(self.style.row_height);
        widget.state_mut().set_item_count(self.item_count);

        let pod = ctx.with_action_widget(|ctx| ctx.create_pod(widget));

        let state = ListNavigableViewState {
            item_count: self.item_count,
        };

        (pod, state)
    }

    fn rebuild(
        &self,
        _prev: &Self,
        view_state: &mut Self::ViewState,
        _ctx: &mut ViewCtx,
        mut element: Mut<'_, Self::Element>,
        _app_state: &mut State,
    ) {
        // Update item count if changed
        if self.item_count != view_state.item_count {
            ListWidget::set_item_count(&mut element, self.item_count);
            view_state.item_count = self.item_count;
        }
    }

    fn teardown(
        &self,
        _view_state: &mut Self::ViewState,
        ctx: &mut ViewCtx,
        element: Mut<'_, Self::Element>,
    ) {
        ctx.teardown_action_source(element);
    }

    fn message(
        &self,
        _view_state: &mut Self::ViewState,
        message: &mut MessageCtx,
        _element: Mut<'_, Self::Element>,
        app_state: &mut State,
    ) -> MessageResult<Action> {
        if let Some(action) = message.take_message::<ListWidgetAction>() {
            match *action {
                ListWidgetAction::RowSelect(ref row_action) => {
                    let mods = SelectionModifiers {
                        shift: row_action.shift,
                        command: row_action.command,
                        alt: false,
                    };
                    (self.on_action)(app_state, ListViewAction::Select(row_action.row_index, mods));
                    return MessageResult::RequestRebuild;
                }
                ListWidgetAction::RowActivate(ref row_action) => {
                    (self.on_action)(app_state, ListViewAction::Activate(row_action.row_index));
                    return MessageResult::RequestRebuild;
                }
                ListWidgetAction::RangeChanged(_) => {
                    // Range changed but this simple API doesn't manage rows
                    return MessageResult::RequestRebuild;
                }
            }
        }
        MessageResult::Stale
    }
}

// =============================================================================
// Sectioned List View
// =============================================================================

/// Definition of a section in a sectioned list.
#[derive(Debug, Clone)]
pub struct SectionDef<'a, R> {
    /// Section title.
    pub title: String,
    /// Items in this section.
    pub items: &'a [R],
}

impl<'a, R> SectionDef<'a, R> {
    /// Creates a new section definition.
    pub fn new(title: impl Into<String>, items: &'a [R]) -> Self {
        Self {
            title: title.into(),
            items,
        }
    }
}

/// Information about a row in a sectioned list.
#[derive(Debug, Clone)]
pub enum SectionedRowInfo {
    /// This row is a section header.
    Header {
        /// Section index.
        section_index: usize,
        /// Section title.
        title: String,
    },
    /// This row is an item.
    Item {
        /// Section index this item belongs to.
        section_index: usize,
        /// Index within the section.
        item_index_in_section: usize,
        /// Global item index (across all sections).
        global_item_index: usize,
        /// Whether this item is selected.
        is_selected: bool,
        /// Whether this row should have striped background.
        is_striped: bool,
    },
}

/// Flat index mapping for sectioned lists.
struct FlatIndexMap {
    /// For each flat index: (section_index, is_header, item_index_in_section)
    entries: Vec<(usize, bool, usize)>,
}

impl FlatIndexMap {
    fn new<R>(sections: &[SectionDef<'_, R>]) -> Self {
        let mut entries = Vec::new();

        for (section_idx, section) in sections.iter().enumerate() {
            // Add header entry
            entries.push((section_idx, true, 0));

            // Add item entries
            for item_idx in 0..section.items.len() {
                entries.push((section_idx, false, item_idx));
            }
        }

        Self { entries }
    }

    fn total_rows(&self) -> usize {
        self.entries.len()
    }

    fn get(&self, flat_idx: usize) -> Option<&(usize, bool, usize)> {
        self.entries.get(flat_idx)
    }

    /// Convert flat index to global item index (skipping headers).
    fn flat_to_global_item(&self, flat_idx: usize) -> Option<usize> {
        let mut global_idx = 0;
        for (i, &(_, is_header, _)) in self.entries.iter().enumerate() {
            if i == flat_idx {
                return if is_header { None } else { Some(global_idx) };
            }
            if !is_header {
                global_idx += 1;
            }
        }
        None
    }
}

/// Internal view state for SectionedListView.
pub struct SectionedListViewState<RowView, RowViewState> {
    /// Pending action from widget.
    pending_action: Option<ListRangeAction>,
    /// Per-row view states.
    children: HashMap<usize, ChildState<RowView, RowViewState>>,
}

/// High-performance virtualized sectioned list view.
pub struct SectionedListView<State, R, RowView, F, H, Sel>
where
    R: Identifiable,
{
    phantom: PhantomData<fn() -> (State, RowView)>,
    /// Flat index map.
    flat_map: FlatIndexMap,
    /// Section titles.
    section_titles: Vec<String>,
    /// Global item IDs for selection.
    item_ids: Vec<R::Id>,
    /// Style configuration.
    style: ListViewStyle,
    /// Function to build row view: (state, SectionedRowInfo) -> RowView.
    row_builder: F,
    /// Action handler.
    handler: H,
    /// Selection state for determining which items are selected.
    selection_fn: Box<dyn Fn(usize) -> bool + Send + Sync>,
    _sel: PhantomData<Sel>,
}

impl<State, R, RowView, F, H, Sel> ViewMarker for SectionedListView<State, R, RowView, F, H, Sel> where
    R: Identifiable
{
}

impl<State, R, RowView, F, H, Sel> View<State, (), ViewCtx>
    for SectionedListView<State, R, RowView, F, H, Sel>
where
    State: 'static,
    R: Identifiable + 'static,
    R::Id: Clone + Send + Sync + 'static,
    RowView: WidgetView<State, ()> + 'static,
    F: Fn(&mut State, SectionedRowInfo) -> RowView + Send + Sync + 'static,
    H: Fn(&mut State, ListViewAction<R::Id>) + Clone + Send + Sync + 'static,
    Sel: SelectionState<R::Id> + 'static,
{
    type Element = Pod<ListWidget>;
    type ViewState = SectionedListViewState<RowView, RowView::ViewState>;

    fn build(&self, ctx: &mut ViewCtx, _app_state: &mut State) -> (Self::Element, Self::ViewState) {
        let mut widget = ListWidget::new(self.style.row_height);
        widget.state_mut().set_item_count(self.flat_map.total_rows());

        let pod = Pod::new(widget);
        ctx.record_action_source(pod.new_widget.id());

        (
            pod,
            SectionedListViewState {
                pending_action: None,
                children: HashMap::new(),
            },
        )
    }

    fn rebuild(
        &self,
        prev: &Self,
        view_state: &mut Self::ViewState,
        ctx: &mut ViewCtx,
        mut element: Mut<'_, Self::Element>,
        app_state: &mut State,
    ) {
        // Update item count if changed
        if self.flat_map.total_rows() != prev.flat_map.total_rows() {
            ListWidget::set_item_count(&mut element, self.flat_map.total_rows());
        }

        // Handle pending range action
        if let Some(pending_action) = view_state.pending_action.take() {
            ListWidget::will_handle_action(&mut element, &pending_action);

            // Teardown old rows not in target range
            for idx in pending_action.old_range.clone() {
                if !pending_action.target_range.contains(&idx) {
                    if let Some(mut child_state) = view_state.children.remove(&idx) {
                        ctx.with_id(view_id_for_row(idx), |ctx| {
                            if let Some(mut row_mut) = ListWidget::row_mut(&mut element, idx) {
                                child_state.view.teardown(&mut child_state.state, ctx, row_mut.downcast());
                            }
                            ListWidget::remove_row(&mut element, idx);
                        });
                    }
                }
            }

            // Build/rebuild rows in target range
            for flat_idx in pending_action.target_range.clone() {
                let row_info = self.make_row_info(flat_idx);

                if let Some(child) = view_state.children.get_mut(&flat_idx) {
                    // Rebuild existing row
                    let next_view = (self.row_builder)(app_state, row_info);
                    ctx.with_id(view_id_for_row(flat_idx), |ctx| {
                        if let Some(mut row_mut) = ListWidget::row_mut(&mut element, flat_idx) {
                            next_view.rebuild(
                                &child.view,
                                &mut child.state,
                                ctx,
                                row_mut.downcast(),
                                app_state,
                            );
                        }
                        child.view = next_view;
                    });
                } else {
                    // Build new row
                    let new_view = (self.row_builder)(app_state, row_info);
                    ctx.with_id(view_id_for_row(flat_idx), |ctx| {
                        let (new_element, child_state) = new_view.build(ctx, app_state);
                        ListWidget::add_row(&mut element, flat_idx, new_element.new_widget.erased());
                        view_state.children.insert(
                            flat_idx,
                            ChildState {
                                view: new_view,
                                state: child_state,
                            },
                        );
                    });
                }
            }

            ListWidget::did_handle_action(&mut element);
        } else {
            // No action, just rebuild existing rows
            for (&flat_idx, child) in &mut view_state.children {
                let row_info = self.make_row_info(flat_idx);
                let next_view = (self.row_builder)(app_state, row_info);
                ctx.with_id(view_id_for_row(flat_idx), |ctx| {
                    if let Some(mut row_mut) = ListWidget::row_mut(&mut element, flat_idx) {
                        next_view.rebuild(
                            &child.view,
                            &mut child.state,
                            ctx,
                            row_mut.downcast(),
                            app_state,
                        );
                    }
                    child.view = next_view;
                });
            }
        }
    }

    fn teardown(
        &self,
        view_state: &mut Self::ViewState,
        ctx: &mut ViewCtx,
        mut element: Mut<'_, Self::Element>,
    ) {
        for (&idx, child) in &mut view_state.children {
            ctx.with_id(view_id_for_row(idx), |ctx| {
                if let Some(mut row_mut) = ListWidget::row_mut(&mut element, idx) {
                    child.view.teardown(&mut child.state, ctx, row_mut.downcast());
                }
            });
        }
        ctx.teardown_action_source(element);
    }

    fn message(
        &self,
        view_state: &mut Self::ViewState,
        message: &mut MessageCtx,
        mut element: Mut<'_, Self::Element>,
        app_state: &mut State,
    ) -> MessageResult<()> {
        // Check for child message routing
        if let Some(first) = message.take_first() {
            let child_idx = row_index_for_view_id(first);
            if let Some(target) = view_state.children.get_mut(&child_idx) {
                if let Some(mut row_mut) = ListWidget::row_mut(&mut element, child_idx) {
                    return target.view.message(
                        &mut target.state,
                        message,
                        row_mut.downcast(),
                        app_state,
                    );
                }
            }
            tracing::error!(
                "Message sent to unloaded view in `SectionedListView::message`: {message:?}"
            );
            return MessageResult::Stale;
        }

        // Handle ListWidgetAction
        if let Some(action) = message.take_message::<ListWidgetAction>() {
            match *action {
                ListWidgetAction::RangeChanged(range_action) => {
                    view_state.pending_action = Some(range_action);
                    return MessageResult::RequestRebuild;
                }
                ListWidgetAction::RowSelect(ref click) => {
                    // Only handle item clicks, not header clicks
                    if let Some(global_idx) = self.flat_map.flat_to_global_item(click.row_index) {
                        if global_idx < self.item_ids.len() {
                            let id = self.item_ids[global_idx].clone();
                            let mods = SelectionModifiers {
                                shift: click.shift,
                                command: click.command,
                                alt: false,
                            };
                            (self.handler)(app_state, ListViewAction::Select(id, mods));
                            return MessageResult::Action(());
                        }
                    }
                    return MessageResult::Nop;
                }
                ListWidgetAction::RowActivate(ref click) => {
                    // Only handle item activations, not header activations
                    if let Some(global_idx) = self.flat_map.flat_to_global_item(click.row_index) {
                        if global_idx < self.item_ids.len() {
                            let id = self.item_ids[global_idx].clone();
                            (self.handler)(app_state, ListViewAction::Activate(id));
                            return MessageResult::Action(());
                        }
                    }
                    return MessageResult::Nop;
                }
            }
        }

        tracing::error!(?message, "Wrong message type in SectionedListView::message");
        MessageResult::Stale
    }
}

impl<State, R, RowView, F, H, Sel> SectionedListView<State, R, RowView, F, H, Sel>
where
    R: Identifiable,
{
    fn make_row_info(&self, flat_idx: usize) -> SectionedRowInfo {
        if let Some(&(section_idx, is_header, item_idx_in_section)) = self.flat_map.get(flat_idx) {
            if is_header {
                SectionedRowInfo::Header {
                    section_index: section_idx,
                    title: self.section_titles.get(section_idx)
                        .cloned()
                        .unwrap_or_default(),
                }
            } else {
                let global_item_idx = self.flat_map.flat_to_global_item(flat_idx).unwrap_or(0);
                let is_selected = (self.selection_fn)(global_item_idx);
                let is_striped = self.style.striped && flat_idx % 2 == 1;

                SectionedRowInfo::Item {
                    section_index: section_idx,
                    item_index_in_section: item_idx_in_section,
                    global_item_index: global_item_idx,
                    is_selected,
                    is_striped,
                }
            }
        } else {
            // Fallback - shouldn't happen
            SectionedRowInfo::Header {
                section_index: 0,
                title: String::new(),
            }
        }
    }
}

/// Creates a high-performance virtualized sectioned list view.
///
/// The list displays items grouped into sections with headers.
///
/// # Arguments
///
/// * `sections` - Slice of section definitions
/// * `selection` - Selection state
/// * `style` - Style configuration
/// * `row_builder` - Function that builds a view for each row: `(state, SectionedRowInfo) -> RowView`
/// * `handler` - Function that handles list actions
///
/// # Example
///
/// ```ignore
/// use xilem_extras::{list_view_sectioned, ListViewAction, ListViewStyle, SectionDef, SectionedRowInfo};
///
/// let sections = [
///     SectionDef::new("Favorites", &model.favorites),
///     SectionDef::new("Recent", &model.recent),
///     SectionDef::new("All Contacts", &model.contacts),
/// ];
///
/// list_view_sectioned(
///     &sections,
///     &model.selection,
///     ListViewStyle::new().row_height(32.0),
///     |state, row_info| {
///         match row_info {
///             SectionedRowInfo::Header { title, .. } => {
///                 section_header(title)
///             }
///             SectionedRowInfo::Item { global_item_index, is_selected, .. } => {
///                 contact_row(&state.all_items[global_item_index], is_selected)
///             }
///         }
///     },
///     |state, action| {
///         match action {
///             ListViewAction::Select(id, mods) => state.selection.select(id, mods),
///             ListViewAction::Activate(id) => { /* handle */ }
///         }
///     },
/// )
/// ```
pub fn list_view_sectioned<'a, State, R, RowView, Sel, F, H>(
    sections: &[SectionDef<'a, R>],
    selection: &'a Sel,
    style: ListViewStyle,
    row_builder: F,
    handler: H,
) -> impl WidgetView<State, ()> + use<'a, State, R, RowView, Sel, F, H>
where
    State: 'static,
    R: Identifiable + Clone + 'static,
    R::Id: Clone + Send + Sync + 'static,
    RowView: WidgetView<State, ()> + 'static,
    Sel: SelectionState<R::Id> + Clone + Send + Sync + 'static,
    F: Fn(&mut State, SectionedRowInfo) -> RowView + Send + Sync + 'static,
    H: Fn(&mut State, ListViewAction<R::Id>) + Clone + Send + Sync + 'static,
{
    // Build flat index map
    let flat_map = FlatIndexMap::new(sections);

    // Collect section titles
    let section_titles: Vec<String> = sections.iter().map(|s| s.title.clone()).collect();

    // Collect all item IDs
    let item_ids: Vec<R::Id> = sections
        .iter()
        .flat_map(|s| s.items.iter().map(|r| r.id()))
        .collect();

    // Clone for selection closure
    let item_ids_for_sel = item_ids.clone();
    let selection_clone = selection.clone();
    let selection_fn = Box::new(move |global_idx: usize| {
        if global_idx < item_ids_for_sel.len() {
            selection_clone.is_selected(&item_ids_for_sel[global_idx])
        } else {
            false
        }
    });

    SectionedListView::<State, R, RowView, F, H, Sel> {
        phantom: PhantomData,
        flat_map,
        section_titles,
        item_ids,
        style,
        row_builder,
        handler,
        selection_fn,
        _sel: PhantomData,
    }
}
