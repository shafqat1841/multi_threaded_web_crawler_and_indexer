use crate::entities_system::Entities;

mod constants;
mod entities_system;

pub fn run() {
    let entities_res = Entities::new();

    match entities_res {
        Err(err) => {
            println!("{}", err)
        }
        Ok(entities) => match entities.run() {
            Err(err) => {
                println!("{}",err)
            }
            _ => {}
        },
    };
}
