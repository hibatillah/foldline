use super::{charts::disk_bar_track, format::percent_text, layout::NETWORK_ROW_PITCH, rows::disk_row_tops, *};
use crate::gfx::Rect;

#[test]
fn the_page_samples_once_a_second() {
    assert_eq!(SAMPLE_INTERVAL_MILLISECONDS, 1000);
}

#[test]
fn the_stats_page_is_opt_in_and_has_no_collapsed_state() {
    let page = StatsPage::new();
    assert!(page.is_opt_in());
    assert_eq!(page.priority(Mode::Resting), None);
    assert!(page.priority(Mode::Expanded).is_some());
}

#[test]
fn every_card_is_132_by_86_with_8_between_them() {
    let width = expanded_width(&tiles_for(true));
    let rects = card_rects(Rect::new(0.0, 0.0, width, 86.0), 1.0, 5);
    assert_eq!(rects.len(), 5);
    for rect in &rects {
        assert_eq!((rect.width(), rect.height()), (132.0, 86.0));
    }
    assert_eq!(rects[1].left - rects[0].right, TILE_GAP);
    assert_eq!(rects[4].right, width);
}

#[test]
fn the_page_is_692_px_wide_with_a_gpu_and_560_without_before_the_host_padding() {
    assert_eq!(expanded_width(&tiles_for(true)), 692.0);
    assert_eq!(expanded_width(&tiles_for(false)), 4.0 * 132.0 + 3.0 * 8.0);
    // The host adds 16 px at each side: 724 px with a GPU. Without one the formula gives 584 px; the board says 592.
    assert_eq!(expanded_width(&tiles_for(true)) + 2.0 * 16.0, 724.0);
}

#[test]
fn a_missing_gpu_leaves_every_other_card_its_width_and_the_page_narrower() {
    assert_eq!(expanded_width(&tiles_for(true)) - expanded_width(&tiles_for(false)), CARD_WIDTH + TILE_GAP);
}

#[test]
fn a_saved_choice_reads_as_a_list_of_names() {
    assert_eq!(choice_list("G:, C:,,", &[',']), ["G:", "C:"]);
    assert!(choice_list("", &[',']).is_empty());
}

#[test]
fn the_gpu_choice_accepts_the_published_bar_and_the_older_comma() {
    let separators = [GPU_SEPARATOR, ','];
    assert_eq!(choice_list("RTX 4050|Radeon 760M", &separators), ["RTX 4050", "Radeon 760M"]);
    assert_eq!(choice_list("RTX 4050", &separators), ["RTX 4050"]);
    assert_eq!(choice_list("RTX 4050,Radeon 760M", &separators), ["RTX 4050", "Radeon 760M"]);
    assert_eq!(choice_list("NVIDIA, Inc. GPU|Radeon", &separators), ["NVIDIA, Inc. GPU", "Radeon"]);
}

#[test]
fn the_gpu_tile_appears_only_when_there_is_a_gpu() {
    assert_eq!(tiles_for(true), [Tile::Cpu, Tile::Memory, Tile::Gpus, Tile::Network, Tile::Disks]);
    assert_eq!(tiles_for(false), [Tile::Cpu, Tile::Memory, Tile::Network, Tile::Disks]);
}

#[test]
fn the_network_rows_sit_at_20_and_43_with_the_kind_at_66() {
    let card = Rect::new(0.0, 0.0, 132.0, 86.0);
    let download = network_row_area(card, 1.0, 0);
    let upload = network_row_area(card, 1.0, 1);
    assert_eq!((download.top, upload.top), (20.0, 43.0));
    assert_eq!(upload.top - download.top, NETWORK_ROW_PITCH);
    assert!(upload.bottom <= network_kind_area(card, 1.0).top, "the network kind stays below the rows");
    assert_eq!(network_kind_area(card, 1.0).top, 66.0);
    assert!(network_kind_area(card, 1.0).bottom <= card.bottom);
}

#[test]
fn the_texts_start_at_the_card_positions_of_the_board() {
    let card = Rect::new(0.0, 0.0, 132.0, 86.0);
    assert_eq!((label_area(card, 1.0).left, label_area(card, 1.0).top), (12.0, 8.0));
    assert_eq!(value_area(card, 1.0).top, 19.0);
    assert_eq!(sub_area(card, 1.0).top, 64.0);
    assert_eq!(label_area(card, 1.0).right, 120.0);
}

