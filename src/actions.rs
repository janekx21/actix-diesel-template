use diesel::prelude::*;
use uuid::Uuid;

use crate::models;
use crate::schema::plant_images::plant_id;

type DbError = Box<dyn std::error::Error + Send + Sync>;

/// Run query using Diesel to find plant by uid and return it.
pub fn find_plant_by_uid(
    conn: &mut SqliteConnection,
    uid: Uuid,
) -> Result<Option<models::Plant>, DbError> {
    use crate::schema::plants::dsl::*;

    let plant = plants
        .filter(id.eq(uid.to_string()))
        .first::<models::Plant>(conn)
        .optional()?;

    Ok(plant)
}

/// Run query using Diesel to insert a new database row and return the result.
pub fn insert_new_plant(
    conn: &mut SqliteConnection,
    nm: &str, // prevent collision with `name` column imported inside the function
) -> Result<models::Plant, DbError> {
    // It is common when using Diesel with Actix Web to import schema-related
    // modules inside a function's scope (rather than the normal module's scope)
    // to prevent import collisions and namespace pollution.
    use crate::schema::plants::dsl::*;

    let new_plant = models::Plant {
        id: Uuid::new_v4().to_string(),
        name: nm.to_owned(),
        care: None,
        humidity: 0.5,
        target_humidity: 0.5,
    };

    diesel::insert_into(plants).values(&new_plant).execute(conn)?;

    Ok(new_plant)
}

pub fn insert_new_plant_image(
    conn: &mut SqliteConnection,
    url_str: &str,
    plant_uid: &str,
)-> Result<models::PlantImage, DbError> {
    use crate::schema::plant_images::dsl::*;

    let new_plant_image = models::PlantImage {
        id: Uuid::new_v4().to_string(),
        url: url_str.to_owned(),
        plant_id: plant_uid.to_owned(),
    };

    diesel::insert_into(plant_images).values(&new_plant_image).execute(conn)?;

    Ok(new_plant_image)
}

pub fn update_plant_humidity(
    conn: &mut SqliteConnection,
    uid: Uuid,
    data: f32
) -> Result<(), DbError> {
    use crate::schema::plants::dsl::*;

    diesel::update(plants)
        .filter(id.eq(uid.to_string()))
        .set(humidity.eq(data))
        .execute(conn)?;

    Ok(())
}
