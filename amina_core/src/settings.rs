use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::ops::{DerefMut, Deref};
use std::sync::{Mutex, RwLock, Arc};
use std::sync::atomic::{AtomicBool, Ordering};
use std::fmt::Debug;

use serde::Serialize;
use yaml_rust::{YamlLoader, Yaml, YamlEmitter};
use yaml_rust::yaml::Hash;

use crate::register_rpc_handler;
use crate::rpc::Rpc;
use crate::service::{Context, ServiceApi, ServiceInitializer};

#[derive(Clone, Debug)]
pub struct Property<T: Clone + Debug> {
    value: Arc<RwLock<T>>,
    change_listener: Arc<AtomicBool>,
}

impl <T: Clone + Debug> Property<T> {

    pub fn new(value: T, change_listener: Arc<AtomicBool>) -> Self {
        Self {
            value: Arc::new(RwLock::new(value)),
            change_listener,
        }
    }

    pub fn set(&mut self, value: T) {
        let mut guard = self.value.write().unwrap();
        let value_ref = guard.deref_mut();
        *value_ref = value;
        // Set flag that one of properties was changed
        self.change_listener.store(true, Ordering::Relaxed);
    }

    pub fn get(&self) -> T {
        self.value.read().unwrap().clone()
    }

}

#[derive(Debug)]
enum PropertyWrapper {
    String(Property<String>),
    _Int(Property<i32>),
    _Bool(Property<bool>),
}

struct SettingsServiceEntry {
    properties: Mutex<HashMap<String, PropertyWrapper>>,
    change_listener: Arc<AtomicBool>,
    path: PathBuf,
}

#[derive(Clone)]
pub struct Settings {
    entry: Arc<SettingsServiceEntry>,
}

impl Settings {

    fn create(properties: HashMap<String, PropertyWrapper>, path: &Path, change_listener: Arc<AtomicBool>) -> Self {
        Self {
            entry: Arc::new(SettingsServiceEntry {
                properties: Mutex::new(properties),
                change_listener,
                path: path.to_path_buf(),
            })
        }
    }

    pub fn create_empty(path: &Path) -> Self {
        Self::create(HashMap::new(), path, Arc::new(AtomicBool::new(false)))
    }

    pub fn init_from_string(text: &str, path: &Path) -> Self {
        let docs = YamlLoader::load_from_str(&text).unwrap();
        let doc = &docs[0];
        let change_listener = Arc::new(AtomicBool::new(false));
        let mut properties = HashMap::<String, PropertyWrapper>::new();
        match doc {
            Yaml::Hash(hash) => {
                Self::load_recursive(hash, &mut properties, "", change_listener.clone());
            },
            _ => panic!("Root element must be 'Hash'")
        }
        Self::create(properties, path, change_listener)
    }

    fn load_recursive(hash: &Hash, properties: &mut HashMap<String, PropertyWrapper>, key: &str, change_listener: Arc<AtomicBool>) {
        for element in hash {
            let name = element.0.as_str().unwrap();
            let next_key = if key.len() > 0 {
                key.to_string() + "." + name
            } else {
                name.to_string()
            };
            match element.1 {
                Yaml::Hash(next_hash) => {
                    Self::load_recursive(next_hash, properties, &next_key, change_listener.clone());
                },
                Yaml::String(string_value) => {
                    properties.insert(next_key, PropertyWrapper::String(
                        Property::new(string_value.clone(), change_listener.clone())
                    ));
                },
                _ => {

                }
            }
        }
    }

    pub fn save_to_file(&self) {
        let data = self.save_to_string();
        std::fs::write(self.entry.path.as_path(), data).expect("Unable to write file");
    }

    fn save_to_string(&self) -> String {
        let mut root = Hash::new();
        for prop in self.entry.properties.lock().unwrap().deref() {
            let mut key: Vec<&str> = prop.0.as_str().split(".").collect();
            Self::dump_recursive(&mut root, &mut key, prop.1);
        }
        let doc = Yaml::Hash(root);
        let mut out_str = String::new();
        YamlEmitter::new(&mut out_str).dump(&doc).unwrap();
        return out_str;
    }

