//! A heat model and its Rust Basic Model Interface implementation.

#![forbid(unsafe_code)]

mod heat;

use bmi_rust::{Bmi, BmiResult, GridType, Location, RefValues, Values};
use heat::{Heat, invalid};
use std::io::{Error, ErrorKind};

/// The model's sole input and output variable, measured in kelvin.
pub const TEMPERATURE: &str = "plate_surface__temperature";

/// A BMI wrapper around a two-dimensional heat diffusion solver.
///
/// Create with `default()`, then call `initialize("")` for defaults or supply a
/// configuration path. Grid 0 has shape `[rows, columns]`, unit spacing, and
/// origin `[0, 0]`. Temperatures are flattened with x varying fastest.
#[derive(Default)]
pub struct BmiHeat {
    model: Option<Heat>,
}

impl BmiHeat {
    fn model(&self) -> BmiResult<&Heat> {
        self.model
            .as_ref()
            .ok_or_else(|| invalid("model is not initialized"))
    }

    fn model_mut(&mut self) -> BmiResult<&mut Heat> {
        self.model
            .as_mut()
            .ok_or_else(|| invalid("model is not initialized"))
    }

    fn variable(&self, name: &str) -> BmiResult<&Heat> {
        let model = self.model()?;
        if name != TEMPERATURE {
            return Err(invalid("unknown variable name"));
        }
        Ok(model)
    }

    fn grid(&self, grid: i32) -> BmiResult<&Heat> {
        let model = self.model()?;
        if grid != 0 {
            return Err(invalid("unknown grid identifier"));
        }
        Ok(model)
    }

    fn unsupported<T>(&self, grid: i32) -> BmiResult<T> {
        self.grid(grid)?;
        Err(Error::new(
            ErrorKind::Unsupported,
            "not used by this uniform rectilinear grid",
        )
        .into())
    }

    fn source(src: RefValues<'_>) -> BmiResult<&[f64]> {
        match src {
            RefValues::F64(values) if values.iter().all(|v| v.is_finite()) => Ok(values),
            _ => Err(invalid("temperature values must be finite f64 values")),
        }
    }
}

impl Bmi for BmiHeat {
    fn initialize(&mut self, config_file: &str) -> BmiResult<()> {
        if self.model.is_some() {
            return Err(invalid("model is already initialized"));
        }
        let model = if config_file.is_empty() {
            Heat::new(1.0, 10.0, 20, 10)?
        } else {
            Heat::from_config(&std::fs::read_to_string(config_file)?)?
        };
        self.model = Some(model);
        Ok(())
    }

    fn update(&mut self) -> BmiResult<()> {
        let model = self.model_mut()?;
        if model.time >= model.end_time {
            return Err(invalid("simulation has reached its end time"));
        }
        let target = (model.time + model.time_step).min(model.end_time);
        if target <= model.time {
            return Err(invalid("time step is too small to advance the clock"));
        }
        model.advance_until(target)
    }

    fn update_until(&mut self, time: f64) -> BmiResult<()> {
        self.model_mut()?.advance_until(time)
    }

    fn finalize(&mut self) -> BmiResult<()> {
        self.model()?;
        self.model = None;
        Ok(())
    }

    fn get_component_name(&self) -> BmiResult<String> {
        self.model()?;
        Ok("The 2D Heat Equation".into())
    }

    fn get_input_item_count(&self) -> BmiResult<usize> {
        self.model()?;
        Ok(1)
    }

    fn get_output_item_count(&self) -> BmiResult<usize> {
        self.get_input_item_count()
    }

    fn get_input_var_names(&self) -> BmiResult<Vec<String>> {
        self.model()?;
        Ok(vec![TEMPERATURE.into()])
    }

    fn get_output_var_names(&self) -> BmiResult<Vec<String>> {
        self.get_input_var_names()
    }

    fn get_var_grid(&self, name: &str) -> BmiResult<i32> {
        self.variable(name)?;
        Ok(0)
    }

    fn get_var_type(&self, name: &str) -> BmiResult<String> {
        self.variable(name)?;
        Ok("float64".into())
    }

    fn get_var_units(&self, name: &str) -> BmiResult<String> {
        self.variable(name)?;
        Ok("K".into())
    }

    fn get_var_itemsize(&self, name: &str) -> BmiResult<usize> {
        self.variable(name)?;
        Ok(size_of::<f64>())
    }

    fn get_var_nbytes(&self, name: &str) -> BmiResult<usize> {
        Ok(std::mem::size_of_val(
            self.variable(name)?.temperature.as_slice(),
        ))
    }

