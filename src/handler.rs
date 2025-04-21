use crate::{LayoutEditorGUI, Message, SwapDirection};
use fltk::{prelude::InputExt, *};
use himewm_layout::*;
use prelude::{GroupExt, WidgetBase, WidgetExt};

pub fn handle_events(layout_editor: &mut LayoutEditorGUI) {
    let editor_widgets = match &mut layout_editor.editor_widgets {
        Some(val) => val,

        None => return,
    };

    if let Some(msg) = layout_editor.receiver.recv() {
        match msg {
            Message::SelectedVariantChanged(idx) => {
                editor_widgets.reset_zone_selection();

                if let Some(_) = &editor_widgets.buffers {
                    editor_widgets
                        .end_behaviour_actions
                        .cancel_preview_button
                        .deactivate();

                    editor_widgets.actions.widgets.activate();

                    editor_widgets.remove_extend_preview();

                    editor_widgets.end_behaviour_actions.preview_count = 0;
                }

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

                editor_widgets
                    .update_shown_variant_state((old_variant_idx, old_variant_state_idx), (idx, 0));

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

                editor_widgets.update_end_zone_idx_choice(&layout_editor.sender);
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

                let selected_variant_state_idx = editor_widgets.editor.selected_variant_state_idx;

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

                variant.set_end_zone_idx(0);

                editor_widgets.update_end_zone_idx_choice(&layout_editor.sender);

                editor_widgets.new_variant_state(&layout_editor.sender);
            }

            Message::CloneVariantState => {
                let variant = &mut editor_widgets.editor.layout.get_variants_mut()
                    [editor_widgets.editor.selected_variant_idx];

                let idx = editor_widgets.editor.selected_variant_state_idx;

                variant.clone_zone_vec(idx);

                editor_widgets.new_variant_state(&layout_editor.sender);
            }

            Message::DeleteVariantState => {
                let variant = &mut editor_widgets.editor.layout.get_variants_mut()
                    [editor_widgets.editor.selected_variant_idx];

                let idx = editor_widgets.editor.selected_variant_state_idx;

                variant.delete_zones(idx);

                editor_widgets.delete_variant_state(&layout_editor.sender, None);
            }

            Message::SwapVariantState(swap_direction) => {
                let variant = &mut editor_widgets.editor.layout.get_variants_mut()
                    [editor_widgets.editor.selected_variant_idx];

                let selected_variant_state_idx = editor_widgets.editor.selected_variant_state_idx;

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

                editor_widgets.swap_variant_states(swap_with, &layout_editor.sender);
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
                let split_at: i32 = match editor_widgets.actions.split_at_input.value().parse() {
                    Ok(val) if val > 0 && val < editor_widgets.actions.split_bound_max.unwrap() => {
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

                let selected_variant_state_idx = editor_widgets.editor.selected_variant_state_idx;

                let variant =
                    &mut editor_widgets.editor.layout.get_variants_mut()[selected_variant_idx];

                let other_zone_idx = if let Some(idx) = editor_widgets.editor.selected_zone_idx2 {
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

                editor_widgets.update_end_zone_idx_choice(&layout_editor.sender);

                editor_widgets.update_variant_state_display(
                    selected_variant_idx,
                    selected_variant_state_idx,
                    &layout_editor.sender,
                );
            }

            Message::Swap => {
                let selected_variant_idx = editor_widgets.editor.selected_variant_idx;

                let selected_variant_state_idx = editor_widgets.editor.selected_variant_state_idx;

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
                    &layout_editor.sender,
                );
            }

            Message::Merge => {
                let selected_variant_idx = editor_widgets.editor.selected_variant_idx;

                let selected_variant_state_idx = editor_widgets.editor.selected_variant_state_idx;

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
                    &layout_editor.sender,
                );
            }

            Message::NewVariant => {
                let layout = &editor_widgets.editor.layout;

                let w = layout.get_monitor_rect().w();

                let h = layout.get_monitor_rect().h();

                let new_variant = Variant::new(w, h);

                editor_widgets.add_new_variant(new_variant, &layout_editor.sender);

                layout_editor.sender.send(Message::SelectedVariantChanged(
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

                editor_widgets.add_new_variant(new_variant, &layout_editor.sender);

                layout_editor.sender.send(Message::SelectedVariantChanged(
                    editor_widgets.editor.layout.variants_len() - 1,
                ));

                if !editor_widgets.variant_actions.delete_button.active() {
                    editor_widgets.variant_actions.delete_button.activate();
                }
            }

            Message::DeleteVariant => {
                let selected_variant_idx = editor_widgets.editor.selected_variant_idx;

                let old_default_variant_idx = editor_widgets.editor.layout.default_variant_idx();

                editor_widgets
                    .editor
                    .layout
                    .delete_variant(selected_variant_idx);

                let variants_pack =
                    group::Pack::from_dyn_widget(&editor_widgets.variant_list.child(0).unwrap())
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

                    layout_editor
                        .sender
                        .send(Message::SelectedVariantChanged(selected_variant_idx - 1));
                } else {
                    layout_editor
                        .sender
                        .send(Message::SelectedVariantChanged(selected_variant_idx));
                }

                for i in selected_variant_idx as i32..variants_pack.children() {
                    let b = &mut button::Button::from_dyn_widget(&variants_pack.child(i).unwrap())
                        .unwrap();

                    b.set_label(i.to_string().as_str());

                    b.emit(
                        layout_editor.sender.clone(),
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
                        if selected_variant_idx == editor_widgets.editor.layout.variants_len() - 1 {
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
                    layout_editor.sender.clone(),
                    Message::SelectedVariantChanged(second_idx),
                );

                second_variant_button.emit(
                    layout_editor.sender.clone(),
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

                variant_state_display.insert(&second_variant_state_display_group, first_idx as i32);

                variant_state_display.insert(&first_variant_state_display_group, second_idx as i32);

                editor_widgets.editor.selected_variant_idx = new_idx;

                layout_editor
                    .sender
                    .send(Message::SelectedVariantChanged(new_idx));
            }

            Message::SetVariantAsDefault => {
                let selected_variant_idx = editor_widgets.editor.selected_variant_idx;

                let old_default_variant_idx = editor_widgets.editor.layout.default_variant_idx();

                editor_widgets
                    .editor
                    .layout
                    .set_default_variant_idx(selected_variant_idx);

                editor_widgets
                    .update_default_variant_label(old_default_variant_idx, selected_variant_idx);

                editor_widgets
                    .variant_actions
                    .set_as_default_button
                    .deactivate();
            }

            Message::CancelPreview => {
                editor_widgets.remove_extend_preview();

                editor_widgets.end_behaviour_actions.preview_count = 0;

                editor_widgets
                    .end_behaviour_actions
                    .cancel_preview_button
                    .deactivate();

                editor_widgets.actions.widgets.activate();

                layout_editor.sender.send(Message::SelectedVariantChanged(
                    editor_widgets.editor.selected_variant_idx,
                ));
            }

            Message::PreviewExtend => {
                editor_widgets
                    .end_behaviour_actions
                    .cancel_preview_button
                    .activate();

                editor_widgets.actions.widgets.deactivate();

                layout_editor.window.begin();

                editor_widgets.preview_extend(
                    editor_widgets.editor.selected_variant_idx,
                    true,
                    &layout_editor.sender,
                );

                layout_editor.window.end();
            }

            Message::EndZoneIdxChanged(idx) => {
                if let Some(_) = &editor_widgets.buffers {
                    editor_widgets.remove_extend_preview();

                    editor_widgets.editor.layout.get_variants_mut()
                        [editor_widgets.editor.selected_variant_idx]
                        .set_end_zone_idx(idx);

                    layout_editor.window.begin();

                    for _i in 0..editor_widgets.end_behaviour_actions.preview_count {
                        editor_widgets.preview_extend(
                            editor_widgets.editor.selected_variant_idx,
                            false,
                            &layout_editor.sender,
                        );
                    }

                    layout_editor.window.end();
                } else {
                    editor_widgets.editor.layout.get_variants_mut()
                        [editor_widgets.editor.selected_variant_idx]
                        .set_end_zone_idx(idx);
                }
            }

            Message::SwapEndTilingBehaviour => {
                let variant = &mut editor_widgets.editor.layout.get_variants_mut()
                    [editor_widgets.editor.selected_variant_idx];

                match variant.get_end_tiling_behaviour().to_owned() {
                    EndTilingBehaviour::Directional {
                        direction,
                        from_zones,
                        zone_idx,
                    } => {
                        editor_widgets
                            .end_behaviour_actions
                            .directional
                            .widgets
                            .hide();

                        editor_widgets
                            .end_behaviour_actions
                            .repeating
                            .widgets
                            .show();

                        editor_widgets
                            .end_behaviour_actions
                            .directional
                            .end_tiling_behaviour = EndTilingBehaviour::Directional {
                            direction,
                            from_zones,
                            zone_idx,
                        };

                        variant.set_end_tiling_behaviour(
                            editor_widgets
                                .end_behaviour_actions
                                .repeating
                                .end_tiling_behaviour
                                .to_owned(),
                        );
                    }
                    EndTilingBehaviour::Repeating { splits, zone_idx } => {
                        editor_widgets
                            .end_behaviour_actions
                            .repeating
                            .widgets
                            .hide();

                        editor_widgets
                            .end_behaviour_actions
                            .directional
                            .widgets
                            .show();

                        editor_widgets
                            .end_behaviour_actions
                            .repeating
                            .end_tiling_behaviour =
                            EndTilingBehaviour::Repeating { splits, zone_idx };

                        variant.set_end_tiling_behaviour(
                            editor_widgets
                                .end_behaviour_actions
                                .directional
                                .end_tiling_behaviour
                                .to_owned(),
                        );
                    }
                }
            }

            Message::SwapEndTilingDirection => {
                let new_direction = editor_widgets.editor.layout.get_variants_mut()
                    [editor_widgets.editor.selected_variant_idx]
                    .get_end_tiling_direction()
                    .unwrap()
                    .other();

                if let Some(_) = &editor_widgets.buffers {
                    editor_widgets.remove_extend_preview();

                    editor_widgets.editor.layout.get_variants_mut()
                        [editor_widgets.editor.selected_variant_idx]
                        .set_end_tiling_direction(new_direction);

                    layout_editor.window.begin();

                    for _i in 0..editor_widgets.end_behaviour_actions.preview_count {
                        editor_widgets.preview_extend(
                            editor_widgets.editor.selected_variant_idx,
                            false,
                            &layout_editor.sender,
                        );
                    }

                    layout_editor.window.end();
                } else {
                    editor_widgets.editor.layout.get_variants_mut()
                        [editor_widgets.editor.selected_variant_idx]
                        .set_end_tiling_direction(new_direction);
                }
            }
        }
    }
}
