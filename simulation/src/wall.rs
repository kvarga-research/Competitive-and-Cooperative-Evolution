use ggez::{graphics, Context, GameResult};
use ggez::nalgebra::Point2;
use nalgebra;

use crate::randomwalker::RandomWalker;


pub struct Wall {
    first: Point2<f32>,
    second: Point2<f32>,
}

impl Wall {
    pub fn new(first: nalgebra::Point2<f32>, second: nalgebra::Point2<f32>) -> Self {
        Wall {
            first: RandomWalker::convert_point(first),
            second: RandomWalker::convert_point(second),
        }
    }

    pub fn draw(&self, ctx: &mut Context) -> GameResult<()> {
        let color = [1.0, 1.0, 1.0, 1.0].into();

        let line = graphics::Mesh::new_line(
            ctx,
            &[self.first, self.second],
            10.0,
            color,
        )?;
        let drawpar = graphics::DrawParam::new();
        graphics::draw(ctx, &line, drawpar)?;
        Ok(())
    }
}