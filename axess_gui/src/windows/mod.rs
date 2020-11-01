use nwg::InsertListViewItem;

pub mod connect;
pub mod main;
pub mod common;
pub mod status_bar;
pub mod keyboard;
pub mod settings;

fn update_list(list: &nwg::ListView, items: &[String], selected_item: Option<usize>) {
    let max_str_len = 256;

    if list.len() != items.len() {
        list.clear();
        list.insert_items(items);
    } else {
        for i in 0..items.len() {
            if let Some(existing) = list.item(i, 0, max_str_len) {
                if existing.text != items[i] {
                    list.update_item(i, InsertListViewItem {
                        index: Some(i as i32),
                        column_index: 0,
                        text: Some(items[i].clone()),
                        image: None
                    });
                }
            }

        }
    }

    if let Some(selected_item) = selected_item {
        let s = list.selected_items();
        for idx in s {
            list.select_item(idx, false);
        }

        list.select_item(selected_item, true);
    }
}