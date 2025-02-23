use directories::UserDirs;

use himewm_layout::*;

#[test]
fn create_vertical_stack() {
    let mut layout_group = Layout::new(1920, 1200);

    let mut idx = 0;

    let mut current_variant = &mut layout_group.get_variants_mut()[idx];

    current_variant.new_zone_vec(1920, 1200);

    for n in 1..=5 {
        current_variant.split(
            1,
            0,
            SplitDirection::Horizontal((1920 as f64 * (n as f64 / 6 as f64)) as i32),
        );

        if n < 3 {
            current_variant.swap_zones(1, 0, 1);
        }

        if n != 5 {
            layout_group.clone_variant(layout_group.default_variant_idx());

            idx += 1;

            current_variant = &mut layout_group.get_variants_mut()[idx];

            current_variant.merge_zones(1, 0, 1);
        }
    }

    layout_group.set_default_variant_idx(2);

    export_layout_to_downloads(&layout_group, "vertical_stack").unwrap();
}

#[test]
fn create_spiral() {
    let mut layout_group = Layout::new(1920, 1200);

    let mut idx = 0;

    let mut current_variant = &mut layout_group.get_variants_mut()[idx];

    current_variant.set_end_tiling_behaviour(EndTilingBehaviour::default_repeating());

    current_variant.add_repeating_split(Direction::Vertical, 0.5, 4, false);
    current_variant.add_repeating_split(Direction::Horizontal, 0.5, 1, true);
    current_variant.add_repeating_split(Direction::Vertical, 0.5, 2, true);
    current_variant.add_repeating_split(Direction::Horizontal, 0.5, 3, false);

    current_variant.new_zone_vec(1920, 1200);

    for n in 1..=5 {
        current_variant.split(
            1,
            0,
            SplitDirection::Horizontal((1920 as f64 * (n as f64 / 6 as f64)) as i32),
        );

        if n < 3 {
            current_variant.swap_zones(1, 0, 1);
        }

        if n != 5 {
            layout_group.clone_variant(layout_group.default_variant_idx());

            idx += 1;

            current_variant = &mut layout_group.get_variants_mut()[idx];

            current_variant.merge_zones(1, 0, 1);
        }
    }

    layout_group.set_default_variant_idx(2);

    export_layout_to_downloads(&layout_group, "spiral").unwrap();
}

#[test]
fn create_horizontal_stack_starting_at_3() {
    let mut layout_group = Layout::new(1920, 1200);

    let mut idx = 0;

    let mut current_variant = &mut layout_group.get_variants_mut()[idx];

    current_variant.set_end_tiling_start_from(3);

    current_variant.set_end_tiling_direction(Direction::Horizontal);

    current_variant.new_zone_vec(1920, 1200);

    for n in 1..=5 {
        match n {
            1 => {
                current_variant.split(
                    1,
                    0,
                    SplitDirection::Horizontal((1920 as f64 * (n as f64 / 6 as f64)) as i32),
                );

                current_variant.swap_zones(1, 0, 1);
            }

            2 => {
                current_variant.merge_and_split_zones(
                    1,
                    0,
                    1,
                    SplitDirection::Horizontal((1920 as f64 * (n as f64 / 6 as f64)) as i32),
                );

                current_variant.swap_zones(1, 0, 1);
            }

            _ => {
                current_variant.merge_and_split_zones(
                    1,
                    0,
                    1,
                    SplitDirection::Horizontal((1920 as f64 * (n as f64 / 6 as f64)) as i32),
                );
            }
        }

        if n != 5 {
            layout_group.clone_variant(layout_group.default_variant_idx());

            idx += 1;

            current_variant = &mut layout_group.get_variants_mut()[idx];
        }
    }

    for variant in layout_group.get_variants_mut() {
        variant.clone_zone_vec(1);

        variant.split(2, 1, SplitDirection::Vertical(600));

        variant.new_zone_vec(1920, 1200);

        variant.split(3, 0, SplitDirection::Vertical(600));
    }

    layout_group.set_default_variant_idx(2);

    export_layout_to_downloads(&layout_group, "horizontal_stack_starting_at_3").unwrap();
}

fn export_layout_to_downloads(layout: &Layout, name: &str) -> std::io::Result<()> {
    let path = UserDirs::new()
        .unwrap()
        .download_dir()
        .unwrap()
        .join(std::path::Path::new(name).with_extension("json"));

    let output_file = std::fs::File::create_new(path)?;

    serde_json::to_writer_pretty(output_file, layout).unwrap();

    return Ok(());
}
