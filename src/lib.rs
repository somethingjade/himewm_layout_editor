use enums::{Align, Color, FrameType};
use fltk::{group::FlexType, prelude::InputExt, *};
use fltk_theme::*;
use group::{PackType, ScrollType};
use himewm_layout::*;
use prelude::{GroupExt, WidgetBase, WidgetExt};

#[derive(Clone)]
enum SwapDirection {
    Previous,
    Next,
}

#[derive(Clone)]
enum Message {
    SelectedVariantChanged(usize),
    SelectedVariantStateChanged(usize),
    SelectedZoneChanged(usize),
    NewVariantState,
    CloneVariantState,
    DeleteVariantState,
    SwapVariantState(SwapDirection),
    SwapSplitDirection,
    Split,
    Swap,
    Merge,
    NewVariant,
    CloneVariant,
    DeleteVariant,
    SwapVariant(SwapDirection),
    SetVariantAsDefault,
}

struct LayoutEditor {
    layout: Layout,
    selected_variant_idx: usize,
    selected_variant_state_idx: usize,
    selected_zone_idx1: Option<usize>,
    selected_zone_idx2: Option<usize>,
}

impl LayoutEditor {
    fn new(layout: Layout) -> Self {
        let default_variant_idx = layout.default_variant_idx();

        return LayoutEditor {
            layout,
            selected_variant_idx: default_variant_idx,
            selected_variant_state_idx: 0,
            selected_zone_idx1: None,
            selected_zone_idx2: None,
        };
    }
}

struct Actions {
    widgets: group::Flex,
    split_bound_max: Option<i32>,
    selected_direction: Direction,
    split_button: button::Button,
    split_axis_text: frame::Frame,
    split_at_input: input::IntInput,
    split_bounds_text: frame::Frame,
    swap_button: button::Button,
    merge_button: button::Button,
}

impl Actions {
    fn initialize(sender: &app::Sender<Message>) -> Self {
        let selected_direction = Direction::Horizontal;

        let mut widgets = group::Flex::default_fill().row();

        let widgets_w = widgets.w() / 4;

        let widgets_h = widgets.h() / 4;

        WidgetExt::set_size(&mut widgets, widgets_w, widgets_h);

        let mut split_actions_column = group::Flex::default().column();

        let mut horizontal_radio_button =
            button::RadioRoundButton::default().with_label("Horizontal");

        let mut vertical_radio_button = button::RadioRoundButton::default().with_label("Vertical");

        horizontal_radio_button.toggle(true);

        horizontal_radio_button.emit(sender.clone(), Message::SwapSplitDirection);

        vertical_radio_button.emit(sender.clone(), Message::SwapSplitDirection);

        let mut split_at_selection = group::Flex::default().row();

        let split_at_selection_w = split_at_selection.w();

        WidgetExt::set_size(&mut split_at_selection, split_at_selection_w, 32);

        let split_axis_text = frame::Frame::default()
            .with_align(Align::Left.union(Align::Inside))
            .with_label("x: ");

        split_at_selection.fixed(&split_axis_text, 16);

        let split_at_input = input::IntInput::default();

        split_at_selection.end();

        let split_bounds_text = frame::Frame::default();

        let mut split_button = button::Button::default().with_label("Split");

        split_button.emit(sender.clone(), Message::Split);

        split_button.deactivate();

        split_actions_column.fixed(&split_button, 32);

        split_actions_column.end();

        let columns_spacer = frame::Frame::default();

        widgets.fixed(&columns_spacer, 4);

        let mut merge_and_swap_column = group::Flex::default().column();

        let mut swap_button = button::Button::default().with_label("Swap");

        swap_button.emit(sender.clone(), Message::Swap);

        swap_button.deactivate();

        merge_and_swap_column.fixed(&swap_button, 32);

        let mut merge_button = button::Button::default().with_label("Merge");

        merge_button.emit(sender.clone(), Message::Merge);

        merge_button.deactivate();

        merge_and_swap_column.fixed(&merge_button, 32);

        merge_and_swap_column.end();

        widgets.end();

        return Actions {
            widgets,
            selected_direction,
            split_bound_max: None,
            split_button,
            split_axis_text,
            split_at_input,
            split_bounds_text,
            merge_button,
            swap_button,
        };
    }
}

struct VariantActions {
    widgets: group::Flex,
    delete_button: button::Button,
    set_as_default_button: button::Button,
}

impl VariantActions {
    fn initialize(sender: &app::Sender<Message>) -> Self {
        let mut widgets = group::Flex::default_fill().column();

        let w = widgets.w() / 8;

        WidgetExt::set_size(&mut widgets, w, 144);

        let create_row = group::Flex::default()
            .with_size(0, 32)
            .with_type(FlexType::Row);

        let mut new_button = button::Button::default().with_label("New");

        let mut clone_button = button::Button::default().with_label("Clone");

        new_button.emit(sender.clone(), Message::NewVariant);

        clone_button.emit(sender.clone(), Message::CloneVariant);

        create_row.end();

        widgets.fixed(&create_row, 32);

        let mut delete_button = button::Button::default().with_label("Delete");

        delete_button.emit(sender.clone(), Message::DeleteVariant);

        widgets.fixed(&delete_button, 32);

        let mut set_as_default_button = button::Button::default().with_label("Set as default");

        set_as_default_button.emit(sender.clone(), Message::SetVariantAsDefault);

        widgets.fixed(&set_as_default_button, 32);

        let swap_row = group::Flex::default().row();

        let mut up_button = button::Button::default().with_label("@8>");

        let mut down_button = button::Button::default().with_label("@2>");

        up_button.emit(
            sender.clone(),
            Message::SwapVariant(SwapDirection::Previous),
        );

        down_button.emit(sender.clone(), Message::SwapVariant(SwapDirection::Next));

        swap_row.end();

        widgets.fixed(&swap_row, 32);

        widgets.end();

        return VariantActions {
            widgets,
            delete_button,
            set_as_default_button,
        };
    }
}

struct RepeatingSplitActions {}

impl RepeatingSplitActions {
    fn initialize() -> Self {
        return RepeatingSplitActions {};
    }
}

struct EndBehaviourActions {
    widgets: group::Flex,
    direction_select: group::Flex,
    zone_idx_choice: menu::Choice,
    repeating_split_actions: RepeatingSplitActions,
}

