use std::iter::Map;
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use warp::{Filter, path};
use warp::http::StatusCode;
use warp::reply::Json;

#[derive(Debug, Deserialize, Serialize)]
struct Humidity {
    humidity: f32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Plant {
    name: String,
    humidity: f32,
}

impl Room {
    fn new(nm: &str) -> Self {
        Room { name: nm.into(), plants: Vec::default() }
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct Room {
    name: String,
    plants: Vec<Plant>,
}

#[derive(Default, Debug, Deserialize, Serialize)]
struct State {
    rooms: Vec<Room>,
    /*rooms_by_id: Map<uuid, Room>,
    plants_by_id: Map<uuid, Plant>,*/
}

#[tokio::main]
async fn main() {
    let state = Arc::new(Mutex::new(State::default()));

    let get_rooms = warp::get()
        .and(warp::path::end())
        .map({
            let state = state.clone();
            move || {
                let state = state.lock().unwrap();
                warp::reply::json(&*state)
            }
        });

    let create_room = warp::post()
        .and(warp::body::json::<Room>())
        .and(warp::path::end())
        .map(
        {
            let state = state.clone();
            move |room| {
                println!("{:?}", room);
                let mut state = state.lock().unwrap();
                state.rooms.push(room);
                warp::reply::json(&*state)
            }
        }
    );

    let log_humidity = warp::post()
        .and(warp::path::param())
        .and(warp::body::json::<Humidity>())
        .map(
        {
            let state = state.clone();
            move |name: String, humidity: Humidity| {
                let mut state = state.lock().unwrap();
                let mut plant = state.rooms.iter_mut().flat_map(|room| &mut room.plants).find(|plant| plant.name == name);
                match plant {
                    None => {}
                    Some(mut plant) => {
                        plant.humidity = humidity.humidity;
                    }
                };
                warp::reply::json(&*state)
            }
        }
    );

    let rooms_api = warp::path("rooms").and(get_rooms.or(create_room));
    let plants_api = warp::path("plants").and(log_humidity);

    let api = rooms_api.or(plants_api);

    warp::serve(api)
        .run(([127, 0, 0, 1], 3030))
        .await;
}