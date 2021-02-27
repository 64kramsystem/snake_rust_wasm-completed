use std::ops::{Add, Mul, Neg, Sub};

use js_sys::Array;
use rand::seq::SliceRandom;
use wasm_bindgen::prelude::*;

trait ApproximateEq {
    const X_EPSILON: f64 = 0.00001;

    fn approximate_eq(&self, other: Self) -> bool;
}

impl ApproximateEq for f64 {
    fn approximate_eq(&self, other: f64) -> bool {
        (self - other).abs() < f64::EPSILON
    }
}

#[wasm_bindgen]
#[derive(Clone, Copy)]
pub struct Vector {
    pub x: f64,
    pub y: f64,
}

#[wasm_bindgen]
impl Vector {
    #[wasm_bindgen(constructor)]
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    // We need this because it's not possible to export the trait implementation.
    //
    pub fn scale(self, value: f64) -> Self {
        self * value
    }

    pub fn length(&self) -> f64 {
        self.x.hypot(self.y)
    }

    pub fn normalize(self) -> Vector {
        self * (1.0 / self.length())
    }
}

impl PartialEq for Vector {
    fn eq(&self, other: &Self) -> bool {
        self.x.approximate_eq(other.x) && self.y.approximate_eq(other.y)
    }
}

impl Add<Vector> for Vector {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub<Vector> for Vector {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Mul<f64> for Vector {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl Neg for Vector {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
        }
    }
}

impl From<&Vector> for JsValue {
    fn from(vector: &Vector) -> Self {
        JsValue::from(vector.clone())
    }
}

struct Segment {
    pub start: Vector,
    pub end: Vector,
}

impl Segment {
    pub fn new(start: Vector, end: Vector) -> Self {
        Self { start, end }
    }

    pub fn is_point_inside(&self, point: Vector) -> bool {
        let first = Segment::new(self.start, point);
        let second = Segment::new(point, self.end);

        self.length()
            .approximate_eq(first.length() + second.length())
    }

    fn vector(&self) -> Vector {
        self.end - self.start
    }

    fn length(&self) -> f64 {
        self.vector().length()
    }
}

fn generate_food_position(width: i32, height: i32, snake: &[Vector]) -> Vector {
    let mut free_positions: Vec<Vector> = Vec::new();

    let segments = snake
        .windows(2)
        .map(|points| Segment::new(points[0], points[1]))
        .collect::<Vec<_>>();

    for x in 0..width {
        for y in 0..height {
            let point = Vector::new(x as f64 + 0.5, y as f64 + 0.5);

            if !segments
                .iter()
                .any(|segment| segment.is_point_inside(point))
            {
                free_positions.push(point);
            }
        }
    }

    free_positions
        .choose(&mut rand::thread_rng())
        .unwrap()
        .clone()
}

#[wasm_bindgen]
pub enum Movement {
    Top,
    Right,
    Down,
    Left,
}

impl Movement {
    fn vector(&self) -> Vector {
        let (new_x, new_y) = match self {
            Movement::Top => (0.0, -1.0),
            Movement::Right => (1.0, 0.0),
            Movement::Down => (0.0, 1.0),
            Movement::Left => (-1.0, 0.0),
        };
        Vector::new(new_x, new_y)
    }
}
#[wasm_bindgen]
pub struct Game {
    pub width: i32,
    pub height: i32,
    pub speed: f64,
    pub score: i32,
    pub direction: Vector,
    pub food: Vector,
    snake: Vec<Vector>,
}

#[wasm_bindgen]
impl Game {
    #[wasm_bindgen(constructor)]
    pub fn new(width: i32, height: i32, speed: f64, snake_length: i32, direction: Vector) -> Self {
        let head = Vector::new(
            (width as f64 / 2.0).round() - 0.5,
            (height as f64 / 2.0).round() - 0.5,
        );

        let tail_tip = head - direction * snake_length as f64;

        let snake = vec![tail_tip, head];

        let food = generate_food_position(width, height, &snake);

        Game {
            width,
            height,
            speed,
            score: 0,
            direction,
            food,
            snake,
        }
    }

    pub fn snake(&self) -> Array {
        self.snake.iter().map(JsValue::from).collect()
    }

    fn process_movement(&mut self, timespan: f64, movement: Option<Movement>) {
        let mut new_snake: Vec<Vector> = Vec::new();

        let full_distance = self.speed * timespan;
        let mut remaining_distance = full_distance;

        while self.snake.len() > 1 {
            let point = self.snake.remove(0);
            let next = self.snake[0];
            let segment = Segment::new(point, next);
            let length = segment.length();

            if length >= remaining_distance {
                let vector = segment.vector().normalize() * remaining_distance;
                new_snake.push(point.add(vector));
                break;
            } else {
                remaining_distance -= length;
            }
        }
        new_snake.append(&mut self.snake);
        self.snake = new_snake;

        let old_head = self.snake.pop().unwrap();
        let new_head = old_head.add(self.direction * full_distance);

        if let Some(movement) = movement {
            let new_direction = movement.vector();

            if self.direction != -new_direction && self.direction != new_direction {
                let Vector { x: old_x, y: old_y } = old_head;
                let old_x_rounded = old_x.round();
                let old_y_rounded = old_y.round();
                let new_x_rounded = new_head.x.round();
                let new_y_rounded = new_head.y.round();

                let rounded_x_changed = old_x_rounded != new_x_rounded;
                let rounded_y_changed = old_y_rounded != new_y_rounded;

                if rounded_x_changed || rounded_y_changed {
                    let (old, old_rounded, new_rounded) = if rounded_x_changed {
                        (old_x, old_x_rounded, new_x_rounded)
                    } else {
                        (old_y, old_y_rounded, new_y_rounded)
                    };
                    let breakpoint_component = if new_rounded > old_rounded {
                        old_rounded + 0.5
                    } else {
                        old_rounded - 0.5
                    };
                    let breakpoint = if rounded_x_changed {
                        Vector::new(breakpoint_component, old_y)
                    } else {
                        Vector::new(old_x, breakpoint_component)
                    };
                    let vector =
                        new_direction * (full_distance - (old - breakpoint_component).abs());
                    let head = breakpoint + vector;

                    self.snake.push(breakpoint);
                    self.snake.push(head);
                    self.direction = new_direction;

                    return;
                }
            }
        }

        self.snake.push(new_head);
    }

    pub fn process(&mut self, timespan: f64, movement: Option<Movement>) {
        self.process_movement(timespan, movement);
    }
}