    fn dump_recursive(root: &mut Hash, key: &mut Vec<&str>, prop: &PropertyWrapper) {
        let key_part = key[0];
        let node_key = Yaml::String(key_part.to_string());
        key.remove(0);
        if key.len() > 0 {
            match root.get_mut(&node_key) {
                Some(node) => {
                    match node {
                        Yaml::Hash(hash_node) => {
                            Self::dump_recursive(hash_node, key, prop);
                        },
                        _ => panic!("Root element must be 'Hash'")
                    }
                },
                None => {
                    let mut hash_node = Hash::new();
                    Self::dump_recursive(&mut hash_node, key, prop);
                    root.insert(node_key, Yaml::Hash(hash_node));
                }
            }
        } else {
            match prop {
                PropertyWrapper::String(string_prop) => {
                    root.insert(node_key, Yaml::String(string_prop.get()));
                },
                _ => panic!("Unsupported property type")
            }
        }
    }

    pub fn get_string(&self, key: &str) -> Property<String> {
        let mut properties = self.entry.properties.lock().unwrap();
        match properties.get(key) {
            Some(wrapper) => {
                match wrapper {
                    PropertyWrapper::String(prop) => {
                        return prop.clone();
                    },
                    _ => panic!("Property type mismatch")
                }
            },
            None => {
                let prop = Property::new("".to_string(), self.entry.change_listener.clone());
                properties.insert(key.to_string(), PropertyWrapper::String(prop.clone()));
                return prop;
            }
        }
    }

    pub fn get_properties(&self) -> Vec<String> {
        let mut result = Vec::new();
        let properties = self.entry.properties.lock().unwrap();
        for prop in properties.deref() {
            result.push(prop.0.clone());
        }
        return result;
    }

}

#[derive(Clone, Debug, Serialize)]
pub struct PropertyDescription {
    pub name: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct SectionDescription {
    pub name: String,
    pub properties: Vec<PropertyDescription>,
}

#[derive(Clone, Debug, Serialize)]
pub struct TabDescription {
    pub name: String,
    pub sections: Vec<SectionDescription>,
}

impl TabDescription {

    pub fn get_section(&mut self, section_name: &str) -> Option<&mut SectionDescription> {
          self.sections.iter_mut().find(|section| section.name == section_name)
    }

    fn get_or_add_section(&mut self, section_name: &str) -> &mut SectionDescription {
        return if let Some(index) = self.sections.iter().position(|section| section.name == section_name) {
            self.sections.get_mut(index).unwrap()
        } else {
            let section_description = SectionDescription {
                name: section_name.to_string(),
                properties: Vec::new(),
            };
            self.sections.push(section_description);
            self.sections.last_mut().unwrap()
        }
    }
}

pub struct SettingsDescription {
    tabs: Vec<TabDescription>,
}

impl SettingsDescription {

    fn empty() -> Self {
        Self {
            tabs: Vec::new(),
        }
    }

    fn get_tab(&mut self, tab_name: &str) -> Option<&mut TabDescription> {
        self.tabs.iter_mut().find(|tab| tab.name == tab_name)
    }

    fn get_or_add_tab(&mut self, tab_name: &str) -> &mut TabDescription {
        return if let Some(index) = self.tabs.iter().position(|tab| tab.name == tab_name) {
            self.tabs.get_mut(index).unwrap()
        } else {
            let tab_description = TabDescription {
                name: tab_name.to_string(),
                sections: Vec::new(),
            };
            self.tabs.push(tab_description);
            self.tabs.last_mut().unwrap()
        }
    }

