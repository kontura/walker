use gtk4::glib;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use std::cell::{Cell, RefCell};

mod imp {
    use super::*;
    use gtk4::graphene;

    #[derive(Default)]
    pub struct ProgressRow {
        pub color: RefCell<Option<gtk4::gdk::RGBA>>,
        pub fraction: Cell<f64>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ProgressRow {
        const NAME: &'static str = "ProgressRow";
        type Type = super::ProgressRow;
        type ParentType = gtk4::Box;
    }

    impl ObjectImpl for ProgressRow {
        fn properties() -> &'static [glib::ParamSpec] {
            use std::sync::LazyLock;
            static PROPERTIES: LazyLock<Vec<glib::ParamSpec>> = LazyLock::new(|| {
                vec![
                    glib::ParamSpecString::builder("color").build(),
                    glib::ParamSpecDouble::builder("fraction")
                        .minimum(0.0)
                        .maximum(1.0)
                        .build(),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            match pspec.name() {
                "color" => {
                    let val = value.get::<String>().unwrap();
                    *self.color.borrow_mut() = val.parse().ok();
                }
                "fraction" => {
                    self.fraction.set(value.get::<f64>().unwrap());
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "color" => {
                    let c = self.color.borrow();
                    match *c {
                        Some(rgba) => rgba.to_string().to_value(),
                        None => String::new().to_value(),
                    }
                }
                "fraction" => self.fraction.get().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for ProgressRow {
        fn snapshot(&self, snapshot: &gtk4::Snapshot) {
            let widget = self.obj();
            let width = widget.width() as f32;
            let height = widget.height() as f32;

            if width > 0.0 && height > 0.0 {
                let padding = widget.style_context().padding();
                let left = padding.left() as f32;
                let right = padding.right() as f32;
                let top = padding.top() as f32;
                let bottom = padding.bottom() as f32;

                let content = graphene::Rect::new(
                    0.0 - left,
                    0.0 - top,
                    width + left + right,
                    height + top + bottom,
                );

                let corner_radius = 10.0;
                let rounded = gtk4::gsk::RoundedRect::from_rect(content, corner_radius);
                snapshot.push_rounded_clip(&rounded);

                let fraction = self.fraction.get();
                if fraction > 0.0 {
                    if let Some(color) = *self.color.borrow() {
                        let frac_rect = graphene::Rect::new(
                            content.x(),
                            content.y(),
                            content.width() * fraction as f32,
                            content.height(),
                        );
                        snapshot.append_color(&color, &frac_rect);
                    }
                }

                snapshot.pop();
            }

            let mut child = widget.first_child();
            while let Some(ref c) = child {
                widget.snapshot_child(c, snapshot);
                child = c.next_sibling();
            }
        }
    }

    impl BoxImpl for ProgressRow {}
}

glib::wrapper! {
    pub struct ProgressRow(ObjectSubclass<imp::ProgressRow>)
        @extends gtk4::Box, gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable;
}
