use bmi_example_rust::{BmiHeat, TEMPERATURE};
use bmi_rust::{Bmi, BmiResult, RefValues, Values};

fn main() -> BmiResult<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() > 1 {
        return Err("usage: cargo run -- [CONFIGURATION_FILE]".into());
    }
    let mut heat = BmiHeat::default();
    let model: &mut dyn Bmi = &mut heat;
    model.initialize(args.first().map_or("", String::as_str))?;
    let shape = model.get_grid_shape(model.get_var_grid(TEMPERATURE)?)?;
    let center = (shape[0] / 2) * shape[1] + shape[1] / 2;
    model.set_value_at_indices(TEMPERATURE, &[center], RefValues::F64(&[300.0]))?;
    println!(
        "{}: {} x {} nodes",
        model.get_component_name()?,
        shape[0],
        shape[1]
    );
    println!("time_s,center_temperature_K");
    loop {
        let Values::F64(values) = model.get_value_at_indices(TEMPERATURE, &[center])? else {
            return Err("expected f64 temperature".into());
        };
        println!("{:.3},{:.6}", model.get_current_time()?, values[0]);
        if model.get_current_time()? >= model.get_end_time()? {
            break;
        }
        model.update()?;
    }
    model.finalize()
}