impl EndBehaviourActions {
    fn initialize(sender: &app::Sender<Message>) -> Self {
        let mut widgets = group::Flex::default_fill().row();

        let w = widgets.w() / 3;

        let h = widgets.h() / 2;

        WidgetExt::set_size(&mut widgets, w, h);

        let behaviour_column = group::Flex::default().column();

        let _directional_radio_button =
            button::RadioRoundButton::default().with_label("Directional");

        let mut direction_select_indent = group::Flex::default().row();

        let indent_spacer = frame::Frame::default();

        direction_select_indent.fixed(&indent_spacer, 16);

        let direction_select = group::Flex::default().column();

        let _horizontal_radio_button = button::RadioRoundButton::default().with_label("Horizontal");

        let _vertical_radio_button = button::RadioRoundButton::default().with_label("Vertical");

        direction_select.end();

        direction_select_indent.end();

        let _repeating_radio_button = button::RadioRoundButton::default().with_label("Repeating");

        let mut zone_idx_flex = group::Flex::default().row();

        let zone_idx_text = frame::Frame::default()
            .with_align(Align::Left.union(Align::Inside))
            .with_label("Zone index: ");

        zone_idx_flex.fixed(&zone_idx_text, 96);

        let zone_idx_choice = menu::Choice::default();

        zone_idx_flex.end();

        let _preview_button = button::Button::default().with_label("Preview");

        behaviour_column.end();

        widgets.end();

        return EndBehaviourActions {
            widgets,
            direction_select,
            zone_idx_choice,
            repeating_split_actions: RepeatingSplitActions::initialize(),
        };
    }
}

struct EditorWidgets {
    editor: LayoutEditor,
    variant_list: group::Scroll,
    variant_state_selection: group::Scroll,
    variant_state_pack: group::Pack,
    variant_state_display: group::Group,
    actions: Actions,
    variant_actions: VariantActions,
    end_behaviour_actions: EndBehaviourActions,
}

impl EditorWidgets {
    fn initialize(layout: Layout, sender: &app::Sender<Message>) -> Self {
        let variant_list = Self::create_variant_list(&layout, sender);

        let variant_state_selection = Self::create_variant_state_selection(&layout, sender);

        let variant_state_buttons =
            Self::create_variant_state_buttons(sender).below_of(&variant_state_selection, 4);

        let variant_state_pack_h = variant_state_selection.h() + variant_state_buttons.h();

        let mut variant_state_pack =
            group::Pack::default().with_size(variant_state_selection.w(), variant_state_pack_h);

        variant_state_pack.end();

        variant_state_pack.set_spacing(4);

        variant_state_pack.resize_callback(move |p, _, _, _, h| {
            if h != variant_state_pack_h {
                p.widget_resize(p.x(), p.y(), p.w(), variant_state_pack_h);
            }
        });

        variant_state_pack.add(&variant_state_selection);

        variant_state_pack.add(&variant_state_buttons);

        let variant_state_display = Self::create_variant_state_display(&layout, sender);

        let actions = Actions::initialize(sender);

        let variant_actions = VariantActions::initialize(sender);

        let end_behaviour_actions = EndBehaviourActions::initialize(sender);

        let editor = LayoutEditor::new(layout);

        let mut ret = EditorWidgets {
            editor,
            variant_list,
            variant_state_selection,
            variant_state_pack,
            variant_state_display,
            actions,
            variant_actions,
            end_behaviour_actions,
        };

        ret.update_highlighted_variant(
            ret.editor.selected_variant_idx,
            ret.editor.selected_variant_idx,
        );

        ret.update_shown_variant_state_selection(
            ret.editor.selected_variant_idx,
            ret.editor.selected_variant_idx,
        );

        ret.update_highlighted_variant_state_button(
            (
                ret.editor.selected_variant_idx,
                ret.editor.selected_variant_state_idx,
            ),
            (
                ret.editor.selected_variant_idx,
                ret.editor.selected_variant_state_idx,
            ),
        );

        ret.update_shown_variant_state(
            (
                ret.editor.selected_variant_idx,
                ret.editor.selected_variant_state_idx,
            ),
            (
                ret.editor.selected_variant_idx,
                ret.editor.selected_variant_state_idx,
            ),
        );

        if ret.editor.layout.variants_len() == 1 {
            ret.variant_actions.delete_button.deactivate();
        }

        if ret.editor.selected_variant_idx == ret.editor.layout.default_variant_idx() {
            ret.variant_actions.set_as_default_button.deactivate();
        }

        return ret;
    }

    fn display_group_from_variant_state(
        variant_width: f64,
        variant_height: f64,
        variant: &Variant,
        idx: usize,
        sender: &app::Sender<Message>,
    ) -> group::Group {
        let group = group::Group::default_fill();

        let w = group.w();

        let h = group.h();

        let x_offset = group.x();

        let y_offset = group.y();

        let zones = &variant.get_zones()[idx];

        for (i, zone) in zones.iter().enumerate() {
            let mut b = button::Button::new(
                ((zone.left as f64 * w as f64) / variant_width).round() as i32 + x_offset,
                ((zone.top as f64 * h as f64) / variant_height).round() as i32 + y_offset,
                ((zone.w() as f64 * w as f64) / variant_width).round() as i32,
                ((zone.h() as f64 * h as f64) / variant_height).round() as i32,
                Some((i + 1).to_string().as_str()),
            );

            // TODO: this frame type doesn't look too great - probably
            // figure out how to make it look better
            b.set_frame(FrameType::EmbossedBox);

            b.set_label_size(36);

            b.set_label_color(Color::Black);

            b.set_color(colors::html::Gainsboro);

            b.set_selection_color(colors::html::DodgerBlue);

            b.emit(sender.clone(), Message::SelectedZoneChanged(i));
        }

        group.end();

        return group;
    }

    fn create_variant_list(layout: &Layout, sender: &app::Sender<Message>) -> group::Scroll {
        let mut scroll = group::Scroll::default_fill().with_type(ScrollType::Vertical);

        scroll.set_size(scroll.w() / 8, scroll.h() / 2);

        scroll.set_color(Color::Background2);

        scroll.resize_callback(|s, _, _, w, _| {
            if let Some(p) = &mut s.child(0) {
                p.set_size(w, p.h());
            }
        });

        let pack = group::Pack::default_fill().with_type(PackType::Vertical);

        for i in 0..layout.variants_len() {
            let mut b = button::Button::default().with_size(0, 20);

            b.set_label_size(16);

            b.set_color(colors::html::DodgerBlue);

            b.set_frame(FrameType::NoBox);

            if i == layout.default_variant_idx() {
                b.set_label(format!("{i} (default)").as_str());
            } else {
                b.set_label(i.to_string().as_str());
            }

            b.emit(sender.clone(), Message::SelectedVariantChanged(i));
        }

        pack.end();

        scroll.end();

        return scroll;
    }

