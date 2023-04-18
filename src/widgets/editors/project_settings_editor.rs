use super::prelude::EditorWidgetProtocol;
use crate::{
    models::{DocumentManifest, TextMetricSize},
    services::{i18n::i18n, DocumentAction},
};
use adw::subclass::prelude::*;
use glib_macros::Properties;
use gtk::{gio, glib::Sender, prelude::*};
use std::cell::{Cell, RefCell};

#[allow(unused)]
const G_LOG_DOMAIN: &str = "ManuscriptProjectSettings";

mod imp {
    use super::*;
    use glib::ParamSpec;

    #[derive(Properties, gtk::CompositeTemplate)]
    #[properties(wrapper_type = super::ManuscriptProjectSettingsEditor)]
    #[template(resource = "/io/sixpounder/Manuscript/editors/project_settings_editor.ui")]
    pub struct ManuscriptProjectSettingsEditor {
        pub(super) sender: RefCell<Option<Sender<DocumentAction>>>,

        #[template_child]
        pub(super) project_title_entry: TemplateChild<adw::EntryRow>,

        #[template_child]
        pub(super) author_entry: TemplateChild<adw::EntryRow>,

        #[template_child]
        pub(super) paragraph_spacing_entry: TemplateChild<adw::ComboRow>,

        #[template_child]
        pub(super) line_height_entry: TemplateChild<adw::ComboRow>,

        #[property(get, set)]
        pub(super) heading: RefCell<String>,

        #[property(get, set)]
        pub(super) title: RefCell<String>,

        #[property(get, set)]
        pub(super) author: RefCell<String>,

        #[property(type = TextMetricSize, get = Self::get_paragraph_spacing, set = Self::set_paragraph_spacing)]
        #[property(type = u32, name = "paragraph-spacing-numeric", get)]
        #[property(type = u32, name = "paragraph-spacing-selected-index", get = Self::get_paragraph_spacing_selected_index, set = Self::set_paragraph_spacing_selected_index)]
        pub(super) paragraph_spacing: Cell<TextMetricSize>,

        #[property(type = TextMetricSize, get = Self::get_line_height, set = Self::set_line_height)]
        #[property(type = u32, name = "line-height-numeric", get)]
        #[property(type = u32, name = "line-height-selected-index", get = Self::get_line_height_selected_index, set = Self::set_line_height_selected_index)]
        pub(super) line_height: Cell<TextMetricSize>,
    }

    impl Default for ManuscriptProjectSettingsEditor {
        fn default() -> Self {
            Self {
                project_title_entry: TemplateChild::default(),
                author_entry: TemplateChild::default(),
                paragraph_spacing_entry: TemplateChild::default(),
                line_height_entry: TemplateChild::default(),
                sender: RefCell::default(),
                heading: RefCell::new(i18n("Project Settings")),
                title: RefCell::default(),
                author: RefCell::default(),
                paragraph_spacing: Cell::default(),
                line_height: Cell::default(),
            }
        }
    }

    impl ManuscriptProjectSettingsEditor {
        pub fn get_paragraph_spacing(&self) -> TextMetricSize {
            self.paragraph_spacing.get()
        }

        pub fn set_paragraph_spacing(&self, value: TextMetricSize) {
            self.paragraph_spacing.set(value);
            self.obj().notify_paragraph_spacing_selected_index();
            let func: Box<dyn FnOnce(&mut DocumentManifest) + 'static> =
                Box::new(move |manifest| {
                    manifest.settings_mut().set_paragraph_spacing(value);
                });
            self.obj()
                .send_action(DocumentAction::UpdateManifestWith(func));
        }

        pub fn get_paragraph_spacing_selected_index(&self) -> u32 {
            match self.get_paragraph_spacing() {
                TextMetricSize::Narrow => 0,
                TextMetricSize::Medium => 1,
                TextMetricSize::Wide => 2,
            }
        }

        pub fn set_paragraph_spacing_selected_index(&self, value: u32) {
            self.set_paragraph_spacing(match value {
                0 => TextMetricSize::Narrow,
                1 => TextMetricSize::Medium,
                2 => TextMetricSize::Wide,
                _ => unreachable!("Not a valid option"),
            });
        }

