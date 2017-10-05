// This UnitLibrary contains all active, loaded modules, as well as the
// "descriptions" that can be used to [re]load modules.

use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;

use config::Config;
use unit::{UnitKind, UnitName};
use unitbroadcaster::{UnitBroadcaster, UnitCategoryEvent, UnitEvent, UnitStatus, UnitStatusEvent};
use units::interface::{Interface, InterfaceDescription};
use units::jig::{Jig, JigDescription};
use units::scenario::{Scenario, ScenarioDescription};
use units::test::{Test, TestDescription};

pub struct UnitLibrary {
    broadcaster: UnitBroadcaster,
    receiver: Receiver<UnitEvent>,
    config: Arc<Mutex<Config>>,

    /// The unit status is used to determine whether to reload units or not.
    unit_status: RefCell<HashMap<UnitName, UnitStatus>>,

    /// Currently available interface descriptions.  The interfaces they describe might not be valid.
    interface_descriptions: RefCell<HashMap<UnitName, InterfaceDescription>>,

    /// Currently available jig descriptions.  The jigs they describe might not be valid.
    jig_descriptions: RefCell<HashMap<UnitName, JigDescription>>,

    /// Currently available scenario descriptions.  The scenarios they describe might not be valid.
    scenario_descriptions: RefCell<HashMap<UnitName, ScenarioDescription>>,

    /// Currently available test descriptions.  The tests they describe might not be valid.
    test_descriptions: RefCell<HashMap<UnitName, TestDescription>>,

    /// A list of unit names that must be checked when a rescan() is performed.
    dirty_interfaces: RefCell<HashMap<UnitName, ()>>,
    dirty_jigs: RefCell<HashMap<UnitName, ()>>,
    dirty_scenarios: RefCell<HashMap<UnitName, ()>>,
    dirty_tests: RefCell<HashMap<UnitName, ()>>,

    /// Loaded Interfaces, available for checkout.
    interfaces: RefCell<HashMap<UnitName, Arc<Mutex<Interface>>>>,

    /// Loaded Jigs, available for checkout.
    jigs: RefCell<HashMap<UnitName, Arc<Mutex<Jig>>>>,

    /// Loaded Scenarios, available for checkout.
    scenarios: RefCell<HashMap<UnitName, Arc<Mutex<Scenario>>>>,

    /// Loaded Tests, available for checkout.
    tests: Rc<RefCell<HashMap<UnitName, Arc<Mutex<Test>>>>>,
}

