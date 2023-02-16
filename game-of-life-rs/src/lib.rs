use rand::Rng;
use wasm_bindgen::prelude::*;

mod utils;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    Dead = 0,
    Alive = 1,
}

#[wasm_bindgen]
pub struct Universe {
    width: u32,
    height: u32,
    cells: Vec<Cell>,
    cell_ages: Vec<u32>,
    generation: u32,
    cell_generations: Vec<u32>,
    activity: f32,
}

#[wasm_bindgen]
impl Universe {
    pub fn new(width: u32, height: u32) -> Universe {
        utils::set_panic_hook();

        let cells = (0..width * height)
            .map(|i| {
                if i % 2 == 0 || i % 7 == 0 {
                    Cell::Alive
                } else {
                    Cell::Dead
                }
            })
            .collect();

        Universe {
            width,
            height,
            cells,
            cell_ages: vec![0; (width * height) as usize],
            generation: 0,
            cell_generations: vec![0; (width * height) as usize],
            activity: 0.0,
        }
    }

    #[wasm_bindgen(constructor)]
    pub fn new_with_lifetime(
        width: u32,
        height: u32,
        min_lifetime: u32,
        min_activity: f32,
    ) -> Universe {
        let mut rng = rand::thread_rng();

        // Loop until we find a universe that maintains `min_activity`
        // for at least `min_lifetime` generations
        loop {
            let cells: Vec<Cell> = (0..width * height)
                .map(|_| {
                    if rng.gen_bool(0.5) {
                        Cell::Alive
                    } else {
                        Cell::Dead
                    }
                })
                .collect();
            let original_cells = cells.clone(); // Store original cells to reset universe

            let mut universe = Universe {
                width,
                height,
                cells,
                cell_ages: vec![0; (width * height) as usize],
                generation: 0,
                cell_generations: vec![0; (width * height) as usize],
                activity: 0.0,
            };

            // Run the universe for `min_lifetime` generations, or until it becomes stagnant
            for _ in 0..min_lifetime {
                universe.tick();
                if universe.activity < min_activity {
                    break;
                }
            }

            // If the universe is stagnant, try again
            if universe.activity < min_activity {
                continue;
            }

            universe.set_cells(&original_cells);
            return universe;
        }
    }

    fn get_index(&self, row: u32, column: u32) -> usize {
        (row * self.width + column) as usize
    }

    fn live_neighbour_count(&self, row: u32, column: u32) -> u8 {
        let mut count = 0;
        for delta_row in [self.height - 1, 0, 1].iter().cloned() {
            for delta_col in [self.width - 1, 0, 1].iter().cloned() {
                if delta_row == 0 && delta_col == 0 {
                    continue;
                }

                let neighbour_row = (row + delta_row) % self.height;
                let neighbour_col = (column + delta_col) % self.width;
                let idx = self.get_index(neighbour_row, neighbour_col);
                count += self.cells[idx] as u8;
            }
        }
        count
    }

    pub fn tick(&mut self) {
        let mut next = self.cells.clone();
        let mut active_cells = 0; // Number of cells that changed state this tick

        for row in 0..self.height {
            for col in 0..self.width {
                let idx = self.get_index(row, col);
                let cell = self.cells[idx];
                let live_neighbours = self.live_neighbour_count(row, col);

                let next_cell = match (cell, live_neighbours) {
                    // Rule 1: Any live cell with fewer than two live neighbours
                    // dies, as if caused by underpopulation.
                    (Cell::Alive, x) if x < 2 => Cell::Dead,
                    // Rule 2: Any live cell with two or three live neighbours
                    // lives on to the next generation.
                    (Cell::Alive, 2) | (Cell::Alive, 3) => Cell::Alive,
                    // Rule 3: Any live cell with more than three live
                    // neighbours dies, as if by overpopulation.
                    (Cell::Alive, x) if x > 3 => Cell::Dead,
                    // Rule 4: Any dead cell with exactly three live neighbours
                    // becomes a live cell, as if by reproduction.
                    (Cell::Dead, 3) => Cell::Alive,
                    // All other cells remain in the same state.
                    (otherwise, _) => otherwise,
                };

                match (cell, next_cell) {
                    // Cells that came alive this tick should have their generation
                    // set to the current generation
                    (Cell::Dead, Cell::Alive) => {
                        self.cell_generations[idx] = self.generation;
                        active_cells += 1;
                    }
                    // Cells that died this tick should have their age reset to 0
                    (Cell::Alive, Cell::Dead) => {
                        self.cell_ages[idx] = 0;
                        active_cells += 1;
                    }
                    // Cells that are alive should have their age incremented
                    (Cell::Alive, Cell::Alive) => self.cell_ages[idx] += 1,
                    // Otherwise, do nothing
                    (_, _) => (),
                }

                next[idx] = next_cell;
            }
        }

        self.cells = next;
        self.generation += 1;
        self.activity = active_cells as f32 / (self.width * self.height) as f32;
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn cells(&self) -> *const Cell {
        self.cells.as_ptr()
    }

    pub fn cell_ages(&self) -> *const u32 {
        self.cell_ages.as_ptr()
    }

    pub fn generation(&self) -> u32 {
        self.generation
    }

    pub fn cell_generations(&self) -> *const u32 {
        self.cell_generations.as_ptr()
    }

    pub fn activity(&self) -> f32 {
        self.activity
    }

    pub fn toggle_cell(&mut self, row: u32, column: u32) {
        let idx = self.get_index(row, column);
        self.cells[idx].toggle();
    }

    fn set_cells(&mut self, cells: &[Cell]) {
        assert_eq!(cells.len(), (self.width * self.height) as usize);
        self.cells = cells.to_vec();
        self.generation = 0;
        self.cell_ages = vec![0; (self.width * self.height) as usize];
        self.cell_generations = vec![0; (self.width * self.height) as usize];
    }
}

impl Default for Universe {
    fn default() -> Self {
        Self::new(200, 100)
    }
}

impl Cell {
    fn toggle(&mut self) {
        *self = match *self {
            Cell::Dead => Cell::Alive,
            Cell::Alive => Cell::Dead,
        }
    }
}
