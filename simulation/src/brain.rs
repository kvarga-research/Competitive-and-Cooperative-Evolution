use nalgebra::base::{MatrixMN, Matrix};

use crate::entity::Entity;
use crate::random_helper::RandomHelper;


pub type InputProcessLayer1 = MatrixMN<f32, nalgebra::U7, nalgebra::U5>;
pub type InputProcessLayer2 = MatrixMN<f32, nalgebra::U3, nalgebra::U7>;

#[derive(Clone)]
pub struct InputProcessorNetwork {
    rand: RandomHelper,
    layer1: InputProcessLayer1,
    layer2: InputProcessLayer2,
}

impl InputProcessorNetwork {
    pub fn new(seed: u64) -> Self {
        // screen size is not important, because the RandomHelper is used to create new random neuron values 
        let mut rand = RandomHelper::new(500.0, 500.0, seed);
        let mut layer1: InputProcessLayer1 = MatrixMN::zeros_generic(nalgebra::U7, nalgebra::U5);
        layer1 = layer1.map(|_| rand.random_between(-1.0, 1.0));
        let mut layer2: InputProcessLayer2 = MatrixMN::zeros_generic(nalgebra::U3, nalgebra::U7);
        layer2 = layer2.map(|_| rand.random_between(-1.0, 1.0));
        InputProcessorNetwork {
            rand: rand,
            layer1: layer1,
            layer2: layer2,
        }
    }

    pub fn mutate(& mut self) {
        let layer = self.rand.random_between(0.0, 2.0) as i32;
        match layer {
            0 => {
                self.layer1[(
                    self.rand.random_between(0.0, 7.0) as usize, self.rand.random_between(0.0, 5.0) as usize
                )] = self.rand.random_between(- 1.0, 1.0);
            }
            1 => {
                self.layer2[(
                    self.rand.random_between(0.0, 3.0) as usize, self.rand.random_between(0.0, 7.0) as usize
                )] = self.rand.random_between(- 1.0, 1.0);
            }
            _ => (),
        }
    }
}

#[derive(Clone)]
pub struct BrainNetwork {
    rand: RandomHelper,
    pub wall_network: InputProcessorNetwork,
    pub food_network: InputProcessorNetwork,
    pub carnivore_network: InputProcessorNetwork,
}

impl BrainNetwork {
    pub fn new(seed: u64) -> Self {
        // screen size is not important here, because the RandomHelper is used to create new random neuron values
        let rand = RandomHelper::new(500.0, 500.0, seed);
        
        BrainNetwork {
            rand: rand,
            wall_network: InputProcessorNetwork::new(seed + 1),
            food_network: InputProcessorNetwork::new(seed + 2),
            carnivore_network: InputProcessorNetwork::new(seed + 3),
        }
    }

    pub fn mutate(& mut self) {
        let network = self.rand.random_between(0.0, 3.0) as i32;
        match network {
            0 => {
                self.wall_network.mutate();
            }
            1 => {
                self.food_network.mutate();
            }
            2 => {
                self.carnivore_network.mutate();
            }
            _ => (),
        }
    }
}

pub struct Brain {
    rand: RandomHelper,
    view_range: f32,
    mutation_rate: f32,
    brain_network: BrainNetwork,
}

impl Brain {
    pub fn new(view_range: f32, mutation_rate: f32, seed: u64) -> Self {
        let rand = RandomHelper::new(500.0, 500.0, seed);
        Brain {
            rand: rand,
            view_range: view_range,
            mutation_rate: mutation_rate,
            brain_network: BrainNetwork::new(seed + 777),
        }
    }

    pub fn get_networks(& self) -> BrainNetwork {
        self.brain_network.clone()
    }

    pub fn set_networks(& mut self, brain_network: BrainNetwork) {
        self.brain_network = brain_network;
    }

    pub fn mutate(& mut self) {
        if (self.mutation_rate as f32) < self.rand.random_between(0.0, 1.0) {
            self.brain_network.mutate();
        }
    }

