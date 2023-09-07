-- Your SQL goes here
create table plant_images
(
    id       VARCHAR NOT NULL PRIMARY KEY,
    url      VARCHAR NOT NULL,
    plant_id VARCHAR not null,
    foreign key (plant_id) references plants (id)
);

alter table plants
add humidity float not null default 0.0;

alter table plants
add care VARCHAR;

alter table plants
add target_humidity float not null default 0.0;