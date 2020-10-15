extern crate csv;
use std::error::Error;
use std::fs::File;


#[derive(Serialize)]
pub struct GamestateRecord<'a> {
    timestep: u64,
    x: f32,
    y: f32,
    id: u64,
    health: i32,
    score: i32,
    entity: &'a str,
}

impl<'a> GamestateRecord<'a> {
    pub fn new(timestep: u64, x: f32, y:f32, id: u64, health: i32, score: i32, entity: &'a str) -> Self {
        GamestateRecord {
            timestep: timestep,
            x: x,
            y: y,
            id: id,
            health: health,
            score: score,
            entity: entity,
        }
    }
}

#[derive(Serialize)]
pub struct EventRecord {
    timestep: u64,
    first_id: i32,
    second_id: i32,
    first: String,
    second: String,
    pos_x: u64,
    pos_y: u64,
    first_score: u64,
    second_score: u64,
}

impl EventRecord{
    pub fn new(timestep: u64, first_id: i32, second_id: i32, first: String, second: String, pos_x: u64, pos_y: u64, first_score: u64, second_score: u64,) -> Self {
        EventRecord {
            timestep: timestep,
            first_id: first_id,
            second_id: second_id,
            first: first,
            second: second,
            pos_x: pos_x,
            pos_y: pos_y,
            first_score: first_score,
            second_score: second_score,
        }
    }
}

#[derive(Serialize)]
pub struct AverageRecord {
    timestep: u64,
    prey_avg_score: f32,
    hunter1_avg_score: f32,
    hunter2_avg_score: f32,
    prey_avg_health: f32,
    hunter1_avg_health: f32,
    hunter2_avg_health: f32,
    top_prey_avg_score: f32,
    top_hunter1_avg_score: f32,
    top_hunter2_avg_score: f32,
    top_prey_avg_health: f32,
    top_hunter1_avg_health: f32,
    top_hunter2_avg_health: f32,
}

impl AverageRecord{
    pub fn new(timestep: u64, prey_avg_score: f32, hunter1_avg_score: f32, hunter2_avg_score: f32, prey_avg_health: f32, hunter1_avg_health: f32, hunter2_avg_health: f32,
        top_prey_avg_score: f32, top_hunter1_avg_score: f32, top_hunter2_avg_score: f32, top_prey_avg_health: f32, top_hunter1_avg_health: f32, top_hunter2_avg_health: f32) -> Self {
        AverageRecord {
            timestep: timestep,
            prey_avg_score: prey_avg_score,
            hunter1_avg_score: hunter1_avg_score,
            hunter2_avg_score: hunter2_avg_score,
            prey_avg_health: prey_avg_health,
            hunter1_avg_health: hunter1_avg_health,
            hunter2_avg_health: hunter2_avg_health,
            top_prey_avg_score: top_prey_avg_score,
            top_hunter1_avg_score: top_hunter1_avg_score,
            top_hunter2_avg_score: top_hunter2_avg_score,
            top_prey_avg_health: top_prey_avg_health,
            top_hunter1_avg_health: top_hunter1_avg_health,
            top_hunter2_avg_health: top_hunter2_avg_health,
        }
    }
}

pub struct Writer {
    writer: csv::Writer<File>,
}

impl<'a> Writer {
    pub fn new(filename: &'a str) -> Self {
        Writer {
            writer: csv::Writer::from_path(filename).unwrap(),
        }
    }

    pub fn write_gamestate_record(&mut self, record: GamestateRecord) -> Result<(), Box<dyn Error>> {
        self.writer.serialize(&record)?;
        self.writer.flush()?;
        Ok(())
    }

    pub fn write_event_record(&mut self, record: EventRecord) -> Result<(), Box<dyn Error>> {
        self.writer.serialize(&record)?;
        self.writer.flush()?;
        Ok(())
    }
    
    pub fn write_average_record(&mut self, record: AverageRecord) -> Result<(), Box<dyn Error>> {
        self.writer.serialize(&record)?;
        self.writer.flush()?;
        Ok(())
    }

}