use crate::camera::Camera;
use crate::primitive;
use crate::primitive::{Cube, Light};
use glam::Vec3;
use iced::advanced::Shell;
use iced::event::Status;
use iced::mouse::Cursor;
use iced::{mouse, Color, Point, Rectangle};
use iced_graphics::custom::{Event, Program};
use rand::Rng;
use std::cmp::Ordering;
use std::iter;
use std::time::Duration;

#[derive(Clone)]
pub struct Cubes {
    // how big cubes are
    pub size: f32,
    // cube origins
    pub cubes: Vec<Cube>,
    // current camera transform
    pub camera: Camera,
    // whether or not to show the depth buffer
    pub show_depth_buffer: bool,
    // light color
    pub light_color: Color,
}

impl Cubes {
    pub fn new() -> Self {
        let mut cubes = Self {
            size: 0.2,
            cubes: vec![],
            camera: Default::default(),
            show_depth_buffer: false,
            light_color: Color::WHITE,
        };

        cubes.adjust_num_cubes(500);

        cubes
    }

    pub fn update(&mut self, time: Duration) {
        for cube in self.cubes.iter_mut() {
            cube.update(self.size, time.as_secs_f32());
        }
    }

    pub fn adjust_num_cubes(&mut self, num_cubes: u32) {
        let curr_cubes = self.cubes.len() as u32;

        match num_cubes.cmp(&curr_cubes) {
            Ordering::Greater => {
                // spawn
                let cubes_2_spawn = (num_cubes - curr_cubes) as usize;

                let mut cubes = 0;
                self.cubes.extend(iter::from_fn(|| {
                    if cubes < cubes_2_spawn {
                        cubes += 1;
                        Some(Cube::new(self.size, rnd_origin()))
                    } else {
                        None
                    }
                }));
            }
            Ordering::Less => {
                // chop
                let cubes_2_cut = curr_cubes - num_cubes;
                let new_len = self.cubes.len() - cubes_2_cut as usize;
                self.cubes.truncate(new_len);
            }
            _ => {}
        }
    }
}

impl<Message> Program<Message> for Cubes {
    type State = Interaction;
    type Primitive = primitive::Primitive;

    fn draw(
        &self,
        state: &Self::State,
        cursor: mouse::Cursor,
        _bounds: Rectangle,
    ) -> Self::Primitive {
        primitive::Primitive::new(
            &self.cubes,
            &self.camera,
            self.show_depth_buffer,
            state.position(),
            Light::new(cursor, self.light_color),
        )
    }

    fn update(
        &mut self,
        state: &mut Self::State,
        _event: Event,
        bounds: Rectangle,
        cursor: Cursor,
        _shell: &mut Shell<'_, Message>,
    ) -> (Status, Option<Message>) {
        let status = if let Some(position) = cursor.position_in(bounds) {
            *state = Interaction::Moving(position);
            Status::Captured
        } else {
            *state = Interaction::None;
            Status::Ignored
        };

        (status, None)
    }
}

#[derive(Default)]
pub enum Interaction {
    #[default]
    None,
    Moving(Point),
}

impl Interaction {
    fn position(&self) -> Point {
        match self {
            Interaction::None => Point::new(-1.0, -1.0),
            Interaction::Moving(pos) => *pos,
        }
    }
}

pub fn rnd_origin() -> Vec3 {
    Vec3::new(
        rand::thread_rng().gen_range(-4.0..4.0),
        rand::thread_rng().gen_range(-4.0..4.0),
        rand::thread_rng().gen_range(-4.0..2.0),
    )
}