#[test]
fn disk_rows_sit_at_32_then_27_46_then_24_40_56() {
    assert_eq!(disk_row_tops(0), &[] as &[f32]);
    assert_eq!(disk_row_tops(1), [32.0]);
    assert_eq!(disk_row_tops(2), [27.0, 46.0]);
    assert_eq!(disk_row_tops(3), [24.0, 40.0, 56.0]);
    let card = Rect::new(0.0, 0.0, 132.0, 86.0);
    for count in 1..=MAX_DISKS {
        for &top in disk_row_tops(count) {
            let track = disk_bar_track(card, 1.0, top);
            assert!(track.right <= card.right - TILE_PADDING_X && track.bottom <= card.bottom - 4.0);
        }
    }
}

#[test]
fn a_reading_sets_the_bar_loads_between_zero_and_one() {
    let mut page = StatsPage::sample();
    let readout = page.readout.as_ref().unwrap();
    assert_eq!(readout.cpu_load, 0.75);
    assert_eq!(readout.memory_load, 0.51);
    assert_eq!(readout.gpus[0].load(), 0.65);
    assert_eq!(readout.gpus[1].load(), 0.0);
    let sample = page.latest.clone().unwrap();
    page.set_latest(Sample { cpu_percent: None, memory_percent: 250.0, ..sample });
    let readout = page.readout.as_ref().unwrap();
    assert_eq!((readout.cpu_load, readout.memory_load), (0.0, 1.0));
}

#[test]
fn the_stored_words_follow_the_latest_reading() {
    let page = StatsPage::sample();
    let readout = page.readout.as_ref().expect("readout");
    let sample = page.latest.as_ref().expect("reading");
    assert_eq!(readout.cpu_text, percent_text(sample.cpu_percent.unwrap()));
    assert_eq!(readout.memory_percent_text, percent_text(sample.memory_percent));
    assert_eq!(readout.memory_pair_text, "8.1 of 16 GB");
    assert_eq!(readout.download_text, "\u{2193} 2.4 MB/s");
    assert_eq!(readout.upload_text, "\u{2191} 120 KB/s");
    assert_eq!(readout.threads_text, format!("{} threads", sample.logical_processors));
    let disks: Vec<(&str, &str)> =
        readout.disks.iter().map(|row| (row.meter.name.as_str(), row.percent_text.as_str())).collect();
    assert_eq!(disks, [("C:", "90%"), ("G:", "79%")]);
    assert!(readout.gpus[0].percent_text.ends_with('%'));
}

#[test]
fn a_missing_figure_is_stored_as_a_dash() {
    let mut page = StatsPage::sample();
    let sample = page.latest.clone().unwrap();
    page.set_latest(Sample {
        cpu_percent: None,
        download_bytes_per_second: None,
        gpus: vec![Meter { name: "RTX 4050".to_owned(), capacity: None, percent: None }],
        ..sample
    });
    let readout = page.readout.as_ref().unwrap();
    assert_eq!(readout.cpu_text, "\u{2013}");
    assert_eq!(readout.download_text, "\u{2193} \u{2013}");
    assert_eq!(readout.gpus[0].percent_text, "\u{2013}");
}

#[test]
fn a_saved_choice_rebuilds_the_rows_and_the_cards() {
    let mut page = StatsPage::sample();
    assert_eq!(page.tiles().len(), 5);
    page.disk_choice = Some(vec!["G:".to_owned()]);
    page.gpu_choice = Some(Vec::new());
    page.rebuild_readout();
    let readout = page.readout.as_ref().unwrap();
    assert_eq!(readout.disks.len(), 1);
    assert_eq!(readout.disks[0].meter.name, "G:");
    assert!(readout.gpus.is_empty());
    assert_eq!(page.tiles(), [Tile::Cpu, Tile::Memory, Tile::Network, Tile::Disks]);
}

#[test]
fn a_chosen_disk_that_is_gone_is_skipped_and_the_others_keep_their_order() {
    let mut page = StatsPage::sample();
    page.disk_choice = Some(vec!["X:".to_owned(), "G:".to_owned()]);
    page.rebuild_readout();
    let names: Vec<&str> = page.readout.as_ref().unwrap().disks.iter().map(|row| row.meter.name.as_str()).collect();
    assert_eq!(names, ["G:"]);
}
