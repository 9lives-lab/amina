use std::collections::HashMap;
use std::any::{TypeId, Any};
use std::sync::{Arc, RwLock};
use std::ops::Deref;

pub trait ServiceApi: Send + Sync + 'static {
    fn start(&self) { }
    fn stop(&self) { }
}

pub trait ServiceInitializer: ServiceApi {
    fn initialize(context: &Context) -> Arc<Self>;
}

struct ServiceWrapper {
    entry: Arc<dyn Any + Send + Sync>,
}

pub struct Service<S: ServiceApi> {
    entry: Arc<dyn Any + Send + Sync>,
    _ptr: Arc<Option<S>>,
}

impl <S: ServiceApi> AsRef<S> for Service<S> {
    fn as_ref(&self) -> &S {
        self.entry.as_ref().downcast_ref::<S>().unwrap()
    }
}

impl <S: ServiceApi> Deref for Service<S> {
    type Target = S;

    fn deref(&self) -> &Self::Target {
        self.entry.deref().downcast_ref::<S>().unwrap()
    }
}

impl<S: ServiceApi> Clone for Service<S> {
    fn clone(&self) -> Self {
        Service {
            entry: self.entry.clone(),
            _ptr: Arc::new(None),
        }
    }
}

pub struct Context {
    services: RwLock<HashMap<TypeId, ServiceWrapper>>,
    services_order: RwLock<Vec<Arc<dyn ServiceApi>>>,
}

impl Context {

    pub fn new() -> Self {
        Context {
            services: RwLock::new(HashMap::new()),
            services_order: RwLock::new(Vec::new()),
        }
    }

    pub fn init_service<S>(&self) where S: ServiceInitializer {
        let name = std::any::type_name::<S>();
        log::debug!("Initializing service: {}", name);
        let service = S::initialize(self);
        self.add_service_internal::<S>(service);
    }

    pub fn add_service<S>(&self, service: S) where S: ServiceApi {
        let name = std::any::type_name::<S>();
        log::debug!("Adding service: {}", name);
        self.add_service_internal::<S>(Arc::new(service));
    }

    pub fn get_service<S>(&self) -> Service<S> where S: ServiceApi  {
        let services = self.services.read().unwrap();
        let wrapper = services.get(&TypeId::of::<S>()).unwrap();
        let service_any = wrapper.entry.clone();
        Service {
            entry: service_any,
            _ptr: Arc::new(None),
        }
    }

    pub fn start(&self) {
        for service in self.services_order.read().unwrap().iter() {
            service.start();
        }
    }

    pub fn stop(&self) {
        for service in self.services_order.read().unwrap().iter().rev() {
            service.stop();
        }
    }

    fn add_service_internal<S>(&self, service_arc: Arc<S>) where S: ServiceApi {
        let type_id = TypeId::of::<S>();
        let wrapper = ServiceWrapper {
            entry: service_arc.clone(),
        };
        let mut services = self.services.write().unwrap();
        services.insert(type_id, wrapper);
        self.services_order.write().unwrap().push(service_arc);
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use crate::service::{ServiceApi, Context, Service, ServiceInitializer};

    struct ServiceOne {}

    impl ServiceApi for ServiceOne {
        fn start(&self) {
            println!("ServiceOne started");
        }

        fn stop(&self) {
            println!("ServiceOne stopped");
        }
    }

    impl ServiceInitializer for ServiceOne {
        fn initialize(_: &Context) -> Arc<Self> {
            println!("ServiceOne initialized");
            Arc::new(Self {})
        }
    }

    impl ServiceOne {
        pub fn say_hello(&self) {
            println!("ServiceOne says Hello");
        }
    }

    struct ServiceTwo {
        service_one: Service<ServiceOne>,
    }

    impl ServiceApi for ServiceTwo {
        fn start(&self) {
            println!("ServiceTwo started");
            self.service_one.say_hello();
        }

        fn stop(&self) {
            println!("ServiceTwo stopped");
        }
    }

    impl ServiceInitializer for ServiceTwo {
        fn initialize(context: &Context) -> Arc<Self> {
            let service_one = context.get_service::<ServiceOne>();
            println!("ServiceTwo initialized");
            Arc::new(Self {
                service_one
            })
        }
    }

    #[test]
    fn test_basic() {
        let context = Context::new();
        context.init_service::<ServiceOne>();
        context.init_service::<ServiceTwo>();
        context.start();
        context.stop();
    }
}
