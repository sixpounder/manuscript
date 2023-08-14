use crate::{models::*, services::DocumentAction};
use adw::subclass::prelude::*;
use glib_macros::Properties;
use gtk::{gdk::RGBA, gio, glib::Sender, prelude::*};
use std::cell::{Cell, RefCell};

#[allow(unused)]
const G_LOG_DOMAIN: &str = "ManuscriptChunkSidePanel";

mod imp {
    use super::*;
    use glib::{ParamSpec, ParamSpecBoxed};
    use once_cell::sync::Lazy;

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

        #[property(get, set, nullable)]
        pub(super) accent: Cell<Option<Color>>,
    }

    impl ManuscriptChunkSidePanel {
        fn accent_rgba(&self) -> Option<RGBA> {
            self.accent.get().map(RGBA::from)
        }

        fn set_accent_rgba(&self, value: Option<RGBA>) {
            self.accent.set(value.map(Color::from));
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
        /// Meshes derived properties with custom defined properties. Custom defined properties
        /// are tipically computed and have no backing variables
        fn properties() -> &'static [gtk::glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                let derived: &'static [gtk::glib::ParamSpec] =
                    ManuscriptChunkSidePanel::derived_properties();

                let mut props: Vec<ParamSpec> = vec![ParamSpecBoxed::builder::<
                    Option<gtk::gdk::RGBA>,
                >("accent-rgba")
                .readwrite()
                .build()];

                let mut derived = derived.to_vec();
                derived.append(&mut props);
                derived
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, id: usize, pspec: &ParamSpec) -> glib::Value {
            match pspec.name() {
                "accent-rgba" => self.accent_rgba().to_value(),
                _ => self.derived_property(id, pspec),
            }
        }

        fn set_property(&self, id: usize, value: &glib::Value, pspec: &ParamSpec) {
            match pspec.name() {
                "accent-rgba" => self.set_accent_rgba(
                    value
                        .get::<Option<RGBA>>()
                        .expect("Expected Option<RGBA>, got something else"),
                ),
                _ => self.derived_set_property(id, value, pspec),
            }
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
        obj.imp().accent.set(chunk.accent());
        obj.set_locked(chunk.locked());
        obj.notify("accent-rgba");
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

        self.connect_accent_notify(glib::clone!(@weak self as this => move |widget| {
            widget.send_update(move |chunk| {
                chunk.set_accent(this.accent()).unwrap();
            })
        }));
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
