use crate::entities_system::Entities;

mod constants;
mod entities_system;

pub fn run() {
    let entities = Entities::new();

    entities.run();
}
