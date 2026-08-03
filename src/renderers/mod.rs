use crate::config::get_config;
use crate::protos::generated_proto::query::query_response::Item;
use crate::providers::PROVIDERS;
use crate::state::{get_dmenu_current, is_grid, is_hide_qa, set_error};
use crate::theme::{Theme, get_css_color, with_themes};
use crate::ui::window::{quit, with_window};
use gtk4::gdk::{ContentProvider, RGBA};
use gtk4::gio::File;
use gtk4::gio::prelude::FileExt;
use gtk4::prelude::{DrawingAreaExtManual, ListItemExt, WidgetExt};
use gtk4::{Widget, Box, Builder, DragSource, Label, ListItem, glib, Overlay, DrawingArea};
use std::path::Path;

pub fn create_item(list_item: &ListItem, item: &Item, theme: &Theme) {
    let mut b = Builder::new();

    let _ = if !is_grid() {
        b.add_from_string(
            theme
                .items
                .get(&item.provider)
                .unwrap_or_else(|| panic!("failed to get item layout: {}", &item.provider)),
        )
    } else {
        b.add_from_string(
            theme
                .grid_items
                .get(&item.provider)
                .unwrap_or_else(|| panic!("failed to get item grid layout: {}", &item.provider)),
        )
    };

    let itembox: Box = match b.object("ItemBox") {
        Some(w) => w,
        None => {
            set_error("Theme: missing 'ItemBox' object".to_string());

            b = Builder::new();

            with_themes(|t| {
                let theme = t.get("default").unwrap();
                let _ = b.add_from_string(
                    theme
                        .items
                        .get(&item.provider)
                        .expect("failed to get item layout"),
                );
            });

            b.object("ItemBox").unwrap()
        }
    };
    let widget: Widget;

    if let Some(overlay) = b.object::<Overlay>("ItemOverlay") {
        overlay.set_measure_overlay(&itembox, true);
        widget = overlay.into();
    } else {
        widget = itembox.into();
    }

    widget.add_css_class(&item.provider.replace("menus:", "menus-"));
    item.state
        .iter()
        .filter(|i| !i.is_empty())
        .for_each(|i| widget.add_css_class(i));

    if get_dmenu_current() != 0 && get_dmenu_current() as u32 == list_item.position() + 1 {
        widget.add_css_class("current");
    }

    list_item.set_child(Some(&widget));

    if Path::new(&item.text).is_absolute() {
        widget.add_controller(create_drag_source(&item.text));
    }


    let p = PROVIDERS.get().unwrap().get(&item.provider).unwrap();

    if let Some(progressbar) = b.object::<DrawingArea>("ProgressBar") {
        let fraction = p.progress_transformer(item);
        let color = get_css_color("progressbar_color")
            .unwrap_or(RGBA::new(0.0, 0.0, 0.0, 0.2));
        progressbar.set_draw_func(move |_, cr, width, height| {
            let progress_width = width as f64 * fraction;
            cr.set_source_rgba(
                color.red() as f64,
                color.green() as f64,
                color.blue() as f64,
                color.alpha() as f64,
            );
            cr.rectangle(0.0, 0.0, progress_width, height as f64);
            let _ = cr.fill();
        });
    }

    if let Some(text) = b.object::<Label>("ItemText") {
        p.text_transformer(item, &text);
    }

    if let Some(text) = b.object::<Label>("ItemSubtext") {
        p.subtext_transformer(item, &text);
    }

    p.image_transformer(&b, list_item, item);

    if let Some(text) = b.object::<Label>("QuickActivation") {
        if is_hide_qa() || get_config().hide_quick_activation {
            text.set_visible(false);
            return;
        }

        if let Some(qa) = &get_config().keybinds.quick_activate {
            let i = list_item.position();

            if let Some(val) = qa.get(i as usize) {
                text.set_label(val);
            } else {
                text.set_visible(false);
            }
        }
    }
}

pub fn create_drag_source(text: &str) -> DragSource {
    let drag_source = DragSource::new();
    let text = text.to_string();

    drag_source.connect_prepare(move |_, _, _| {
        let file = File::for_path(&text);
        let uri_string = format!("{}\n", file.uri());
        let b = glib::Bytes::from(uri_string.as_bytes());

        let cp = ContentProvider::for_bytes("text/uri-list", &b);
        Some(cp)
    });

    drag_source.connect_drag_begin(|_, _| with_window(|w| w.window.set_visible(false)));
    drag_source.connect_drag_end(|_, _, _| with_window(|w| quit(&w.app, false)));
    drag_source
}
