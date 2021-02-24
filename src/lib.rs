use std::ops::{Mul, Sub};

use js_sys::Array;
use wasm_bindgen::prelude::*;

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

        let food = Vector::new(0.5, 0.5);

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