    fn create_variant_state_selection(
        layout: &Layout,
        sender: &app::Sender<Message>,
    ) -> group::Scroll {
        let mut scroll = group::Scroll::default_fill().with_type(ScrollType::Horizontal);

        scroll.set_size(scroll.w() / 2, 72);

        // Any styling of the scrollbar should probably happen here

        scroll.set_color(Color::Background2);

        for variant in layout.get_variants() {
            Self::create_state_selection_pack(&scroll, variant, sender);
        }

        scroll.end();

        return scroll;
    }

    fn create_state_selection_pack(
        scroll: &group::Scroll,
        variant: &Variant,
        sender: &app::Sender<Message>,
    ) -> group::Pack {
        let mut pack = group::Pack::default()
            .with_size(scroll.w() - 8, 64)
            .with_type(PackType::Horizontal)
            .center_of_parent();

        pack.set_spacing(4);

        for j in 0..variant.manual_zones_until() {
            let mut b = button::Button::default()
                .with_size(64, 0)
                .with_label((j + 1).to_string().as_str());

            b.set_color(Color::Background2);

            b.set_selection_color(Color::Background);

            b.emit(sender.clone(), Message::SelectedVariantStateChanged(j));
        }

        pack.end();

        pack.hide();

        return pack;
    }

    fn create_variant_state_buttons(sender: &app::Sender<Message>) -> group::Flex {
        let mut flex = group::Flex::default_fill().row();

        let new_width = flex.w() / 2;

        WidgetExt::set_size(&mut flex, new_width, 32);

        flex.set_pad(4);

        let mut new_button = button::Button::default().with_label("New");

        let mut clone_button = button::Button::default().with_label("Clone");

        let mut delete_button = button::Button::default().with_label("Delete");

        let _frame = frame::Frame::default();

        let mut left_button = button::Button::default().with_label("@<");

        let mut right_button = button::Button::default().with_label("@>");

        flex.fixed(&new_button, 64);

        flex.fixed(&clone_button, 80);

        flex.fixed(&delete_button, 64);

        flex.fixed(&left_button, 32);

        flex.fixed(&right_button, 32);

        flex.end();

        new_button.emit(sender.clone(), Message::NewVariantState);

        clone_button.emit(sender.clone(), Message::CloneVariantState);

        delete_button.emit(sender.clone(), Message::DeleteVariantState);

        left_button.emit(
            sender.clone(),
            Message::SwapVariantState(SwapDirection::Previous),
        );

        right_button.emit(
            sender.clone(),
            Message::SwapVariantState(SwapDirection::Next),
        );

        return flex;
    }

    fn create_variant_state_display(
        layout: &Layout,
        sender: &app::Sender<Message>,
    ) -> group::Group {
        let mut group = group::Group::default_fill();

        group.set_size(group.w() / 2, group.h() / 2);

        for variant in layout.get_variants() {
            let mut display = group::Group::default_fill();

            for i in 0..variant.manual_zones_until() {
                let mut g = Self::display_group_from_variant_state(
                    layout.get_monitor_rect().w() as f64,
                    layout.get_monitor_rect().h() as f64,
                    variant,
                    i,
                    sender,
                );

                g.hide();
            }

            display.end();

            display.hide();
        }

        group.end();

        return group;
    }

    fn update_highlighted_variant(&mut self, old_idx: usize, new_idx: usize) {
        if let Some(p) = &self.variant_list.child(0) {
            let pack = group::Pack::from_dyn_widget(p).unwrap();

            if let Some(old_button) = &mut pack.child(old_idx as i32) {
                old_button.set_frame(FrameType::NoBox);
            }

            if let Some(new_button) = &mut pack.child(new_idx as i32) {
                new_button.set_frame(FrameType::UpBox);
            }
        }
    }

    fn update_shown_variant_state_selection(&mut self, old_idx: usize, new_idx: usize) {
        if let Some(old_pack) = &mut self.variant_state_selection.child(old_idx as i32) {
            old_pack.hide();
        }

        if let Some(new_pack) = &mut self.variant_state_selection.child(new_idx as i32) {
            new_pack.show();
        }
    }

    fn update_highlighted_variant_state_button(
        &mut self,
        old_idx: (usize, usize),
        new_idx: (usize, usize),
    ) {
        if let Some(old_pack) = &mut self.variant_state_selection.child(old_idx.0 as i32) {
            if let Some(old_button) = &mut group::Pack::from_dyn_widget(old_pack)
                .unwrap()
                .child(old_idx.1 as i32)
            {
                old_button.set_color(Color::Background2);
            }

            old_pack.hide();
        }

        if let Some(new_pack) = &mut self.variant_state_selection.child(new_idx.0 as i32) {
            if let Some(new_button) = &mut group::Pack::from_dyn_widget(new_pack)
                .unwrap()
                .child(new_idx.1 as i32)
            {
                new_button.set_color(colors::html::DimGray);
            }

            new_pack.show();
        }
    }

    fn update_shown_variant_state(&mut self, old_idx: (usize, usize), new_idx: (usize, usize)) {
        if let Some(old_variant) = &mut self.variant_state_display.child(old_idx.0 as i32) {
            if let Some(old_group) = &mut group::Group::from_dyn_widget(old_variant)
                .unwrap()
                .child(old_idx.1 as i32)
            {
                old_group.hide();
            }

            old_variant.hide();
        }

        if let Some(new_variant) = &mut self.variant_state_display.child(new_idx.0 as i32) {
            if let Some(new_group) = &mut group::Group::from_dyn_widget(new_variant)
                .unwrap()
                .child(new_idx.1 as i32)
            {
                new_group.show();
            }

            new_variant.show();
        }
    }

    fn highlight_selected_zone(&mut self, idx: usize) {
        let selected_variant_idx = self.editor.selected_variant_idx;

        let selected_variant_state_idx = self.editor.selected_variant_state_idx;

        if let Some(variant) = group::Group::from_dyn_widget(
            &self
                .variant_state_display
                .child(selected_variant_idx as i32)
                .unwrap(),
        ) {
            if let Some(group) = group::Group::from_dyn_widget(
                &variant.child(selected_variant_state_idx as i32).unwrap(),
            ) {
                if let Some(zone_button) =
                    &mut button::Button::from_dyn_widget(&group.child(idx as i32).unwrap())
                {
                    zone_button.set_color(colors::html::DodgerBlue);

                    zone_button.redraw();
                }
            }
        }
    }

