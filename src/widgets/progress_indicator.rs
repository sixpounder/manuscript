use adw::subclass::prelude::*;
use gtk::{gio, glib, prelude::*};
use std::cell::Cell;

mod imp {
    use super::*;
    use glib::{ParamFlags, ParamSpec, ParamSpecBoolean, ParamSpecInt};
    use gtk::gdk::RGBA;
    use once_cell::sync::Lazy;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/sixpounder/Manuscript/progress_indicator.ui")]
    pub struct ManuscriptProgressIndicator {
        pub(super) value: Cell<i32>,
        pub(super) min: Cell<i32>,
        pub(super) max: Cell<i32>,
        pub(super) show_label: Cell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ManuscriptProgressIndicator {
        const NAME: &'static str = "ManuscriptProgressIndicator";
        type Type = super::ManuscriptProgressIndicator;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.set_layout_manager_type::<gtk::BinLayout>();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ManuscriptProgressIndicator {
        fn constructed(&self) {
            self.parent_constructed();
        }
        fn properties() -> &'static [gtk::glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecInt::new("value", "", "", 0, i32::MAX, 0, ParamFlags::READWRITE),
                    ParamSpecInt::new("min", "", "", 0, i32::MAX, 0, ParamFlags::READWRITE),
                    ParamSpecInt::new("max", "", "", 0, i32::MAX, 0, ParamFlags::READWRITE),
                    ParamSpecBoolean::new("show-label", "", "", true, ParamFlags::READWRITE),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> glib::Value {
            let obj = self.obj();
            match pspec.name() {
                "value" => obj.value().to_value(),
                "min" => obj.minimum().to_value(),
                "max" => obj.maximum().to_value(),
                "show-label" => obj.show_label().to_value(),
                _ => unimplemented!(),
            }
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &ParamSpec) {
            let obj = self.obj();
            match pspec.name() {
                "value" => obj.set_value(value.get::<i32>().unwrap()),
                "min" => obj.set_minimum(value.get::<i32>().unwrap()),
                "max" => obj.set_maximum(value.get::<i32>().unwrap()),
                "show-label" => obj.set_show_label(value.get::<bool>().unwrap()),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for ManuscriptProgressIndicator {
        fn snapshot(&self, snapshot: &gtk::Snapshot) {
            let obj = self.obj();
            let widget_bounds =
                gtk::graphene::Rect::new(0.0, 0.0, obj.width() as f32, obj.height() as f32);

            let mut color: RGBA = obj.style_context().color();
            color.set_alpha(0.8);

            let baseline = widget_bounds.height() / 2.0;
            let pad_left = 10.0;
            let pad_right = match obj.show_label() {
                true => 60.0,
                false => 10.0,
            };

            let available_space_w = widget_bounds.width() - pad_left - pad_right;
            let w_amount_percent = (available_space_w / 100.0) * obj.progress() as f32;
            let target_len = (available_space_w / 100.0) * w_amount_percent;

            // Create a cairo context
            let cr = snapshot.append_cairo(&widget_bounds);

            cr.set_source_rgba(
                color.red().into(),
                color.green().into(),
                color.blue().into(),
                color.alpha().into(),
            );

            // Set progress line properties
            cr.set_line_cap(gtk::cairo::LineCap::Round);
            cr.set_line_width(10.0);

            // Move to the start of the left padding point, draw the line
            cr.move_to(pad_left.into(), baseline.into());
            cr.line_to((target_len + pad_left).into(), baseline.into());
            cr.stroke().expect("Failed to draw progress");

            // Draw the progress label if needed
            if obj.show_label() {
                cr.move_to(
                    (widget_bounds.width() - pad_right).into(),
                    (baseline + 5.0).into(),
                );
                cr.set_font_size(16.0);
                cr.select_font_face(
                    "Sans",
                    gtk::cairo::FontSlant::Normal,
                    gtk::cairo::FontWeight::Normal,
                );
                cr.show_text(format!("{}%", obj.progress() as u64).as_str())
                    .expect("Failed to show progress text");
            }
        }
    }
}

glib::wrapper! {
    pub struct ManuscriptProgressIndicator(ObjectSubclass<imp::ManuscriptProgressIndicator>)
        @extends gtk::Widget, @implements gio::ActionGroup, gio::ActionMap;
}

impl Default for ManuscriptProgressIndicator {
    fn default() -> Self {
        Self::new()
    }
}

impl ManuscriptProgressIndicator {
    pub fn new() -> Self {
        glib::Object::new(&[])
    }

    pub fn value(&self) -> i32 {
        self.imp().value.get()
    }

    pub fn set_value(&self, value: i32) {
        self.imp().value.set(value);
        self.queue_draw();
    }

    pub fn minimum(&self) -> i32 {
        self.imp().min.get()
    }

    pub fn set_minimum(&self, value: i32) {
        self.imp().min.set(value);
        self.queue_draw();
    }

    pub fn maximum(&self) -> i32 {
        self.imp().max.get()
    }

    pub fn set_maximum(&self, value: i32) {
        self.imp().max.set(value);
        self.queue_draw();
    }

    pub fn show_label(&self) -> bool {
        self.imp().show_label.get()
    }

    pub fn set_show_label(&self, value: bool) {
        self.imp().show_label.set(value);
        self.queue_draw();
    }

    pub fn progress(&self) -> i32 {
        let min = self.minimum();
        let max = self.maximum();
        let value = self.value();

        if max > min {
            ((value - min) * 100) / (max - min)
        } else {
            0
        }
    }
}
