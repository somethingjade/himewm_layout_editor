use himewm_layout::*;

#[test]
fn show_layout() {
    let mut layout_group = Layout::new(1920, 1200);

    let idx = layout_group.default_variant_idx();

    let variant = &mut layout_group.get_variants_mut()[idx];

    variant.new_zone_vec(1920, 1200);

    variant.split(1, 0, SplitDirection::Horizontal(960));

    variant.clone_zone_vec(1);

    variant.split(2, 1, SplitDirection::Vertical(600));

    layout_group.clone_variant(layout_group.default_variant_idx());

    let new_variant = &mut layout_group.get_variants_mut()[1];

    for _i in 0..2 {
        new_variant.delete_zones(1);
    }

    new_variant.new_zone_vec(1920, 1200);

    new_variant.split(1, 0, SplitDirection::Horizontal(960));

    new_variant.new_zone_vec(1920, 1200);

    new_variant.split(2, 0, SplitDirection::Vertical(600));
    new_variant.split(2, 1, SplitDirection::Horizontal(960));

    new_variant.clone_zone_vec(2);
    new_variant.split(3, 0, SplitDirection::Horizontal(1280));

    new_variant.clone_zone_vec(3);
    new_variant.split(4, 0, SplitDirection::Horizontal(640));
    new_variant.merge_and_split_zones(4, 2, 1, SplitDirection::Horizontal(1440));
    let mut gui = himewm_layout_editor::LayoutEditorGUI::create();

    gui.edit_layout(layout_group);

    gui.run();
}