        pub fn get_line_height(&self) -> TextMetricSize {
            self.line_height.get()
        }

        pub fn set_line_height(&self, value: TextMetricSize) {
            self.line_height.set(value);
            self.obj().notify_line_height_selected_index();
            let func: Box<dyn FnOnce(&mut DocumentManifest) + 'static> =
                Box::new(move |manifest| {
                    manifest.settings_mut().set_line_height(value);
                });

            self.obj()
                .send_action(DocumentAction::UpdateManifestWith(func));
        }

        pub fn get_line_height_selected_index(&self) -> u32 {
            match self.get_line_height() {
                TextMetricSize::Narrow => 0,
                TextMetricSize::Medium => 1,
                TextMetricSize::Wide => 2,
            }
        }

        pub fn set_line_height_selected_index(&self, value: u32) {
            self.set_line_height(match value {
                0 => TextMetricSize::Narrow,
                1 => TextMetricSize::Medium,
                2 => TextMetricSize::Wide,
                _ => unreachable!("Not a valid option"),
            });
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ManuscriptProjectSettingsEditor {
        const NAME: &'static str = "ManuscriptProjectSettingsEditor";
        type Type = super::ManuscriptProjectSettingsEditor;
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

    impl ObjectImpl for ManuscriptProjectSettingsEditor {
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

    impl WidgetImpl for ManuscriptProjectSettingsEditor {}
}

glib::wrapper! {
    pub struct ManuscriptProjectSettingsEditor(ObjectSubclass<imp::ManuscriptProjectSettingsEditor>)
        @extends gtk::Widget, @implements gio::ActionGroup, gio::ActionMap;
}

impl EditorWidgetProtocol for ManuscriptProjectSettingsEditor {
    fn editor_widget(&self) -> Option<gtk::Widget> {
        Some(self.upcast_ref::<gtk::Widget>().clone())
    }
    fn side_panel_widget(&self) -> Option<gtk::Widget> {
        None
    }
}

impl ManuscriptProjectSettingsEditor {
    pub fn new(manifest: &DocumentManifest, sender: Option<Sender<DocumentAction>>) -> Self {
        glib::g_debug!(G_LOG_DOMAIN, "Initializing on manifest -> {:?}", manifest);

        let obj: Self = glib::Object::builder().build();
        let imp = obj.imp();

        *imp.sender.borrow_mut() = sender;
        obj.set_title(
            manifest
                .manifest_title()
                .cloned()
                .unwrap_or(String::default()),
        );
        obj.set_author(manifest.author());

        // Set directly on impl to avoid triggering document updates, not needed here
        imp.paragraph_spacing
            .set(manifest.settings().paragraph_spacing());
        imp.line_height.set(manifest.settings().line_height());
        obj.notify_paragraph_spacing_selected_index();
        obj.notify_line_height_selected_index();

        obj
    }

    fn send_action(&self, action: DocumentAction) {
        let maybe_channel = self.imp().sender.borrow();
        if let Some(channel) = maybe_channel.as_ref() {
            channel.send(action).expect("Could not send action");
        }
    }
}

#[gtk::template_callbacks]
impl ManuscriptProjectSettingsEditor {
    #[template_callback]
    fn on_project_title_entry_changed(&self, entry: adw::EntryRow) {
        let new_title = entry.text().to_string();
        let func: Box<dyn FnOnce(&mut DocumentManifest) + 'static> = Box::new(move |manifest| {
            manifest.set_manifest_title(Some(new_title));
        });
        self.send_action(DocumentAction::UpdateManifestWith(func));
    }

    #[template_callback]
    fn on_author_entry_changed(&self, entry: adw::EntryRow) {
        let new_author = entry.text().to_string();
        let func: Box<dyn FnOnce(&mut DocumentManifest) + 'static> = Box::new(move |manifest| {
            manifest.set_author(new_author);
        });
        self.send_action(DocumentAction::UpdateManifestWith(func));
    }
}