impl UnitLibrary {
    pub fn new(broadcaster: &UnitBroadcaster, config: &Arc<Mutex<Config>>) -> Self {
        UnitLibrary {
            broadcaster: broadcaster.clone(),
            receiver: broadcaster.subscribe(),
            config: config.clone(),
            unit_status: RefCell::new(HashMap::new()),

            interface_descriptions: RefCell::new(HashMap::new()),
            jig_descriptions: RefCell::new(HashMap::new()),
            scenario_descriptions: RefCell::new(HashMap::new()),
            test_descriptions: RefCell::new(HashMap::new()),

            dirty_interfaces: RefCell::new(HashMap::new()),
            dirty_jigs: RefCell::new(HashMap::new()),
            dirty_scenarios: RefCell::new(HashMap::new()),
            dirty_tests: RefCell::new(HashMap::new()),

            interfaces: RefCell::new(HashMap::new()),
            jigs: RefCell::new(HashMap::new()),
            scenarios: RefCell::new(HashMap::new()),
            tests: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    pub fn update_interface_description(&mut self, description: InterfaceDescription) {
        let id = description.id().clone();

        self.dirty_interfaces.borrow_mut().insert(id.clone(), ());

        match self.interface_descriptions
            .borrow_mut()
            .insert(id.clone(), description) {
            None => {
                self.unit_status
                    .borrow_mut()
                    .insert(id.clone(), UnitStatus::LoadStarted)
            }
            Some(_) => {
                self.unit_status
                    .borrow_mut()
                    .insert(id.clone(), UnitStatus::UpdateStarted)
            }
        };

        self.broadcaster
            .broadcast(&UnitEvent::Category(UnitCategoryEvent::new(UnitKind::Interface,
                                                                   &format!(
                    "Number of interfaces \
                     loaded: {}",
                    self.interface_descriptions.borrow().len()
                ))));
    }

    pub fn update_jig_description(&mut self, description: JigDescription) {
        let id = description.id().clone();

        // Add the jig name to a list of "dirty jigs" that will be checked during "rescan()"
        self.dirty_jigs.borrow_mut().insert(id.clone(), ());

        // Add an entry to the status to determine whether this unit is new or not.
        match self.jig_descriptions
            .borrow_mut()
            .insert(id.clone(), description) {
            None => {
                self.unit_status
                    .borrow_mut()
                    .insert(id.clone(), UnitStatus::LoadStarted)
            }
            Some(_) => {
                self.unit_status
                    .borrow_mut()
                    .insert(id.clone(), UnitStatus::UpdateStarted)
            }
        };

        self.broadcaster
            .broadcast(&UnitEvent::Category(UnitCategoryEvent::new(UnitKind::Jig,
                                                                   &format!(
                    "Number of units \
                     loaded: {}",
                    self.jig_descriptions.borrow().len()
                ))));
    }

    pub fn update_scenario_description(&mut self, description: ScenarioDescription) {
        let id = description.id().clone();

        self.dirty_scenarios.borrow_mut().insert(id.clone(), ());

        match self.scenario_descriptions
            .borrow_mut()
            .insert(id.clone(), description) {
            None => {
                self.unit_status
                    .borrow_mut()
                    .insert(id.clone(), UnitStatus::LoadStarted)
            }
            Some(_) => {
                self.unit_status
                    .borrow_mut()
                    .insert(id.clone(), UnitStatus::UpdateStarted)
            }
        };

        self.broadcaster
            .broadcast(&UnitEvent::Category(UnitCategoryEvent::new(UnitKind::Scenario,
                                                                   &format!(
                    "Number of scenarios \
                     loaded: {}",
                    self.scenario_descriptions.borrow().len()
                ))));
    }

    pub fn update_test_description(&mut self, description: TestDescription) {
        let id = description.id().clone();

        self.dirty_tests.borrow_mut().insert(id.clone(), ());

        match self.test_descriptions
            .borrow_mut()
            .insert(id.clone(), description) {
            None => {
                self.unit_status
                    .borrow_mut()
                    .insert(id.clone(), UnitStatus::LoadStarted)
            }
            Some(_) => {
                self.unit_status
                    .borrow_mut()
                    .insert(id.clone(), UnitStatus::UpdateStarted)
            }
        };

        self.broadcaster
            .broadcast(&UnitEvent::Category(UnitCategoryEvent::new(UnitKind::Test,
                                                                   &format!(
                    "Number of tests \
                     loaded: {}",
                    self.test_descriptions.borrow().len()
                ))));
    }

    pub fn remove_interface(&mut self, id: &UnitName) {
        self.unit_status
            .borrow_mut()
            .insert(id.clone(), UnitStatus::UnloadStarted);
        self.broadcaster
            .broadcast(&UnitEvent::Status(UnitStatusEvent::new_unloading(id)));
        self.interface_descriptions.borrow_mut().remove(id);
    }

    pub fn remove_jig(&mut self, id: &UnitName) {
        self.unit_status
            .borrow_mut()
            .insert(id.clone(), UnitStatus::UnloadStarted);
        self.broadcaster
            .broadcast(&UnitEvent::Status(UnitStatusEvent::new_unloading(id)));
        self.jig_descriptions.borrow_mut().remove(id);
    }

    pub fn remove_scenario(&mut self, id: &UnitName) {
        self.unit_status
            .borrow_mut()
            .insert(id.clone(), UnitStatus::UnloadStarted);
        self.broadcaster
            .broadcast(&UnitEvent::Status(UnitStatusEvent::new_unloading(id)));
        self.scenario_descriptions.borrow_mut().remove(id);
    }

    pub fn remove_test(&mut self, id: &UnitName) {
        self.unit_status
            .borrow_mut()
            .insert(id.clone(), UnitStatus::UnloadStarted);
        self.broadcaster
            .broadcast(&UnitEvent::Status(UnitStatusEvent::new_unloading(id)));
        self.test_descriptions.borrow_mut().remove(id);
    }

    /// Examine all of the loaded units and ensure they can be loaded.
    ///
    /// Each unit type must be handled differently.
    ///
    /// 1. Mark every Interface, Scenario or Test that depends on a dirty jig as dirty.
    ///    That way, they will be rescanned.
    /// 2. Mark every Scenario that uses a dirty Test as dirty.
    ///    That way, scenario dependency graphs will be re-evaluated.
    /// 3. Delete any "dirty" objects that were Deleted.
    /// 4. Load all Jigs that are valid.
    /// 5. Load all Interfaces that are valid.
    /// 6. Load all Tests that are compatible with this Jig.
    /// 7. Load all Scenarios.
    pub fn rescan(&mut self) {
        self.broadcaster.broadcast(&UnitEvent::RescanStart);
        let mut statuses = self.unit_status.borrow_mut();

        // 1. Go through jigs and mark dependent scenarios and tests as dirty.
        for (jig_name, _) in self.dirty_jigs.borrow().iter() {
            for (test_name, test_description) in self.test_descriptions.borrow().iter() {
                if test_description.supports_jig(jig_name) {
                    self.dirty_tests.borrow_mut().insert(test_name.clone(), ());
                }
            }

            for (scenario_name, scenario_description) in self.scenario_descriptions
                .borrow()
                .iter() {
                if scenario_description.supports_jig(jig_name) {
                    self.dirty_scenarios
                        .borrow_mut()
                        .insert(scenario_name.clone(), ());
                }
            }

            for (interface_name, interface_description) in self.interface_descriptions
                .borrow()
                .iter() {
                if interface_description.supports_jig(jig_name) {
                    self.dirty_interfaces.borrow_mut().insert(interface_name.clone(), ());
                }
            }
        }

        // 2. Go through tests and mark scenarios as dirty.
        for (test_name, _) in self.dirty_tests.borrow().iter() {
            for (scenario_name, scenario) in self.scenarios.borrow().iter() {
                if scenario.lock().unwrap().uses_test(test_name) {
                    self.dirty_scenarios
                        .borrow_mut()
                        .insert(scenario_name.clone(), ());
                }
            }
        }

        // 3. Delete any "dirty" objects that were Deleted.
        for (id, _) in self.dirty_jigs.borrow().iter() {
            if statuses.get(id).unwrap() == &UnitStatus::UnloadStarted {
                self.jigs.borrow_mut().remove(id);
                statuses.remove(id);
            }
        }
        for (id, _) in self.dirty_tests.borrow().iter() {
            if statuses.get(id).unwrap() == &UnitStatus::UnloadStarted {
                self.tests.borrow_mut().remove(id);
                statuses.remove(id);
            }
        }
        for (id, _) in self.dirty_scenarios.borrow().iter() {
            if statuses.get(id).unwrap() == &UnitStatus::UnloadStarted {
                self.scenarios.borrow_mut().remove(id);
                statuses.remove(id);
            }
        }
        for (id, _) in self.dirty_interfaces.borrow().iter() {
            if statuses.get(id).unwrap() == &UnitStatus::UnloadStarted {
                self.interfaces.borrow_mut().remove(id);
                statuses.remove(id);
            }
        }

        // 4. Load all Jigs that are valid.
        for (id, _) in self.dirty_jigs.borrow().iter() {
            match statuses.get(id).unwrap() {
                &UnitStatus::LoadStarted => {
                    self.load_jig(self.jig_descriptions.borrow().get(id).unwrap())
                }
                &UnitStatus::UpdateStarted => {
                    self.load_jig(self.jig_descriptions.borrow().get(id).unwrap())
                }
                x => panic!("Unexpected jig unit status: {}", x),
            }
        }
        self.dirty_jigs.borrow_mut().clear();

        // 5. Load all Interfaces that are compatible with this Jig.
        for (id, _) in self.dirty_interfaces.borrow().iter() {
            match statuses.get(id).unwrap() {
                &UnitStatus::LoadStarted => {
                    self.load_interface(self.interface_descriptions.borrow().get(id).unwrap())
                }
                &UnitStatus::UpdateStarted => {
                    self.load_interface(self.interface_descriptions.borrow().get(id).unwrap())
                }
                x => panic!("Unexpected interface unit status: {}", x),
            }
        }
        self.dirty_interfaces.borrow_mut().clear();

        // 6. Load all Tests that are compatible with this Jig.
        for (id, _) in self.dirty_tests.borrow().iter() {
            match statuses.get(id).unwrap() {
                &UnitStatus::LoadStarted => {
                    self.load_test(self.test_descriptions.borrow().get(id).unwrap())
                }
                &UnitStatus::UpdateStarted => {
                    self.load_test(self.test_descriptions.borrow().get(id).unwrap())
                }
                x => panic!("Unexpected test unit status: {}", x),
            }
        }
        self.dirty_tests.borrow_mut().clear();

        // 7. Load all Scenarios that are compatible with this Jig.
        for (id, _) in self.dirty_scenarios.borrow().iter() {
            match statuses.get(id).unwrap() {
                &UnitStatus::LoadStarted => {
                    self.load_scenario(self.scenario_descriptions.borrow().get(id).unwrap())
                }
                &UnitStatus::UpdateStarted => {
                    self.load_scenario(self.scenario_descriptions.borrow().get(id).unwrap())
                }
                x => panic!("Unexpected scenario unit status: {}", x),
            }
        }
        self.dirty_scenarios.borrow_mut().clear();

        self.broadcaster.broadcast(&UnitEvent::RescanFinish);
    }

    pub fn jig_is_loaded(&self, id: &UnitName) -> bool {
        self.jigs.borrow().get(id).is_some()
    }

    pub fn get_test(&self, id: &UnitName) -> Option<Arc<Mutex<Test>>> {
        match self.tests.borrow().get(id) {
            None => None,
            Some(test) => Some(test.clone()),
        }
    }

    pub fn get_tests(&self) -> Rc<RefCell<HashMap<UnitName, Arc<Mutex<Test>>>>> {
        self.tests.clone()
    }

    fn load_jig(&self, description: &JigDescription) {
        self.jigs.borrow_mut().remove(description.id());

        // "Select" the Jig, which means we can activate it later on.
        let new_jig = match description.select(self, &*self.config.lock().unwrap()) {
            Ok(o) => o,
            Err(e) => {
                self.broadcaster.broadcast(
                    &UnitEvent::Status(UnitStatusEvent::new_unit_incompatible(
                        description.id(),
                        format!("{}", e),
                    )),
                );
                return;
            }
        };
        self.jigs
            .borrow_mut()
            .insert(description.id().clone(), Arc::new(Mutex::new(new_jig)));
        self.broadcaster
            .broadcast(&UnitEvent::Status(UnitStatusEvent::new_selected(description.id())));
    }

    fn load_interface(&self, description: &InterfaceDescription) {
        // If the interface exists in the array already, then it is active and will be deactivated first.
        if let Some(old_interface) = self.interfaces.borrow_mut().remove(description.id()) {
            match old_interface.lock().unwrap().deactivate() {
                Ok(_) =>
            self.broadcaster.broadcast(
                    &UnitEvent::Status(UnitStatusEvent::new_deactivate_success(description.id(), "Reloading interface".to_owned()))),
                Err(e) =>
            self.broadcaster.broadcast(
                    &UnitEvent::Status(UnitStatusEvent::new_deactivate_failure(description.id(), format!("Unable to deactivate: {}", e)))),
            }
            self.broadcaster
                .broadcast(&UnitEvent::Status(UnitStatusEvent::new_deselected(description.id())));
        }

        // "Select" the Interface, which means we can activate it later on.
        let new_interface = match description.select(self, &*self.config.lock().unwrap()) {
            Ok(o) => o,
            Err(e) => {
                self.broadcaster.broadcast(
                    &UnitEvent::Status(UnitStatusEvent::new_unit_incompatible(
                        description.id(),
                        format!("{}", e),
                    )),
                );
                return;
            }
        };

        self.broadcaster
            .broadcast(&UnitEvent::Status(UnitStatusEvent::new_selected(description.id())));

        if let Err(e) = new_interface.activate(&*self.config.lock().unwrap()) {
            self.broadcaster
            .broadcast(&UnitEvent::Status(UnitStatusEvent::new_active_failed(description.id(), format!("{}", e))));
            return;
        }

        self.broadcaster
            .broadcast(&UnitEvent::Status(UnitStatusEvent::new_active(description.id())));

        self.interfaces
            .borrow_mut()
            .insert(description.id().clone(),
                    Arc::new(Mutex::new(new_interface)));
    }

    fn load_test(&self, description: &TestDescription) {
        self.tests.borrow_mut().remove(description.id());

        // "Select" the Jig, which means we can activate it later on.
        let new_test = match description.select(self, &*self.config.lock().unwrap()) {
            Ok(o) => o,
            Err(e) => {
                self.broadcaster.broadcast(
                    &UnitEvent::Status(UnitStatusEvent::new_unit_incompatible(
                        description.id(),
                        format!("{}", e),
                    )),
                );
                return;
            }
        };

        self.tests
            .borrow_mut()
            .insert(description.id().clone(), Arc::new(Mutex::new(new_test)));
        self.broadcaster
            .broadcast(&UnitEvent::Status(UnitStatusEvent::new_selected(description.id())));
    }

    fn load_scenario(&self, description: &ScenarioDescription) {
        self.scenarios.borrow_mut().remove(description.id());

        // "Select" the Jig, which means we can activate it later on.
        let new_scenario = match description.select(self, &*self.config.lock().unwrap()) {
            Ok(o) => o,
            Err(e) => {
                self.broadcaster.broadcast(
                    &UnitEvent::Status(UnitStatusEvent::new_unit_incompatible(
                        description.id(),
                        format!("{}", e),
                    )),
                );
                return;
            }
        };

        self.scenarios
            .borrow_mut()
            .insert(description.id().clone(), Arc::new(Mutex::new(new_scenario)));
        self.broadcaster
            .broadcast(&UnitEvent::Status(UnitStatusEvent::new_selected(description.id())));
    }
}
