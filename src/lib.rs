pub mod app;
pub mod controllers;
pub mod data;
pub mod dtos;
pub mod env_loader;
pub mod initializers;
pub mod mailers;
pub mod models;
pub mod tasks;
pub mod views;
pub mod workers;

#[cfg(test)]
#[ctor::ctor(unsafe)]
fn init_unit_test_env() {
    env_loader::load_test_env();
}
