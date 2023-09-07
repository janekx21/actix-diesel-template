-- This file should undo anything in `up.sql`
drop table plant_images;
alter table plants drop column humidity;
alter table plants drop column target_humidity;
alter table plants drop column care;