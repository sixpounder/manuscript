use crate::{models::*, services::DocumentAction};
use adw::subclass::prelude::*;
use glib_macros::Properties;
use gtk::{gdk::RGBA, gio, glib::Sender, prelude::*};
use std::cell::{Cell, RefCell};

#[allow(unused)]
const G_LOG_DOMAIN: &str = "ManuscriptChunkSidePanel";

mod imp {
    use super::*;
    use glib::ParamSpec;

    #[derive(Properties, Default, gtk::CompositeTemplate)]
    #[properties(wrapper_type = super::ManuscriptChunkSidePanel)]
    #[template(resource = "/io/sixpounder/Manuscript/editors/chunk_side_panel.ui")]
    pub struct ManuscriptChunkSidePanel {
        pub(super) sender: RefCell<Option<Sender<DocumentAction>>>,

        #[template_child]
        pub(super) priority_adjustment: TemplateChild<gtk::Adjustment>,

        #[property(get, set)]
        pub(super) chunk_id: RefCell<String>,

        #[property(get, set)]
        pub(super) include_in_compilation: Cell<bool>,

        #[property(get, set)]
        pub(super) locked: Cell<bool>,

        #[property(name = "accent", get, set)]
        pub(super) accent: RefCell<Option<Color>>,

        #[allow(dead_code)]
        #[property(type = Option<RGBA>, name = "accent-rgba", get = Self::accent_rgba, set = Self::set_accent_rgba)]
        accent_rgba: Option<Color>,
    }

    impl ManuscriptChunkSidePanel {
        fn accent_rgba(&self) -> Option<RGBA> {
            let owned = self.accent.borrow().to_owned();
            owned.map(|a| RGBA::from(a))
        }

        fn set_accent_rgba(&self, value: Option<RGBA>) {
            let mapped = value.map(|rgba| Color::from(rgba));
            *self.accent.borrow_mut() = mapped;
            self.obj().notify_accent();
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ManuscriptChunkSidePanel {
        const NAME: &'static str = "ManuscriptChunkSidePanel";
        type Type = super::ManuscriptChunkSidePanel;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.set_layout_manager_type::<gtk::BinLayout>();
            klass.bind_template_instance_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ManuscriptChunkSidePanel {
        fn properties() -> &'static [gtk::glib::ParamSpec] {
            Self::derived_properties()
        }

        fn property(&self, id: usize, pspec: &ParamSpec) -> glib::Value {
            self.derived_property(id, pspec)
        }

        fn set_property(&self, id: usize, value: &glib::Value, pspec: &ParamSpec) {
            self.derived_set_property(id, value, pspec)
        }
    }

    impl WidgetImpl for ManuscriptChunkSidePanel {}
}

glib::wrapper! {
    pub struct ManuscriptChunkSidePanel(ObjectSubclass<imp::ManuscriptChunkSidePanel>)
        @extends gtk::Widget, @implements gio::ActionGroup, gio::ActionMap;
}

impl ManuscriptChunkSidePanel {
    pub fn new(chunk: &dyn DocumentChunk, sender: Option<Sender<DocumentAction>>) -> Self {
        let obj: Self = glib::Object::new();
        *obj.imp().sender.borrow_mut() = sender;
        obj.set_chunk_id(chunk.id());
        obj.set_include_in_compilation(chunk.include_in_compilation());
        *obj.imp().accent.borrow_mut() = chunk.accent();
        obj.set_locked(chunk.locked());
        obj.notify_accent_rgba();
        obj.connect_events();

        obj
    }

    fn connect_events(&self) {
        self.connect_include_in_compilation_notify(
            glib::clone!(@weak self as this => move |widget| {
                widget.send_update(move |chunk| {
                    chunk.set_include_in_compilation(this.include_in_compilation()).expect("Failed to set include_in_compilation");
                });
            })
        );

        self.connect_locked_notify(glib::clone!(@weak self as this => move |widget| {
            widget.send_update(move |chunk| {
                chunk.set_locked(this.locked())
            });
        }));

        self.connect_accent_notify(
            glib::clone!(@weak self as this => move |widget| {
                widget.send_update(move |chunk| {
                    chunk.set_accent(this.accent()).unwrap();
                })
            })
        );
    }

    fn send_update<F>(&self, f: F)
    where
        F: FnOnce(&mut dyn DocumentChunk) + 'static,
    {
        let sender = self.imp().sender.borrow();
        let sender = sender.as_ref().unwrap();
        sender
            .send(DocumentAction::UpdateChunkWith(
                self.chunk_id(),
                Box::new(f),
            ))
            .expect("Failed to send update");
    }
}

#[gtk::template_callbacks]
impl ManuscriptChunkSidePanel {}