    pub fn get_new_direction(&self, closest_objects: Vec<Option<(Entity, f32)>>, brain_entity: Entity, facing: i8) -> i8 {
        let step = 5;
        let rays = closest_objects.len();
        let input_size = step * rays;
        let mut input_vec = vec![0.0; input_size];
        if brain_entity == Entity::HERBIVORE {
            for i in (0..input_size).step_by(step) {
                if let Some((entity, toi)) = &closest_objects[i / step] {
                    match entity {
                        Entity::FOOD => input_vec[i] = 1.0,
                        Entity::CARNIVORE => {
                            input_vec[i + 1] = 1.0;
                            },
                            Entity::WALL => {
                                input_vec[i + 2] = 1.0;
                                },
                        _ => (),
                    }
                    input_vec[i + 3] = 1.0 - *toi / self.view_range;
                }
                else {
                    input_vec[i + 4] = 1.0;
                }
            }
        }
        else {
            for i in (0..input_size).step_by(step) {
                if let Some((entity, toi)) = &closest_objects[i / step] {
                    match entity {
                        Entity::FOOD => (),
                        Entity::HERBIVORE => {
                            input_vec[i] = 1.0;
                        },
                        Entity::CARNIVORE => input_vec[i + 1] = 1.0,
                        Entity::WALL => {
                            input_vec[i + 2] = 1.0;
                        },
                        _ => (),
                    }
                    input_vec[i + 3] = 1.0 - *toi / self.view_range;
                }
                else {
                    input_vec[i + 4] = 1.0;
                }
            }
        }
        
        let mut inputs = Vec::new();
        for j in 0..3 {
            for i in (0..input_size).step_by(step) {
                inputs.push(input_vec[i + j] * input_vec[i + 3]);
            }
        }

        let mut max_i = 1;
        let mut relevant_inputs = Vec::new();
        for i in (0..24).step_by(8) {
            relevant_inputs.push(inputs[(facing as usize + 6) % 8 + i]);
            relevant_inputs.push(inputs[(facing as usize + 7) % 8 + i]);
            relevant_inputs.push(inputs[(facing as usize) % 8 + i]);
            relevant_inputs.push(inputs[(facing as usize + 1) % 8 + i]);
            relevant_inputs.push(inputs[(facing as usize + 2) % 8 + i]);
        }

        let input1 = Matrix::<f32, nalgebra::U5, nalgebra::U1, _>::from_vec((&relevant_inputs[..5]).to_vec());
        let mut first_output1 = self.brain_network.food_network.layer1 * input1;
        first_output1 = first_output1.map(Brain::sigmoid);
        let mut first_output2 = self.brain_network.food_network.layer2 * first_output1;
        first_output2 = first_output2.map(Brain::sigmoid);
        let input2 = Matrix::<f32, nalgebra::U5, nalgebra::U1, _>::from_vec((&relevant_inputs[5..10]).to_vec());
        let mut second_output1 = self.brain_network.carnivore_network.layer1 * input2;
        second_output1 = second_output1.map(Brain::sigmoid);
        let mut second_output2 = self.brain_network.carnivore_network.layer2 * second_output1;
        second_output2 = second_output2.map(Brain::sigmoid);
        let input3 = Matrix::<f32, nalgebra::U5, nalgebra::U1, _>::from_vec((&relevant_inputs[10..]).to_vec());
        let mut third_output1 = self.brain_network.wall_network.layer1 * input3;
        third_output1 = third_output1.map(Brain::sigmoid);
        let mut third_output2 = self.brain_network.wall_network.layer2 * third_output1;
        third_output2 = third_output2.map(Brain::sigmoid);
        for i in 0..3 {
            if first_output2[i] + second_output2[i] + third_output2[i] > first_output2[max_i] + second_output2[max_i] + third_output2[max_i] {
                max_i = i;
            }
        }

        match max_i {
            0 => (facing + 7) % 8,
            1 => facing,
            2 => (facing + 1) % 8,
            _ => -1,
        }
    }

    pub fn max_sensor_distance(&self) -> f32 {
        self.view_range
    }

    pub fn sigmoid(num: f32) -> f32 {
        num / (num.abs() + 0.5)
    }
}