use bmi_rust::BmiResult;
use std::io::{Error, ErrorKind};

pub(crate) fn invalid(message: &str) -> Box<dyn std::error::Error + Send + Sync> {
    Error::new(ErrorKind::InvalidInput, message).into()
}

/// Explicit finite-difference solver with fixed boundary temperatures.
pub(crate) struct Heat {
    pub temperature: Vec<f64>,
    next: Vec<f64>,
    pub shape: [usize; 2],
    pub time: f64,
    pub time_step: f64,
    pub end_time: f64,
    alpha: f64,
}

impl Heat {
    pub fn new(alpha: f64, end_time: f64, nx: usize, ny: usize) -> BmiResult<Self> {
        let time_step = 1.0 / (4.0 * alpha);
        if !alpha.is_finite() || alpha <= 0.0 || !time_step.is_finite() || time_step <= 0.0 {
            return Err(invalid(
                "diffusivity must give a finite, positive time step",
            ));
        }
        if !end_time.is_finite() || end_time <= 0.0 {
            return Err(invalid("end time must be finite and positive"));
        }
        if nx < 3 || ny < 3 {
            return Err(invalid("the grid must have at least three nodes per axis"));
        }
        let size = nx
            .checked_mul(ny)
            .ok_or_else(|| invalid("grid size overflow"))?;
        let mut temperature = Vec::new();
        temperature.try_reserve_exact(size)?;
        temperature.resize(size, 0.0);
        let mut next = Vec::new();
        next.try_reserve_exact(size)?;
        next.resize(size, 0.0);
        Ok(Self {
            temperature,
            next,
            shape: [ny, nx],
            time: 0.0,
            time_step,
            end_time,
            alpha,
        })
    }

    pub fn from_config(config: &str) -> BmiResult<Self> {
        let fields: Vec<_> = config.trim().split(',').map(str::trim).collect();
        if fields.len() != 4 {
            return Err(invalid("expected diffusivity, end_time, columns, rows"));
        }
        Self::new(
            fields[0].parse()?,
            fields[1].parse()?,
            fields[2].parse()?,
            fields[3].parse()?,
        )
    }

    pub fn advance_until(&mut self, target: f64) -> BmiResult<()> {
        if !target.is_finite() || target < self.time || target > self.end_time {
            return Err(invalid(
                "target time must be finite and between current and end time",
            ));
        }
        while self.time < target {
            let dt = self.time_step.min(target - self.time);
            let next_time = (self.time + dt).min(target);
            if next_time <= self.time {
                return Err(invalid("time step is too small to advance the clock"));
            }
            let [ny, nx] = self.shape;
            self.next.copy_from_slice(&self.temperature);
            for row in 1..ny - 1 {
                for col in 1..nx - 1 {
                    let i = row * nx + col;
                    let laplacian = self.temperature[i - nx]
                        + self.temperature[i + nx]
                        + self.temperature[i - 1]
                        + self.temperature[i + 1]
                        - 4.0 * self.temperature[i];
                    self.next[i] = self.temperature[i] + self.alpha * dt * laplacian;
                }
            }
            // Keep the same allocation so direct BMI borrows always address model storage.
            self.temperature.copy_from_slice(&self.next);
            self.time = next_time;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn malformed_and_invalid_configurations_are_rejected() {
        for config in [
            "",
            "1, 10, 20",
            "1, 10, 20, 10, 5",
            "abc, 10, 20, 10",
            "0, 10, 20, 10",
            "-1, 10, 20, 10",
            "NaN, 10, 20, 10",
            "inf, 10, 20, 10",
            "1e308, 10, 20, 10",
            "1e-320, 10, 20, 10",
            "1, 0, 20, 10",
            "1, NaN, 20, 10",
            "1, inf, 20, 10",
            "1, 10, 2, 10",
            "1, 10, 20, 2",
            "1, 10, -20, 10",
            "1, 10, 18446744073709551615, 10",
        ] {
            assert!(Heat::from_config(config).is_err(), "accepted {config:?}");
        }
    }
}
