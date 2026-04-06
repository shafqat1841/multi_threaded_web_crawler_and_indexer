use crate::entities_system::Entities;

mod constants;
mod entities_system;

pub fn run() {
    let mut entities = Entities::new();

    entities.run();
}