    fn dehighlight_zone(&mut self, variant_idx: usize, variant_state_idx: usize, zone_idx: usize) {
        if let Some(variant) = group::Group::from_dyn_widget(
            &self
                .variant_state_display
                .child(variant_idx as i32)
                .unwrap(),
        ) {
            if let Some(group) =
                group::Group::from_dyn_widget(&variant.child(variant_state_idx as i32).unwrap())
            {
                if let Some(zone_button) =
                    &mut button::Button::from_dyn_widget(&group.child(zone_idx as i32).unwrap())
                {
                    zone_button.set_color(colors::html::Gainsboro);

                    zone_button.redraw();
                }
            }
        }
    }

    fn reset_zone_selection(&mut self) {
        self.disable_split();

        let variant_idx = self.editor.selected_variant_idx;

        let variant_state_idx = self.editor.selected_variant_state_idx;

        if let Some(zone_idx) = self.editor.selected_zone_idx1 {
            self.dehighlight_zone(variant_idx, variant_state_idx, zone_idx);

            self.editor.selected_zone_idx1 = None;

            if let Some(zone_idx) = self.editor.selected_zone_idx2 {
                self.dehighlight_zone(variant_idx, variant_state_idx, zone_idx);

                self.editor.selected_zone_idx2 = None;
            }
        }
    }

    fn new_variant_state(&mut self, sender: &app::Sender<Message>) {
        let variant_idx = self.editor.selected_variant_idx;

        let variant_state_pack = &mut group::Pack::from_dyn_widget(
            &self
                .variant_state_selection
                .child(variant_idx as i32)
                .unwrap(),
        )
        .unwrap();

        variant_state_pack.begin();

        let mut b = button::Button::default()
            .with_size(64, 0)
            .with_label(variant_state_pack.children().to_string().as_str());

        b.set_color(Color::Background2);

        b.set_selection_color(Color::Background);

        b.emit(
            sender.clone(),
            Message::SelectedVariantStateChanged(variant_state_pack.children() as usize - 1),
        );

        variant_state_pack.end();

        let variant_state_display_group = &mut group::Group::from_dyn_widget(
            &self
                .variant_state_display
                .child(variant_idx as i32)
                .unwrap(),
        )
        .unwrap();

        variant_state_display_group.begin();

        let _g = Self::display_group_from_variant_state(
            self.editor.layout.get_monitor_rect().w() as f64,
            self.editor.layout.get_monitor_rect().h() as f64,
            &self.editor.layout.get_variants()[variant_idx],
            variant_state_pack.children() as usize - 1,
            sender,
        );

        variant_state_display_group.end();

        sender.send(Message::SelectedVariantStateChanged(
            variant_state_pack.children() as usize - 1,
        ));
    }

    fn delete_variant_state(&mut self, sender: &app::Sender<Message>) {
        let variant_idx = self.editor.selected_variant_idx;

        let variant_state_idx = self.editor.selected_variant_state_idx;

        let variant_state_pack = &mut group::Pack::from_dyn_widget(
            &self
                .variant_state_selection
                .child(variant_idx as i32)
                .unwrap(),
        )
        .unwrap();

        WidgetBase::delete(variant_state_pack.child(variant_state_idx as i32).unwrap());

        let variant_state_display_group = &mut group::Group::from_dyn_widget(
            &self
                .variant_state_display
                .child(variant_idx as i32)
                .unwrap(),
        )
        .unwrap();

        WidgetBase::delete(
            variant_state_display_group
                .child(variant_state_idx as i32)
                .unwrap(),
        );

        self.decrement_all_variant_state_buttons_after(variant_idx, variant_state_idx, sender);

        if variant_state_idx
            == self.editor.layout.get_variants()[self.editor.selected_variant_idx]
                .manual_zones_until()
        {
            self.editor.selected_variant_state_idx -= 1;
        }

        sender.send(Message::SelectedVariantStateChanged(
            self.editor.selected_variant_state_idx,
        ));
    }

    fn swap_variant_states(&mut self, swap_with: usize, sender: &app::Sender<Message>) {
        let variant_idx = self.editor.selected_variant_idx;

        let first_idx = std::cmp::min(self.editor.selected_variant_state_idx, swap_with);
        let second_idx = std::cmp::max(self.editor.selected_variant_state_idx, swap_with);

        let variant_state_pack = &mut group::Pack::from_dyn_widget(
            &self
                .variant_state_selection
                .child(variant_idx as i32)
                .unwrap(),
        )
        .unwrap();

        let first_button = &mut button::Button::from_dyn_widget(
            &variant_state_pack.child(first_idx as i32).unwrap(),
        )
        .unwrap();

        let second_button = &mut button::Button::from_dyn_widget(
            &variant_state_pack.child(second_idx as i32).unwrap(),
        )
        .unwrap();

        variant_state_pack.remove_by_index(second_idx as i32);

        variant_state_pack.remove_by_index(first_idx as i32);

        first_button.set_label((second_idx + 1).to_string().as_str());

        first_button.emit(
            sender.clone(),
            Message::SelectedVariantStateChanged(second_idx),
        );

        second_button.set_label((first_idx + 1).to_string().as_str());

        second_button.emit(
            sender.clone(),
            Message::SelectedVariantStateChanged(first_idx),
        );

        variant_state_pack.insert(second_button, first_idx as i32);

        variant_state_pack.insert(first_button, second_idx as i32);

        let variant_state_display_group = &mut group::Group::from_dyn_widget(
            &self
                .variant_state_display
                .child(variant_idx as i32)
                .unwrap(),
        )
        .unwrap();

        let first_variant_state_display = &group::Group::from_dyn_widget(
            &variant_state_display_group.child(first_idx as i32).unwrap(),
        )
        .unwrap();

        let second_variant_state_display = &group::Group::from_dyn_widget(
            &variant_state_display_group
                .child(second_idx as i32)
                .unwrap(),
        )
        .unwrap();

        variant_state_display_group.remove_by_index(second_idx as i32);

        variant_state_display_group.remove_by_index(first_idx as i32);

        variant_state_display_group.insert(second_variant_state_display, first_idx as i32);

        variant_state_display_group.insert(first_variant_state_display, second_idx as i32);

        sender.send(Message::SelectedVariantStateChanged(swap_with));
    }

