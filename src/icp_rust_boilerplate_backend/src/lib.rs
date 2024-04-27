#[macro_use]
extern crate serde;
use candid::{Decode, Encode};
use ic_cdk::api::time;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::{borrow::Cow, cell::RefCell};

type Memory = VirtualMemory<DefaultMemoryImpl>;
type IdCell = Cell<u64, Memory>;

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Console {
    id: u64,
    name: String,
    occupied: bool,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Rental {
    id: u64,
    console_id: u64,
    player: String,
    start_time: u64,
    duration: u64,
}

impl Storable for Console {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for Console {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

impl Storable for Rental {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for Rental {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    static CONSOLE_ID_COUNTER: RefCell<IdCell> = RefCell::new(
        IdCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))), 0)
            .expect("Cannot create a counter")
    );

    static RENTAL_ID_COUNTER: RefCell<IdCell> = RefCell::new(
        IdCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1))), 0)
            .expect("Cannot create a counter")
    );

    static CONSOLES: RefCell<StableBTreeMap<u64, Console, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2)))
    ));

    static RENTALS: RefCell<StableBTreeMap<u64, Rental, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(3)))
    ));
}

#[ic_cdk::query]
fn get_console(id: u64) -> Result<Console, Error> {
    match _get_console(&id) {
        Some(console) => Ok(console),
        None => Err(Error::NotFound {
            msg: format!("a console with id={} not found", id),
        }),
    }
}

fn _get_console(id: &u64) -> Option<Console> {
    CONSOLES.with(|c| c.borrow().get(id))
}

#[ic_cdk::query]
fn get_rental(id: u64) -> Result<Rental, Error> {
    match _get_rental(&id) {
        Some(rental) => Ok(rental),
        None => Err(Error::NotFound {
            msg: format!("a rental with id={} not found", id),
        }),
    }
}

fn _get_rental(id: &u64) -> Option<Rental> {
    RENTALS.with(|r| r.borrow().get(id))
}

#[ic_cdk::update]
fn add_console(name: String) -> Option<Console> {
    let id = CONSOLE_ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("cannot increment id counter");
    let console = Console {
        id,
        name,
        occupied: true,
    };
    do_insert_console(&console);
    Some(console)
}

fn do_insert_console(console: &Console) {
    CONSOLES.with(|service| service.borrow_mut().insert(console.id, console.clone()));
}

#[ic_cdk::update]
fn add_rental(console_id: u64, player: String, start_time: u64, duration: u64) -> Option<Rental> {
    let id = RENTAL_ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("cannot increment id counter");
    let rental = Rental {
        id,
        console_id,
        player,
        start_time,
        duration,
    };
    do_insert_rental(&rental);
    Some(rental)
}

fn do_insert_rental(rental: &Rental) {
    RENTALS.with(|service| service.borrow_mut().insert(rental.id, rental.clone()));
}

#[ic_cdk::update]
fn update_console(id: u64, name: String, occupied: bool) -> Result<Console, Error> {
    match CONSOLES.with(|service| service.borrow().get(&id)) {
        Some(mut console) => {
            console.name = name;
            console.occupied = occupied;
            do_insert_console(&console);
            Ok(console)
        }
        None => Err(Error::NotFound {
            msg: format!("a console with id={} not found", id),
        }),
    }
}

#[ic_cdk::update]
fn update_rental(id: u64, console_id: u64, player: String, start_time: u64, duration: u64) -> Result<Rental, Error> {
    match RENTALS.with(|service| service.borrow().get(&id)) {
        Some(mut rental) => {
            rental.console_id = console_id;
            rental.player = player;
            rental.start_time = start_time;
            rental.duration = duration;
            do_insert_rental(&rental);
            Ok(rental)
        }
        None => Err(Error::NotFound {
            msg: format!("a rental with id={} not found", id),
        }),
    }
}

#[ic_cdk::update]
fn delete_console(id: u64) -> Result<(), Error> {
    match CONSOLES.with(|service| service.borrow_mut().remove(&id)) {
        Some(_) => Ok(()),
        None => Err(Error::NotFound {
            msg: format!("a console with id={} not found", id),
        }),
    }
}

#[derive(candid::CandidType, Serialize, Deserialize)]
enum Error {
    NotFound { msg: String },
}

// need this to generate candid
ic_cdk::export_candid!();

