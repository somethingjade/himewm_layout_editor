mod handler;

use handler::handle_events;
use enums::{Align, Color, FrameType};
use fltk::{enums::Shortcut, group::FlexType, menu::MenuFlag, prelude::{InputExt, MenuExt}, *};
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
    PreviewExtend,
    CancelPreview,
    EndZoneIdxChanged(usize),
    SwapEndTilingDirection,
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

struct DirectionalActions {
    widgets: group::Flex,
}

impl DirectionalActions {
    fn initialize(layout: &Layout, sender: &app::Sender<Message>) -> Self {
        let widgets = group::Flex::default_fill().column();

        let _direction_label = frame::Frame::default().with_label("Direction");

        let mut horizontal_radio_button = button::RadioRoundButton::default().with_label("Horizontal");

        horizontal_radio_button.emit(sender.clone(), Message::SwapEndTilingDirection);

        let mut vertical_radio_button = button::RadioRoundButton::default().with_label("Vertical");

        vertical_radio_button.emit(sender.clone(), Message::SwapEndTilingDirection);

        match layout.get_variants()[layout.default_variant_idx()].get_end_tiling_direction() {
            Some(direction) => {
                match direction {
                    Direction::Horizontal => {
                        horizontal_radio_button.toggle(true);
                    }
                    Direction::Vertical => {
                        vertical_radio_button.toggle(true);
                    }
                }
            }

            None => {
                vertical_radio_button.toggle(true);
            }
        }

        let _compression_frame = frame::Frame::default();

        widgets.end();

        return DirectionalActions {
            widgets,
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
    preview_count: u32,
    widgets: group::Flex,
    zone_idx_choice: menu::Choice,
    cancel_preview_button: button::Button,
    directional_actions: DirectionalActions,
    repeating_split_actions: RepeatingSplitActions,
}

impl EndBehaviourActions {
    fn initialize(layout: &Layout, sender: &app::Sender<Message>) -> Self {
        let mut widgets = group::Flex::default_fill().row();

        let w = widgets.w() / 3;

        let h = widgets.h() / 4;

        WidgetExt::set_size(&mut widgets, w, h);

        let mut behaviour_column = group::Flex::default().column();

        let _offset_frame = frame::Frame::default();

        let mut directional_radio_button =
            button::RadioRoundButton::default().with_label("Directional");

        let _repeating_radio_button = button::RadioRoundButton::default().with_label("Repeating");

        directional_radio_button.toggle(true);

        let mut zone_idx_flex = group::Flex::default().row();

        let zone_idx_text = frame::Frame::default()
            .with_align(Align::Left.union(Align::Inside))
            .with_label("In zone: ");

        zone_idx_flex.fixed(&zone_idx_text, 96);

        let zone_idx_choice = menu::Choice::default();

        zone_idx_flex.end();

        behaviour_column.fixed(&zone_idx_flex, 32);

        let mut preview_button = button::Button::default().with_label("Preview next");

        preview_button.emit(sender.clone(), Message::PreviewExtend);

        behaviour_column.fixed(&preview_button, 32);

        let mut cancel_preview_button = button::Button::default().with_label("Stop previewing");

        cancel_preview_button.emit(sender.clone(), Message::CancelPreview);

        behaviour_column.fixed(&cancel_preview_button, 32);

        cancel_preview_button.deactivate();

        behaviour_column.end();

        let column_spacer = frame::Frame::default();

        widgets.fixed(&column_spacer, 16);

        widgets.end();

        let directional_actions = DirectionalActions::initialize(layout, sender);

        widgets.add(&directional_actions.widgets);

        let repeating_split_actions = RepeatingSplitActions::initialize();

        return EndBehaviourActions {
            preview_count: 0,
            widgets,
            zone_idx_choice,
            cancel_preview_button,
            directional_actions,
            repeating_split_actions,
        };
    }
    
}

#[derive(Clone)]
struct Buffers {
    layout: Layout,
    variant_state_selection: group::Scroll,
    variant_state_pack: group::Pack,
    variant_state_display: group::Group
}

impl Buffers {
    fn new(layout: Layout, variant_state_selection: group::Scroll, variant_state_pack: group::Pack, variant_state_display: group::Group) -> Self {
        Buffers {
            layout,
            variant_state_selection,
            variant_state_pack,
            variant_state_display,
        }
    }
}

struct EditorWidgets {
    editor: LayoutEditor,
    buffers: Option<Buffers>,
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

        let end_behaviour_actions = EndBehaviourActions::initialize(&layout, sender);

        let editor = LayoutEditor::new(layout);

        let buffers = None;

        let mut ret = EditorWidgets {
            editor,
            buffers,
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

        ret.update_end_zone_idx_choice(sender);

        ret.end_behaviour_actions.zone_idx_choice.set_value(ret.editor.layout.get_variants()[ret.editor.selected_variant_idx].get_end_zone_idx() as i32);

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

    fn delete_variant_state(&mut self, sender: &app::Sender<Message>, custom_idx: Option<usize>) {
        let variant_idx = self.editor.selected_variant_idx;

        let variant_state_idx = match custom_idx {
            Some(idx) => idx,
            None => self.editor.selected_variant_state_idx
        };

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

        if let None = custom_idx {
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

    fn update_end_zone_idx_choice(&mut self, sender: &app::Sender<Message>) {
        let variant = &self.editor.layout.get_variants()[self.editor.selected_variant_idx];

        let zones = &variant.get_zones()[variant.manual_zones_until() - 1];

        self.end_behaviour_actions.zone_idx_choice.clear();
        
        for idx in 0..zones.len() {
            self.end_behaviour_actions.zone_idx_choice.add_emit((idx + 1).to_string().as_str(), Shortcut::None, MenuFlag::Normal, sender.clone(), Message::EndZoneIdxChanged(idx));
        }

        self.end_behaviour_actions.zone_idx_choice.set_value(variant.get_end_zone_idx() as i32);
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

    fn preview_extend(&mut self, variant_idx: usize, update_count: bool, sender: &app::Sender<Message>) {
        if let None = self.buffers {
            let layout_buffer = self.editor.layout.clone();

            let mut display_buffer = EditorWidgets::create_variant_state_display(&self.editor.layout, sender);

            display_buffer.set_pos(self.variant_state_display.x(), self.variant_state_display.y());

            display_buffer.hide();

            let selection_buffer = EditorWidgets::create_variant_state_selection(&self.editor.layout, sender);

            let state_buttons_buffer =
                EditorWidgets::create_variant_state_buttons(sender).below_of(&selection_buffer, 4);

            let state_pack_buffer_h = selection_buffer.h() + state_buttons_buffer.h();

            let mut variant_state_pack_buffer =
                group::Pack::default().with_size(selection_buffer.w(), state_pack_buffer_h);

            variant_state_pack_buffer.end();

            variant_state_pack_buffer.set_spacing(4);

            variant_state_pack_buffer.resize_callback(move |p, _, _, _, h| {
                if h != state_pack_buffer_h {
                    p.widget_resize(p.x(), p.y(), p.w(), state_pack_buffer_h);
                }
            });

            variant_state_pack_buffer.add(&selection_buffer);

            variant_state_pack_buffer.add(&state_buttons_buffer);

            variant_state_pack_buffer.set_pos(self.variant_state_pack.x(), self.variant_state_pack.y());

            variant_state_pack_buffer.hide();
            
            self.buffers = Some(Buffers::new(layout_buffer, selection_buffer, variant_state_pack_buffer, display_buffer));

            self.variant_state_pack.child(1).unwrap().deactivate();

            self.editor.layout.get_variants_mut()[variant_idx].update_from_zones();

            if self.editor.layout.get_variants()[variant_idx].using_from_zones() {
                self.delete_variant_state(sender, Some(self.editor.layout.get_variants()[variant_idx].manual_zones_until()));
            }
        }

        if update_count {
            self.end_behaviour_actions.preview_count += 1;
        }

        self.editor.layout.get_variants_mut()[variant_idx].extend();

        self.new_variant_state(sender);
    }

    fn remove_extend_preview(&mut self) {
        let Buffers{layout, variant_state_selection, variant_state_pack, variant_state_display } = match &self.buffers {
            Some(buffers) => buffers.to_owned(),
            None => return,
        };

        self.buffers = None;

        self.editor.layout = layout;

        WidgetBase::delete(self.variant_state_display.to_owned());

        self.variant_state_display = variant_state_display;

        self.variant_state_display.show();

        WidgetBase::delete(self.variant_state_selection.to_owned());

        self.variant_state_selection = variant_state_selection;

        WidgetBase::delete(self.variant_state_pack.to_owned());

        self.variant_state_pack = variant_state_pack;

        self.variant_state_pack.show();

        self.variant_list.redraw();
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

    pub fn run(mut self) {
        self.window.show();

        while self.app.wait() {
            handle_events(&mut self);
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
