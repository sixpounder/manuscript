use super::prelude::*;
use crate::services::DocumentAction;
use adw::subclass::prelude::*;
use glib;
use glib::Sender;
use glib_macros::Properties;
use gtk::{gio, prelude::*};
use std::cell::{Cell, RefCell};

mod imp {
    use super::*;
    use glib::ParamSpec;
    // use once_cell::sync::Lazy;

    #[derive(Properties, Default, gtk::CompositeTemplate)]
    #[properties(wrapper_type = super::ManuscriptEditorView)]
    #[template(resource = "/io/sixpounder/Manuscript/editors/editor_view.ui")]
    pub struct ManuscriptEditorView {
        pub(super) sender: RefCell<Option<Sender<DocumentAction>>>,

        #[template_child]
        pub(super) container: TemplateChild<gtk::Paned>,

        #[property(get, set)]
        pub(super) child: RefCell<Option<gtk::Widget>>,

        #[property(get, set)]
        pub(super) side_panel: RefCell<Option<gtk::Widget>>,

        #[property(get, set)]
        pub(super) side_panel_visible: Cell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ManuscriptEditorView {
        const NAME: &'static str = "ManuscriptEditorView";
        type Type = super::ManuscriptEditorView;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.set_layout_manager_type::<gtk::BinLayout>();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ManuscriptEditorView {
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

    impl WidgetImpl for ManuscriptEditorView {}
}

glib::wrapper! {
    pub struct ManuscriptEditorView(ObjectSubclass<imp::ManuscriptEditorView>)
        @extends gtk::Widget, @implements gio::ActionGroup, gio::ActionMap;
}

impl ManuscriptEditorView {
    pub fn new(editor: impl EditorWidgetProtocol) -> Self {
        let obj: Self = glib::Object::builder().build();

        if let Some(editor) = editor.editor_widget() {
            obj.set_child(&editor);
        }

        if let Some(props_panel) = editor.side_panel_widget() {
            props_panel.set_visible(obj.side_panel_visible());
            obj.set_side_panel(&props_panel);
            obj.connect_side_panel_visible_notify(|this| {
                this.side_panel()
                    .unwrap()
                    .set_visible(this.side_panel_visible());
            });
        }

        *obj.imp().sender.borrow_mut() = editor.document_action_sender();

        obj
    }

    pub fn container(&self) -> gtk::Paned {
        self.imp().container.get()
    }
}