    fn add_property(&mut self, property_path: &str) {
        let mut parts = property_path.splitn(3, ".");
        let tab_name = parts.next().unwrap();
        let section_name = parts.next().unwrap();
        let property_name = parts.next().unwrap();
        let tab_description = self.get_or_add_tab(tab_name);
        let section_description = tab_description.get_or_add_section(section_name);
        if !section_description.properties.iter().any(|prop| prop.name == property_name) {
            section_description.properties.push(PropertyDescription {
                name: property_path.to_string(),
            });
        }
    }

    fn add_properties(&mut self, properties: Vec<String>) {
        for property in properties {
            self.add_property(&property);
        }
    }
}

pub struct SettingsManager {
    settings_list: Mutex<Vec<Arc<Settings>>>,
    settings_description: Mutex<SettingsDescription>,
}

impl SettingsManager {

    pub fn get_tabs(&self) -> Vec<String> {
        let settings_description = self.settings_description.lock().unwrap();
        let mut result = Vec::new();
        for tab in settings_description.tabs.deref() {
            result.push(tab.name.clone());
        }
        return result;
    }

    pub fn get_tab(&self, tab_name: String) -> TabDescription {
        let mut settings_description = self.settings_description.lock().unwrap();
        return settings_description.get_tab(&tab_name).unwrap().clone();
    }

    pub fn register_settings(&self, settings: Arc<Settings>) {
        let mut settings_list = self.settings_list.lock().unwrap();
        settings_list.push(settings);
    }

    pub fn get_string_value(&self, key: String) -> String {
        let settings_list = self.settings_list.lock().unwrap();
        let property = settings_list.first().unwrap().get_string(&key).get();
        return property;
    }

    pub fn set_string_value(&self, key: String, data: String) {
        let settings_list = self.settings_list.lock().unwrap();
        settings_list.first().unwrap().get_string(&key).set(data);
    }

    fn regenerate_settings_description(&self) {
        let mut settings_description = self.settings_description.lock().unwrap();
        settings_description.tabs.clear();
        let settings_list = self.settings_list.lock().unwrap();
        for settings in settings_list.deref() {
            let settings_properties = settings.get_properties();
            settings_description.add_properties(settings_properties);
        }
    }

}

impl ServiceApi for SettingsManager {
    fn start(&self) {
        self.regenerate_settings_description();
    }
}

impl ServiceInitializer for SettingsManager {
    fn initialize(context: &Context) -> Arc<Self> {
        let rpc = context.get_service::<Rpc>();

        let settings_manager = Arc::new(Self {
            settings_list: Mutex::new(Vec::new()),
            settings_description: Mutex::new(SettingsDescription::empty()),
        });

        register_rpc_handler!(rpc, settings_manager, "amina_core.settings_manager.get_tabs", get_tabs());
        register_rpc_handler!(rpc, settings_manager, "amina_core.settings_manager.get_tab", get_tab(tab_name: String));
        register_rpc_handler!(rpc, settings_manager, "amina_core.settings_manager.get_string_value", get_string_value(key: String));
        register_rpc_handler!(rpc, settings_manager, "amina_core.settings_manager.set_string_value", set_string_value(key: String, data: String));

        return settings_manager;
    }
}

#[cfg(test)]
mod tests {
    use crate::settings::Settings;
    use std::path::PathBuf;

    #[test]
    fn test_init() {
        let text =
            "
            main:
                collection_dir: \"some_dir\"
            bar:
                - 1
                - 2.0
            ";
        let service = Settings::init_from_string(&text, PathBuf::new().as_path());

        assert_eq!(service.get_string("main.collection_dir").get(), "some_dir".to_string());
    }

    #[test]
    fn test_save() {
        let service = Settings::create_empty(PathBuf::new().as_path());
        service.get_string("main.collection_dir").set("some_dir".to_string());
        let text = service.save_to_string();

        println!("{}", &text);

        let service = Settings::init_from_string(&text, PathBuf::new().as_path());
        assert_eq!(service.get_string("main.collection_dir").get(), "some_dir".to_string());
    }

}
