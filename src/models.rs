use serde::{Deserialize, Serialize};

use crate::schema::plants;
use crate::schema::plant_images;
use diesel::prelude::*;

/// plant details.
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable)]
#[diesel(table_name = plants)]
pub struct Plant {
    pub id: String,
    pub name: String,
    pub humidity: f32,
    pub care: Option<String>,
    pub target_humidity: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable)]
#[diesel(belongs_to(Plant))]
#[diesel(table_name = plant_images)]
pub struct PlantImage {
    pub id: String,
    pub url: String,
    pub plant_id: String,
}

/// New plant details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewPlant {
    pub name: String,
}

impl NewPlant {
    /// Constructs new plant details from name.
    #[cfg(test)] // only needed in tests
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Humidity {
    pub humidity: f32
}
