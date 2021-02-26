use std::ops::{Mul, Sub};

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
}