    fn update_split_bounds(&mut self) {
        let variant_state = &self.editor.layout.get_variants()[self.editor.selected_variant_idx]
            .get_zones()[self.editor.selected_variant_state_idx];

        let selected_zone1 = &variant_state[self.editor.selected_zone_idx1.unwrap()];

        let max = match self.actions.selected_direction {
            Direction::Horizontal => {
                if let Some(idx) = self.editor.selected_zone_idx2 {
                    let selected_zone2 = &variant_state[idx];

                    std::cmp::max(selected_zone1.right, selected_zone2.right)
                        - std::cmp::min(selected_zone1.left, selected_zone2.left)
                } else {
                    selected_zone1.w()
                }
            }
            Direction::Vertical => {
                if let Some(idx) = self.editor.selected_zone_idx2 {
                    let selected_zone2 = &variant_state[idx];

                    std::cmp::max(selected_zone1.bottom, selected_zone2.bottom)
                        - std::cmp::min(selected_zone1.top, selected_zone2.top)
                } else {
                    selected_zone1.h()
                }
            }
        };

        self.actions.split_bound_max = Some(max);

        self.actions
            .split_bounds_text
            .set_label(format!("0 - {max}").as_str());
    }

    fn disable_split(&mut self) {
        self.actions.split_bound_max = None;

        self.actions.split_button.set_label("Split");

        self.actions.split_button.deactivate();

        self.actions.split_bounds_text.set_label("");
    }

    fn update_variant_state_display(
        &mut self,
        variant_idx: usize,
        variant_state_idx: usize,
        sender: &app::Sender<Message>,
    ) {
        let w = self.editor.layout.get_monitor_rect().w() as f64;

        let h = self.editor.layout.get_monitor_rect().h() as f64;

        let variant = &self.editor.layout.get_variants()[variant_idx];

        let variant_display_group = &mut group::Group::from_dyn_widget(
            &self
                .variant_state_display
                .child(variant_idx as i32)
                .unwrap(),
        )
        .unwrap();

        variant_display_group.begin();

        let mut new_display = EditorWidgets::display_group_from_variant_state(
            w,
            h,
            variant,
            variant_state_idx,
            sender,
        );

        variant_display_group.end();

        variant_display_group.remove(&new_display);

        if let Some(old_display) = variant_display_group.child(variant_state_idx as i32) {
            WidgetBase::delete(old_display);
        } else {
            new_display.hide();
        }

        variant_display_group.insert(&new_display, variant_state_idx as i32);

        self.reset_zone_selection();
    }

    fn add_new_variant(&mut self, variant: Variant, sender: &app::Sender<Message>) {
        let mut variants_pack =
            group::Pack::from_dyn_widget(&self.variant_list.child(0).unwrap()).unwrap();

        let idx = variants_pack.children() as usize;

        let mut new_variant_button = button::Button::default().with_size(0, 20);

        new_variant_button.set_label_size(16);

        new_variant_button.set_color(colors::html::DodgerBlue);

        new_variant_button.set_frame(FrameType::NoBox);

        new_variant_button.set_label(idx.to_string().as_str());

        new_variant_button.emit(sender.clone(), Message::SelectedVariantChanged(idx));

        variants_pack.add(&new_variant_button);

        let variant_state_selection = &mut self.variant_state_selection;

        variant_state_selection.begin();

        variant_state_selection.add(&EditorWidgets::create_state_selection_pack(
            variant_state_selection,
            &variant,
            sender,
        ));

        variant_state_selection.end();

        self.variant_state_display.begin();

        let mut new_variant_display_group = group::Group::default_fill();

        self.variant_state_display.end();

        let manual_zones_until = variant.manual_zones_until();

        self.editor.layout.get_variants_mut().push(variant);

        for i in 0..manual_zones_until {
            self.update_variant_state_display(idx, i, sender);
        }

        new_variant_display_group.hide();
    }

    fn update_default_variant_label(&mut self, old_idx: usize, new_idx: usize) {
        let variants_pack =
            &mut group::Pack::from_dyn_widget(&self.variant_list.child(0).unwrap()).unwrap();

        if let Some(button) = &mut variants_pack.child(old_idx as i32) {
            button.set_label(old_idx.to_string().as_str());
        }

        variants_pack
            .child(new_idx as i32)
            .unwrap()
            .set_label(format!("{new_idx} (default)").as_str());
    }

    fn decrement_all_variant_state_buttons_after(
        &mut self,
        variant_idx: usize,
        variant_state_idx: usize,
        sender: &app::Sender<Message>,
    ) {
        let variant_state_pack = &mut group::Pack::from_dyn_widget(
            &self
                .variant_state_selection
                .child(variant_idx as i32)
                .unwrap(),
        )
        .unwrap();

        for i in variant_state_idx as i32..variant_state_pack.children() {
            let b = &mut button::Button::from_dyn_widget(&variant_state_pack.child(i).unwrap())
                .unwrap();

            b.set_label((i + 1).to_string().as_str());

            b.emit(
                sender.clone(),
                Message::SelectedVariantStateChanged(i as usize),
            );
        }
    }
}

pub struct LayoutEditorGUI {
    app: app::App,
    window: window::Window,
    sender: app::Sender<Message>,
    receiver: app::Receiver<Message>,
    editor_widgets: Option<EditorWidgets>,
}

impl LayoutEditorGUI {
    pub fn create() -> Self {
        initialize_colours();

        let app = app::App::default();

        let widget_scheme = WidgetScheme::new(SchemeType::Fluent);

        widget_scheme.apply();

        let (sender, receiver) = app::channel();

        return LayoutEditorGUI {
            app,
            window: create_window(),
            sender,
            receiver,
            editor_widgets: None,
        };
    }

    pub fn edit_layout(&mut self, layout: Layout) {
        self.window.begin();

        self.editor_widgets = Some(EditorWidgets::initialize(layout, &self.sender));

        self.window.end();

        // Test code

        if let Some(editor) = &mut self.editor_widgets {
            editor
                .variant_state_pack
                .set_pos(self.window.w() / 2 - editor.variant_state_pack.w() / 2, 0);

            editor.variant_state_display.set_pos(
                self.window.w() / 2 - editor.variant_state_display.w() / 2,
                self.window.h() / 2 - editor.variant_state_display.h() / 2,
            );

            editor
                .actions
                .widgets
                .set_pos(self.window.w() - editor.actions.widgets.w() - 4, 0);

            editor
                .variant_actions
                .widgets
                .set_pos(0, editor.variant_list.h() + 4);

            editor.end_behaviour_actions.widgets.set_pos(
                self.window.w() - editor.end_behaviour_actions.widgets.w(),
                self.window.h() / 2,
            );
        }
    }

