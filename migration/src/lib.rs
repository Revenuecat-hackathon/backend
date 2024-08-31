pub use sea_orm_migration::prelude::*;

mod m20240819_093833_create_user_table;
mod m20240821_140334_create_post_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240819_093833_create_user_table::Migration),
            Box::new(m20240821_140334_create_post_table::Migration),
        ]
    }
}