    fn get_var_location(&self, name: &str) -> BmiResult<Location> {
        self.variable(name)?;
        Ok(Location::Node)
    }

    fn get_current_time(&self) -> BmiResult<f64> {
        Ok(self.model()?.time)
    }

    fn get_start_time(&self) -> BmiResult<f64> {
        self.model()?;
        Ok(0.0)
    }

    fn get_end_time(&self) -> BmiResult<f64> {
        Ok(self.model()?.end_time)
    }

    fn get_time_units(&self) -> BmiResult<String> {
        self.model()?;
        Ok("s".into())
    }

    fn get_time_step(&self) -> BmiResult<f64> {
        Ok(self.model()?.time_step)
    }

    fn get_value(&self, name: &str) -> BmiResult<Values> {
        Ok(Values::F64(self.variable(name)?.temperature.clone()))
    }

    fn get_value_ptr(&self, name: &str) -> BmiResult<RefValues<'_>> {
        Ok(RefValues::F64(&self.variable(name)?.temperature))
    }

    fn get_value_at_indices(&self, name: &str, indices: &[usize]) -> BmiResult<Values> {
        let values = &self.variable(name)?.temperature;
        let selected: BmiResult<Vec<f64>> = indices
            .iter()
            .map(|&i| {
                values
                    .get(i)
                    .copied()
                    .ok_or_else(|| invalid("temperature index out of bounds"))
            })
            .collect();
        Ok(Values::F64(selected?))
    }

    fn set_value(&mut self, name: &str, src: RefValues<'_>) -> BmiResult<()> {
        let size = self.variable(name)?.temperature.len();
        let src = Self::source(src)?;
        if src.len() != size {
            return Err(invalid("source length must equal the grid size"));
        }
        self.model_mut()?.temperature.copy_from_slice(src);
        Ok(())
    }

    fn set_value_at_indices(
        &mut self,
        name: &str,
        indices: &[usize],
        src: RefValues<'_>,
    ) -> BmiResult<()> {
        let size = self.variable(name)?.temperature.len();
        let src = Self::source(src)?;
        if src.len() != indices.len() || indices.iter().any(|&i| i >= size) {
            return Err(invalid("source length must match valid grid indices"));
        }
        let values = &mut self.model_mut()?.temperature;
        for (&i, &value) in indices.iter().zip(src) {
            values[i] = value;
        }
        Ok(())
    }

    fn get_grid_rank(&self, grid: i32) -> BmiResult<usize> {
        self.grid(grid)?;
        Ok(2)
    }

    fn get_grid_size(&self, grid: i32) -> BmiResult<usize> {
        Ok(self.grid(grid)?.temperature.len())
    }

    fn get_grid_type(&self, grid: i32) -> BmiResult<GridType> {
        self.grid(grid)?;
        Ok(GridType::UniformRectilinear)
    }

    fn get_grid_shape(&self, grid: i32) -> BmiResult<Vec<usize>> {
        Ok(self.grid(grid)?.shape.to_vec())
    }

    fn get_grid_spacing(&self, grid: i32) -> BmiResult<Vec<f64>> {
        self.grid(grid)?;
        Ok(vec![1.0, 1.0])
    }

    fn get_grid_origin(&self, grid: i32) -> BmiResult<Vec<f64>> {
        self.grid(grid)?;
        Ok(vec![0.0, 0.0])
    }

    fn get_grid_x(&self, grid: i32) -> BmiResult<Vec<f64>> {
        self.unsupported(grid)
    }
    fn get_grid_y(&self, grid: i32) -> BmiResult<Vec<f64>> {
        self.unsupported(grid)
    }
    fn get_grid_z(&self, grid: i32) -> BmiResult<Vec<f64>> {
        self.unsupported(grid)
    }

    fn get_grid_node_count(&self, grid: i32) -> BmiResult<usize> {
        self.get_grid_size(grid)
    }
    fn get_grid_edge_count(&self, grid: i32) -> BmiResult<usize> {
        self.unsupported(grid)
    }
    fn get_grid_face_count(&self, grid: i32) -> BmiResult<usize> {
        self.unsupported(grid)
    }
    fn get_grid_edge_nodes(&self, grid: i32) -> BmiResult<Vec<usize>> {
        self.unsupported(grid)
    }
    fn get_grid_face_edges(&self, grid: i32) -> BmiResult<Vec<usize>> {
        self.unsupported(grid)
    }
    fn get_grid_face_nodes(&self, grid: i32) -> BmiResult<Vec<usize>> {
        self.unsupported(grid)
    }
    fn get_grid_nodes_per_face(&self, grid: i32) -> BmiResult<Vec<usize>> {
        self.unsupported(grid)
    }
}
