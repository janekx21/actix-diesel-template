// @generated automatically by Diesel CLI.

diesel::table! {
    plant_images (id) {
        id -> Text,
        url -> Text,
        plant_id -> Text,
    }
}

diesel::table! {
    plants (id) {
        id -> Text,
        name -> Text,
        humidity -> Float,
        care -> Nullable<Text>,
        target_humidity -> Float,
    }
}

diesel::joinable!(plant_images -> plants (plant_id));

diesel::allow_tables_to_appear_in_same_query!(
    plant_images,
    plants,
);
