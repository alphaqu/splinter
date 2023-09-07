use std::any::{Any, TypeId};
use std::collections::vec_deque::Iter;
use std::collections::{HashMap, VecDeque};
use std::fmt::Debug;
use std::iter::Rev;

use tracing::{trace, warn};

pub struct EventSystem {
    storages: HashMap<TypeId, Box<dyn EventStorageDyn>>,
    states: HashMap<TypeId, Box<dyn Any>>,
    id: u64,
}

impl EventSystem {
    pub fn new() -> EventSystem {
        EventSystem {
            storages: Default::default(),
            states: Default::default(),
            id: 0,
        }
    }

    pub fn get<V: 'static>(&self) -> &V {
        self.states.get(&TypeId::of::<V>()).unwrap().downcast_ref().unwrap()
    }

    pub fn get_mut<V: 'static>(&mut self) -> &mut V {
        self.states.get_mut(&TypeId::of::<V>()).unwrap().downcast_mut().unwrap()
    }
    
    pub fn set<V: 'static>(&mut self, value: V) {
        self.states.insert(TypeId::of::<V>(), Box::new(value));
    }

    pub fn run(&mut self, tracker: &mut EventTracker) -> EventCommander<'_> {
        tracker.tick(self)
    }

    fn storage<T: Event>(&self) -> Option<&EventStorage<T>> {
        self.storages
            .get(&TypeId::of::<T>())?
            .as_any()
            .downcast_ref::<EventStorage<T>>()
    }

    fn storage_mut<T: Event>(&mut self) -> &mut EventStorage<T> {
        let storage = self
            .storages
            .entry(TypeId::of::<T>())
            .or_insert_with(|| Box::new(EventStorage::<T>::new()));

        storage
            .as_any_mut()
            .downcast_mut::<EventStorage<T>>()
            .unwrap()
    }
}

pub struct EventStorage<D: Event> {
    events: VecDeque<EventData<D>>,
}

impl<D: Event> EventStorage<D> {
    pub fn new() -> EventStorage<D> {
        EventStorage {
            events: Default::default(),
        }
    }
}

impl<D: Event> EventStorageDyn for EventStorage<D> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn clear_ids(&mut self, id: u64) {
        // Go through all the event in the back and clear them until we hit events above our id.
        while let Some(event) = self.events.back() {
            if event.id <= id {
                if event.id < id {
                    warn!("Id {id} was never cleared by its own system");
                }

                self.events.remove(self.events.len() - 1);
            } else {
                return;
            }
        }
    }
}

pub trait EventStorageDyn {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn clear_ids(&mut self, id: u64);
}

// Holds information about your events so they can get cleared once this runs again.
pub struct EventTracker {
    last_id: Option<u64>,
}

impl EventTracker {
    pub fn new() -> EventTracker {
        EventTracker { last_id: None }
    }

    pub fn tick<'a>(&mut self, system: &'a mut EventSystem) -> EventCommander<'a> {
        if let Some(id) = self.last_id {
            for storage in system.storages.values_mut() {
                storage.clear_ids(id);
            }
        }
        let id = system.id;
        self.last_id = Some(id);
        system.id += 1;
        EventCommander { id, system }
    }
}

pub struct EventCommander<'a> {
    id: u64,
    system: &'a mut EventSystem,
}

impl<'a> EventCommander<'a> {
    pub fn dispatch<D: Event>(&mut self, event: D) {
        trace!(target: "event", "Dispatched event {event:?}");
        self.system.storage_mut::<D>().events.push_front(EventData {
            data: event,
            id: self.id,
        });
    }

    pub fn consume<D: Event>(&self) -> EventIterator<'_, D> {
        trace!(target: "event", "Consuming {} events", std::any::type_name::<D>());

        match self.system.storage::<D>() {
            Some(iter) => EventIterator::Storage(iter.events.iter().rev()),
            None => EventIterator::Empty,
        }
    }
}

pub enum EventIterator<'a, T: Event> {
    Storage(Rev<Iter<'a, EventData<T>>>),
    Empty,
}

impl<'a, T: Event> Iterator for EventIterator<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            EventIterator::Storage(iter) => match iter.next() {
                Some(value) => Some(&value.data),
                None => None,
            },
            EventIterator::Empty => None,
        }
    }
}

pub struct EventData<D: Event> {
    data: D,
    id: u64,
}

pub trait Event: Debug + 'static {

}

impl<A: Debug + 'static> Event for A {

}

#[cfg(test)]
mod tests {
    use crate::{EventSystem, EventTracker};

    #[test]
    fn basic() {
        let mut system = EventSystem::new();
        let mut tracker = EventTracker::new();
        let mut tracker2 = EventTracker::new();

        let mut commander = tracker.tick(&mut system);
        commander.dispatch(0u32);
        commander.dispatch(1u32);
        commander.dispatch(2u32);

        let mut commander = tracker2.tick(&mut system);
        commander.dispatch(5u32);
        commander.dispatch(6u32);

        assert_eq!(commander.consume::<u32>().copied().collect::<Vec<u32>>(), vec![0u32, 1u32, 2u32, 5u32, 6u32]);


        let mut commander = tracker.tick(&mut system);
        commander.dispatch(69u32);
        commander.dispatch(420u32);
        commander.dispatch(10u32);

        assert_eq!(commander.consume::<u32>().copied().collect::<Vec<u32>>(), vec![5u32, 6u32, 69u32, 420u32, 10u32]);

        let commander = tracker2.tick(&mut system);
        assert_eq!(commander.consume::<u32>().copied().collect::<Vec<u32>>(), vec![69u32, 420u32, 10u32]);
        let commander = tracker.tick(&mut system);

        assert_eq!(commander.consume::<u32>().copied().collect::<Vec<u32>>(), vec![]);
    }
}