    fn handle_events(&mut self) {
        let editor_widgets = match &mut self.editor_widgets {
            Some(val) => val,

            None => return,
        };

        if let Some(msg) = self.receiver.recv() {
            match msg {
                Message::SelectedVariantChanged(idx) => {
                    editor_widgets.reset_zone_selection();

                    let old_variant_idx = editor_widgets.editor.selected_variant_idx;

                    let old_variant_state_idx = editor_widgets.editor.selected_variant_state_idx;

                    editor_widgets.editor.selected_variant_idx = idx;

                    editor_widgets.editor.selected_variant_state_idx = 0;

                    editor_widgets.update_highlighted_variant(old_variant_idx, idx);

                    editor_widgets.update_shown_variant_state_selection(old_variant_idx, idx);

                    editor_widgets.update_highlighted_variant_state_button(
                        (old_variant_idx, old_variant_state_idx),
                        (idx, 0),
                    );

                    editor_widgets.update_shown_variant_state(
                        (old_variant_idx, old_variant_state_idx),
                        (idx, 0),
                    );

                    if idx == editor_widgets.editor.layout.default_variant_idx() {
                        editor_widgets
                            .variant_actions
                            .set_as_default_button
                            .deactivate();
                    } else if !editor_widgets
                        .variant_actions
                        .set_as_default_button
                        .active()
                    {
                        editor_widgets
                            .variant_actions
                            .set_as_default_button
                            .activate();
                    }
                }

                Message::SelectedVariantStateChanged(idx) => {
                    editor_widgets.reset_zone_selection();

                    let variant_idx = editor_widgets.editor.selected_variant_idx;

                    let old_idx = editor_widgets.editor.selected_variant_state_idx;

                    editor_widgets.editor.selected_variant_state_idx = idx;

                    editor_widgets.update_highlighted_variant_state_button(
                        (variant_idx, old_idx),
                        (variant_idx, idx),
                    );

                    editor_widgets
                        .update_shown_variant_state((variant_idx, old_idx), (variant_idx, idx));
                }

                Message::SelectedZoneChanged(idx) => {
                    let selected_variant_idx = editor_widgets.editor.selected_variant_idx;

                    let selected_variant_state_idx =
                        editor_widgets.editor.selected_variant_state_idx;

                    if app::is_event_shift() {
                        if editor_widgets.editor.selected_zone_idx1 == None
                            || editor_widgets.editor.selected_zone_idx1 == Some(idx)
                        {
                            return;
                        }

                        if let Some(old_idx) = editor_widgets.editor.selected_zone_idx2 {
                            editor_widgets.dehighlight_zone(
                                selected_variant_idx,
                                selected_variant_state_idx,
                                old_idx,
                            );

                            if old_idx == idx {
                                editor_widgets.editor.selected_zone_idx2 = None;

                                editor_widgets.actions.split_button.set_label("Split");

                                editor_widgets.actions.split_button.activate();

                                editor_widgets.update_split_bounds();

                                editor_widgets.actions.merge_button.deactivate();

                                editor_widgets.actions.swap_button.deactivate();

                                return;
                            }
                        }

                        editor_widgets.editor.selected_zone_idx2 = Some(idx);

                        editor_widgets.highlight_selected_zone(idx);

                        editor_widgets.actions.swap_button.activate();

                        let variant =
                            &editor_widgets.editor.layout.get_variants()[selected_variant_idx];

                        if variant.can_merge_zones(
                            selected_variant_state_idx,
                            editor_widgets.editor.selected_zone_idx1.unwrap(),
                            idx,
                        ) {
                            editor_widgets
                                .actions
                                .split_button
                                .set_label("Merge and split");

                            editor_widgets.actions.split_button.activate();

                            editor_widgets.update_split_bounds();

                            editor_widgets.actions.merge_button.activate();
                        } else {
                            editor_widgets.disable_split();

                            editor_widgets.actions.merge_button.deactivate();
                        }
                    } else {
                        editor_widgets.actions.split_button.set_label("Split");

                        editor_widgets.actions.merge_button.deactivate();

                        editor_widgets.actions.swap_button.deactivate();

                        let mut new_selection = true;

                        if let Some(old_idx) = editor_widgets.editor.selected_zone_idx1 {
                            editor_widgets.dehighlight_zone(
                                selected_variant_idx,
                                selected_variant_state_idx,
                                old_idx,
                            );

                            if old_idx == idx {
                                editor_widgets.editor.selected_zone_idx1 = None;

                                new_selection = false;
                            }
                        }

                        if let Some(old_idx) = editor_widgets.editor.selected_zone_idx2 {
                            editor_widgets.editor.selected_zone_idx2 = None;

                            if old_idx == idx {
                                editor_widgets.editor.selected_zone_idx1 = Some(idx);

                                editor_widgets.update_split_bounds();

                                return;
                            }

                            editor_widgets.dehighlight_zone(
                                selected_variant_idx,
                                selected_variant_state_idx,
                                old_idx,
                            );
                        }

                        if new_selection {
                            editor_widgets.editor.selected_zone_idx1 = Some(idx);

                            editor_widgets.highlight_selected_zone(idx);

                            editor_widgets.actions.split_button.activate();

                            editor_widgets.update_split_bounds();
                        } else {
                            editor_widgets.disable_split();
                        }
                    }
                }

                Message::NewVariantState => {
                    let w = editor_widgets.editor.layout.get_monitor_rect().w();

                    let h = editor_widgets.editor.layout.get_monitor_rect().h();

                    let variant = &mut editor_widgets.editor.layout.get_variants_mut()
                        [editor_widgets.editor.selected_variant_idx];

                    variant.new_zone_vec(w, h);

                    editor_widgets.new_variant_state(&self.sender);
                }

                Message::CloneVariantState => {
                    let variant = &mut editor_widgets.editor.layout.get_variants_mut()
                        [editor_widgets.editor.selected_variant_idx];

                    let idx = editor_widgets.editor.selected_variant_state_idx;

                    variant.clone_zone_vec(idx);

                    editor_widgets.new_variant_state(&self.sender);
                }

                Message::DeleteVariantState => {
                    let variant = &mut editor_widgets.editor.layout.get_variants_mut()
                        [editor_widgets.editor.selected_variant_idx];

                    let idx = editor_widgets.editor.selected_variant_state_idx;

                    variant.delete_zones(idx);

                    editor_widgets.delete_variant_state(&self.sender);
                }

                Message::SwapVariantState(swap_direction) => {
                    let variant = &mut editor_widgets.editor.layout.get_variants_mut()
                        [editor_widgets.editor.selected_variant_idx];

                    let selected_variant_state_idx =
                        editor_widgets.editor.selected_variant_state_idx;

                    let swap_with = match swap_direction {
                        SwapDirection::Previous if selected_variant_state_idx != 0 => {
                            selected_variant_state_idx - 1
                        }
                        SwapDirection::Next
                            if editor_widgets.editor.selected_variant_state_idx
                                != variant.manual_zones_until() - 1 =>
                        {
                            selected_variant_state_idx + 1
                        }
                        _ => return,
                    };

                    variant.swap_zone_vectors(selected_variant_state_idx, swap_with);

                    editor_widgets.swap_variant_states(swap_with, &self.sender);
                }

                Message::SwapSplitDirection => {
                    editor_widgets.actions.selected_direction =
                        editor_widgets.actions.selected_direction.other();

                    match editor_widgets.actions.selected_direction {
                        Direction::Horizontal => {
                            editor_widgets.actions.split_axis_text.set_label("x: ");
                        }
                        Direction::Vertical => {
                            editor_widgets.actions.split_axis_text.set_label("y: ");
                        }
                    }

                    if let Some(_) = editor_widgets.actions.split_bound_max {
                        editor_widgets.update_split_bounds();
                    }
                }

                Message::Split => {
                    let split_at: i32 = match editor_widgets.actions.split_at_input.value().parse()
                    {
                        Ok(val)
                            if val > 0 && val < editor_widgets.actions.split_bound_max.unwrap() =>
                        {
                            val
                        }
                        _ => {
                            editor_widgets.reset_zone_selection();

                            return;
                        }
                    };

                    editor_widgets.actions.split_at_input.set_value("");

                    let mut zone_idx = editor_widgets.editor.selected_zone_idx1.unwrap();

                    let selected_variant_idx = editor_widgets.editor.selected_variant_idx;

                    let selected_variant_state_idx =
                        editor_widgets.editor.selected_variant_state_idx;

                    let variant =
                        &mut editor_widgets.editor.layout.get_variants_mut()[selected_variant_idx];

                    let other_zone_idx = if let Some(idx) = editor_widgets.editor.selected_zone_idx2
                    {
                        variant.merge_zones(selected_variant_state_idx, zone_idx, idx);

                        zone_idx = std::cmp::min(zone_idx, idx);

                        Some(std::cmp::max(zone_idx, idx))
                    } else {
                        None
                    };

                    let zone = &variant.get_zones()[selected_variant_state_idx][zone_idx];

                    let direction = match editor_widgets.actions.selected_direction {
                        Direction::Horizontal => SplitDirection::Horizontal(zone.left + split_at),
                        Direction::Vertical => SplitDirection::Vertical(zone.top + split_at),
                    };

                    variant.split(selected_variant_state_idx, zone_idx, direction);

                    if let Some(idx) = other_zone_idx {
                        let zone = variant.get_zones_mut()[selected_variant_state_idx]
                            .pop()
                            .unwrap();

                        variant.get_zones_mut()[selected_variant_state_idx].insert(idx, zone);
                    }

                    editor_widgets.update_variant_state_display(
                        selected_variant_idx,
                        selected_variant_state_idx,
                        &self.sender,
                    );
                }

                Message::Swap => {
                    let selected_variant_idx = editor_widgets.editor.selected_variant_idx;

                    let selected_variant_state_idx =
                        editor_widgets.editor.selected_variant_state_idx;

                    let selected_zone_idx1 = editor_widgets.editor.selected_zone_idx1.unwrap();

                    let selected_zone_idx2 = editor_widgets.editor.selected_zone_idx2.unwrap();

                    let variant =
                        &mut editor_widgets.editor.layout.get_variants_mut()[selected_variant_idx];

                    variant.swap_zones(
                        selected_variant_state_idx,
                        selected_zone_idx1,
                        selected_zone_idx2,
                    );

                    editor_widgets.update_variant_state_display(
                        selected_variant_idx,
                        selected_variant_state_idx,
                        &self.sender,
                    );
                }

                Message::Merge => {
                    let selected_variant_idx = editor_widgets.editor.selected_variant_idx;

                    let selected_variant_state_idx =
                        editor_widgets.editor.selected_variant_state_idx;

                    let selected_zone_idx1 = editor_widgets.editor.selected_zone_idx1.unwrap();

                    let selected_zone_idx2 = editor_widgets.editor.selected_zone_idx2.unwrap();

                    let variant =
                        &mut editor_widgets.editor.layout.get_variants_mut()[selected_variant_idx];

                    variant.merge_zones(
                        selected_variant_state_idx,
                        selected_zone_idx1,
                        selected_zone_idx2,
                    );

                    editor_widgets.editor.selected_zone_idx1 = None;

                    editor_widgets.editor.selected_zone_idx2 = None;

                    editor_widgets.update_variant_state_display(
                        selected_variant_idx,
                        selected_variant_state_idx,
                        &self.sender,
                    );
                }

                Message::NewVariant => {
                    let layout = &editor_widgets.editor.layout;

                    let w = layout.get_monitor_rect().w();

                    let h = layout.get_monitor_rect().h();

                    let new_variant = Variant::new(w, h);

                    editor_widgets.add_new_variant(new_variant, &self.sender);

                    self.sender.send(Message::SelectedVariantChanged(
                        editor_widgets.editor.layout.variants_len() - 1,
                    ));

                    if !editor_widgets.variant_actions.delete_button.active() {
                        editor_widgets.variant_actions.delete_button.activate();
                    }
                }

                Message::CloneVariant => {
                    let layout = &mut editor_widgets.editor.layout;

                    let new_variant =
                        layout.get_variants()[editor_widgets.editor.selected_variant_idx].clone();

                    editor_widgets.add_new_variant(new_variant, &self.sender);

                    self.sender.send(Message::SelectedVariantChanged(
                        editor_widgets.editor.layout.variants_len() - 1,
                    ));

                    if !editor_widgets.variant_actions.delete_button.active() {
                        editor_widgets.variant_actions.delete_button.activate();
                    }
                }

                Message::DeleteVariant => {
                    let selected_variant_idx = editor_widgets.editor.selected_variant_idx;

                    let old_default_variant_idx =
                        editor_widgets.editor.layout.default_variant_idx();

                    editor_widgets
                        .editor
                        .layout
                        .delete_variant(selected_variant_idx);

                    let variants_pack = group::Pack::from_dyn_widget(
                        &editor_widgets.variant_list.child(0).unwrap(),
                    )
                    .unwrap();

                    let variant_state_selection = &editor_widgets.variant_state_selection;

                    let variant_state_display = &editor_widgets.variant_state_display;

                    WidgetBase::delete(variants_pack.child(selected_variant_idx as i32).unwrap());

                    WidgetBase::delete(
                        variant_state_selection
                            .child(selected_variant_idx as i32)
                            .unwrap(),
                    );

                    WidgetBase::delete(
                        variant_state_display
                            .child(selected_variant_idx as i32)
                            .unwrap(),
                    );

                    if selected_variant_idx == editor_widgets.editor.layout.variants_len() {
                        editor_widgets.editor.selected_variant_idx = selected_variant_idx - 1;

                        self.sender
                            .send(Message::SelectedVariantChanged(selected_variant_idx - 1));
                    } else {
                        self.sender
                            .send(Message::SelectedVariantChanged(selected_variant_idx));
                    }

                    for i in selected_variant_idx as i32..variants_pack.children() {
                        let b =
                            &mut button::Button::from_dyn_widget(&variants_pack.child(i).unwrap())
                                .unwrap();

                        b.set_label(i.to_string().as_str());

                        b.emit(
                            self.sender.clone(),
                            Message::SelectedVariantChanged(i as usize),
                        );
                    }

                    editor_widgets.update_default_variant_label(
                        old_default_variant_idx,
                        editor_widgets.editor.layout.default_variant_idx(),
                    );

                    if editor_widgets.editor.layout.variants_len() == 1 {
                        editor_widgets.variant_actions.delete_button.deactivate();
                    }
                }

                Message::SwapVariant(swap_direction) => {
                    let selected_variant_idx = editor_widgets.editor.selected_variant_idx;

                    let new_idx;

                    let variants_pack = &mut group::Pack::from_dyn_widget(
                        &editor_widgets.variant_list.child(0).unwrap(),
                    )
                    .unwrap();

                    let variant_state_selection = &mut editor_widgets.variant_state_selection;

                    let variant_state_display = &mut editor_widgets.variant_state_display;

                    let first_idx;

                    let second_idx;

                    match swap_direction {
                        SwapDirection::Previous => {
                            if selected_variant_idx == 0 {
                                return;
                            } else {
                                first_idx = selected_variant_idx - 1;

                                second_idx = selected_variant_idx;

                                new_idx = selected_variant_idx - 1;
                            }
                        }
                        SwapDirection::Next => {
                            if selected_variant_idx
                                == editor_widgets.editor.layout.variants_len() - 1
                            {
                                return;
                            } else {
                                first_idx = selected_variant_idx;

                                second_idx = selected_variant_idx + 1;

                                new_idx = selected_variant_idx + 1;
                            }
                        }
                    };

                    editor_widgets
                        .editor
                        .layout
                        .swap_variants(first_idx, second_idx);

                    let first_variant_button = &mut button::Button::from_dyn_widget(
                        &variants_pack.child(first_idx as i32).unwrap(),
                    )
                    .unwrap();

                    let second_variant_button = &mut button::Button::from_dyn_widget(
                        &variants_pack.child(second_idx as i32).unwrap(),
                    )
                    .unwrap();

                    if second_idx == editor_widgets.editor.layout.default_variant_idx() {
                        first_variant_button.set_label(format!("{second_idx} (default)").as_str());
                    } else {
                        first_variant_button.set_label(second_idx.to_string().as_str());
                    }

                    if first_idx == editor_widgets.editor.layout.default_variant_idx() {
                        second_variant_button.set_label(format!("{first_idx} (default)").as_str());
                    } else {
                        second_variant_button.set_label(first_idx.to_string().as_str());
                    }

                    first_variant_button.emit(
                        self.sender.clone(),
                        Message::SelectedVariantChanged(second_idx),
                    );

                    second_variant_button.emit(
                        self.sender.clone(),
                        Message::SelectedVariantChanged(first_idx),
                    );

                    let first_state_selection_pack =
                        variant_state_selection.child(first_idx as i32).unwrap();

                    let second_state_selection_pack =
                        variant_state_selection.child(second_idx as i32).unwrap();

                    let first_variant_state_display_group =
                        variant_state_display.child(first_idx as i32).unwrap();

                    let second_variant_state_display_group =
                        variant_state_display.child(second_idx as i32).unwrap();

                    variants_pack.remove_by_index(second_idx as i32);

                    variants_pack.remove_by_index(first_idx as i32);

                    variant_state_selection.remove_by_index(second_idx as i32);

                    variant_state_selection.remove_by_index(first_idx as i32);

                    variant_state_display.remove_by_index(second_idx as i32);

                    variant_state_display.remove_by_index(first_idx as i32);

                    variants_pack.insert(second_variant_button, first_idx as i32);

                    variants_pack.insert(first_variant_button, second_idx as i32);

                    variant_state_selection.insert(&second_state_selection_pack, first_idx as i32);

                    variant_state_selection.insert(&first_state_selection_pack, second_idx as i32);

                    variant_state_display
                        .insert(&second_variant_state_display_group, first_idx as i32);

                    variant_state_display
                        .insert(&first_variant_state_display_group, second_idx as i32);

                    editor_widgets.editor.selected_variant_idx = new_idx;

                    self.sender.send(Message::SelectedVariantChanged(new_idx));
                }

                Message::SetVariantAsDefault => {
                    let selected_variant_idx = editor_widgets.editor.selected_variant_idx;

                    let old_default_variant_idx =
                        editor_widgets.editor.layout.default_variant_idx();

                    editor_widgets
                        .editor
                        .layout
                        .set_default_variant_idx(selected_variant_idx);

                    editor_widgets.update_default_variant_label(
                        old_default_variant_idx,
                        selected_variant_idx,
                    );

                    editor_widgets
                        .variant_actions
                        .set_as_default_button
                        .deactivate();
                }
            }
        }
    }

    pub fn run(mut self) {
        self.window.show();

        while self.app.wait() {
            self.handle_events();
        }
    }
}

fn initialize_colours() {
    app::background(16, 16, 16);

    app::background2(32, 32, 32);
}

fn create_window() -> window::Window {
    let primary_screen = app::Screen::new(0).unwrap();

    let mut window = window::Window::new(
        primary_screen.w() / 4,
        primary_screen.h() / 4,
        primary_screen.w() / 2,
        primary_screen.h() / 2,
        "PLACEHOLDER",
    );

    window.make_resizable(true);

    window.end();

    return window;
}
