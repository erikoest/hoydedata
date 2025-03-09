use std::error;
use crate::coord::Coord;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

#[derive(Clone, Debug, Error, PartialEq)]
pub enum Error {
    #[error("Lookup '{}' on map '{}' failed", .0, .1)]
    LookupError(Coord, String),
    #[error("No map for coordinate '{}'", .0)]
    MapNotFound(Coord),
    #[error("Map not loaded '{}'", .0)]
    MapNotLoaded(String),
    #[error("Error: '{}'", .0)]
    Generic(String),
}
