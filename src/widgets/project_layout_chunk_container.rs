use crate::{models::*, widgets::ManuscriptChunkRow};
use adw::subclass::prelude::*;
use glib_macros::Properties;
use gtk::{gio, glib, prelude::*};
use std::{cell::RefCell, collections::HashMap};

#[allow(unused)]
const G_LOG_DOMAIN: &str = "ManuscriptProjectLayoutChunkContainer";

mod imp {
    use super::*;
    use glib::{ParamSpec, ParamSpecBoolean};
    use once_cell::sync::Lazy;

    #[derive(Default, Properties, gtk::CompositeTemplate)]
    #[properties(wrapper_type = super::ManuscriptProjectLayoutChunkContainer)]
    #[template(resource = "/io/sixpounder/Manuscript/project_layout_chunk_container.ui")]
    pub struct ManuscriptProjectLayoutChunkContainer {
        #[template_child]
        pub(super) listbox: TemplateChild<gtk::ListBox>,

        #[property(get, set)]
        pub(super) category_name: RefCell<String>,

        #[property(get, set)]
        pub(super) title: RefCell<String>,

        pub(super) children_map: RefCell<HashMap<String, ManuscriptChunkRow>>,
    }

    impl ManuscriptProjectLayoutChunkContainer {}

    #[glib::object_subclass]
    impl ObjectSubclass for ManuscriptProjectLayoutChunkContainer {
        const NAME: &'static str = "ManuscriptProjectLayoutChunkContainer";
        type Type = super::ManuscriptProjectLayoutChunkContainer;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.set_layout_manager_type::<gtk::BinLayout>();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ManuscriptProjectLayoutChunkContainer {
        fn constructed(&self) {
            self.parent_constructed();
            self.listbox.set_sort_func(|row1, row2| {
                let chunk_row1 = row1.downcast_ref::<ManuscriptChunkRow>().expect("How?");
                let chunk_row2 = row2.downcast_ref::<ManuscriptChunkRow>().expect("How?");

                chunk_row1.priority().cmp(&chunk_row2.priority()).into()
            })
        }

        fn properties() -> &'static [gtk::glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                let derived: &'static [gtk::glib::ParamSpec] =
                    ManuscriptProjectLayoutChunkContainer::derived_properties();
                let mut props: Vec<ParamSpec> = vec![ParamSpecBoolean::builder("has-items")
                    .read_only()
                    .default_value(false)
                    .build()];

                props.append(&mut derived.to_vec());
                props
            });

            PROPERTIES.as_ref()
        }

        fn property(&self, id: usize, pspec: &ParamSpec) -> glib::Value {
            match pspec.name() {
                "has-items" => (self.children_map.borrow().len() > 0).to_value(),
                _ => self.derived_property(id, pspec),
            }
        }

        fn set_property(&self, id: usize, value: &glib::Value, pspec: &ParamSpec) {
            self.derived_set_property(id, value, pspec)
        }
    }

    impl WidgetImpl for ManuscriptProjectLayoutChunkContainer {}
}

glib::wrapper! {
    pub struct ManuscriptProjectLayoutChunkContainer(ObjectSubclass<imp::ManuscriptProjectLayoutChunkContainer>)
        @extends gtk::Widget, @implements gio::ActionGroup, gio::ActionMap;
}

impl Default for ManuscriptProjectLayoutChunkContainer {
    fn default() -> Self {
        Self::new()
    }
}

impl ManuscriptProjectLayoutChunkContainer {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }

    pub fn show_all(&self) {}

    pub fn set_selection_mode(&self, mode: gtk::SelectionMode) {
        let listbox = self.imp().listbox.get();
        listbox.set_selection_mode(mode);

        let map = self.imp().children_map.borrow();
        map.iter().for_each(|(_chunk_id, widget)| {
            if mode == gtk::SelectionMode::Multiple {
                widget.set_select_mode(true);
            } else {
                widget.set_select_mode(false);
            }
        });
    }

    pub fn clear_selection(&self) {}

    pub fn select_all_rows(&self) {
        let map = self.imp().children_map.borrow();
        map.iter()
            .for_each(|(_key, widget)| widget.set_property("selected", true));
    }

    pub fn add(&self, chunk: &dyn DocumentChunk) -> ManuscriptChunkRow {
        let row = ManuscriptChunkRow::new(Some(chunk), self);
        let row_map = row.clone();
        {
            let mut map = self.imp().children_map.borrow_mut();
            map.insert(chunk.id().to_string(), row_map);

            self.imp().listbox.append(&row);
        }

        self.notify("has-items");
        row
    }

    pub fn remove(&self, chunk: &dyn DocumentChunk) {
        self.remove_by_id(chunk.id().to_string())
    }

    pub fn remove_by_id(&self, chunk_id: String) {
        let mut changed: bool = false;
        {
            let mut map = self.imp().children_map.borrow_mut();
            if let Some(removed) = map.remove(&chunk_id) {
                self.imp().listbox.remove(&removed);
                changed = true;
            }
        }

        if changed {
            self.notify("has-items");
        }
    }

    pub fn remove_all(&self) {
        {
            let listbox = self.imp().listbox.get();
            let mut map = self.imp().children_map.borrow_mut();
            for widget in map.values() {
                listbox.remove(widget);
            }

            *map = HashMap::new();
        }
        self.notify("has-items");
    }

    pub fn chunk_row(&self, chunk: &dyn DocumentChunk) -> Option<ManuscriptChunkRow> {
        let widget_ref = self.imp().children_map.borrow();
        widget_ref.get(&chunk.id().to_string()).cloned()
    }
}
