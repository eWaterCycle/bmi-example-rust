use bmi_example_rust::{BmiHeat, TEMPERATURE};
use bmi_rust::{Bmi, BmiResult, GridType, Location, RefValues, Values};

fn initialized() -> BmiHeat {
    let mut model = BmiHeat::default();
    model.initialize("").unwrap();
    model
}

fn values(model: &dyn Bmi) -> Vec<f64> {
    let Values::F64(values) = model.get_value(TEMPERATURE).unwrap() else {
        panic!("expected f64 values");
    };
    values
}

#[test]
fn lifecycle_releases_storage_and_allows_reinitialization() -> BmiResult<()> {
    let mut model = BmiHeat::default();
    assert!(model.update().is_err());
    assert!(model.get_value(TEMPERATURE).is_err());
    assert!(model.finalize().is_err());
    model.initialize("")?;
    assert!(model.initialize("").is_err());
    assert_eq!(model.get_current_time()?, 0.0);
    model.finalize()?;
    assert!(model.get_grid_shape(0).is_err());
    assert!(model.get_current_time().is_err());
    model.initialize("")?;
    assert_eq!(model.get_current_time()?, 0.0);
    Ok(())
}

#[test]
fn metadata_describes_temperature_on_a_uniform_grid() -> BmiResult<()> {
    let model = initialized();
    assert_eq!(model.get_component_name()?, "The 2D Heat Equation");
    assert_eq!(model.get_input_var_names()?, [TEMPERATURE]);
    assert_eq!(model.get_output_var_names()?, [TEMPERATURE]);
    assert_eq!(model.get_input_item_count()?, 1);
    assert_eq!(model.get_output_item_count()?, 1);
    assert_eq!(model.get_var_grid(TEMPERATURE)?, 0);
    assert_eq!(model.get_var_type(TEMPERATURE)?, "float64");
    assert_eq!(model.get_var_units(TEMPERATURE)?, "K");
    assert_eq!(model.get_var_location(TEMPERATURE)?, Location::Node);
    assert_eq!(model.get_var_itemsize(TEMPERATURE)?, 8);
    assert_eq!(model.get_var_nbytes(TEMPERATURE)?, 200 * 8);
    assert_eq!(model.get_grid_type(0)?, GridType::UniformRectilinear);
    assert_eq!(model.get_grid_rank(0)?, 2);
    assert_eq!(model.get_grid_shape(0)?, [10, 20]);
    assert_eq!(model.get_grid_size(0)?, 200);
    assert_eq!(model.get_grid_node_count(0)?, 200);
    assert_eq!(model.get_grid_spacing(0)?, [1.0, 1.0]);
    assert_eq!(model.get_grid_origin(0)?, [0.0, 0.0]);
    assert_eq!(model.get_start_time()?, 0.0);
    assert_eq!(model.get_end_time()?, 10.0);
    assert_eq!(model.get_time_step()?, 0.25);
    assert_eq!(model.get_time_units()?, "s");
    Ok(())
}

#[test]
fn reads_copy_or_borrow_and_index_order_is_preserved() -> BmiResult<()> {
    let mut model = initialized();
    model.set_value_at_indices(TEMPERATURE, &[5, 6], RefValues::F64(&[10.0, 20.0]))?;
    let snapshot = values(&model);
    assert_eq!(
        model.get_value_at_indices(TEMPERATURE, &[6, 5, 6])?,
        Values::F64(vec![20.0, 10.0, 20.0])
    );
    assert_eq!(
        model.get_value_at_indices(TEMPERATURE, &[])?,
        Values::F64(vec![])
    );
    let RefValues::F64(borrowed) = model.get_value_ptr(TEMPERATURE)? else {
        panic!("expected f64 borrow");
    };
    assert_eq!(borrowed[5], 10.0);
    model.set_value_at_indices(TEMPERATURE, &[5, 5], RefValues::F64(&[30.0, 40.0]))?;
    assert_eq!(values(&model)[5], 40.0);
    assert_eq!(snapshot[5], 10.0);
    Ok(())
}

#[test]
fn invalid_setters_leave_all_values_unchanged() -> BmiResult<()> {
    let mut model = initialized();
    let before = values(&model);
    assert!(
        model
            .set_value("unknown", RefValues::F64(&[1.0; 200]))
            .is_err()
    );
    assert!(
        model
            .set_value(TEMPERATURE, RefValues::F32(&[1.0; 200]))
            .is_err()
    );
    assert!(
        model
            .set_value(TEMPERATURE, RefValues::F64(&[1.0]))
            .is_err()
    );
    assert!(
        model
            .set_value(TEMPERATURE, RefValues::F64(&[f64::NAN; 200]))
            .is_err()
    );
    assert!(
        model
            .set_value_at_indices(TEMPERATURE, &[1, 200], RefValues::F64(&[1.0, 2.0]))
            .is_err()
    );
    assert!(
        model
            .set_value_at_indices(TEMPERATURE, &[1, 2], RefValues::F64(&[1.0]))
            .is_err()
    );
    assert!(
        model
            .set_value_at_indices(TEMPERATURE, &[1], RefValues::F64(&[f64::INFINITY]))
            .is_err()
    );
    assert!(model.get_value_at_indices(TEMPERATURE, &[200]).is_err());
    assert_eq!(values(&model), before);
    Ok(())
}

#[test]
fn heat_spreads_using_a_five_point_stencil() -> BmiResult<()> {
    let mut model = initialized();
    let center = 5 * 20 + 10;
    model.set_value_at_indices(TEMPERATURE, &[center], RefValues::F64(&[300.0]))?;
    model.update_until(0.125)?;
    let result = values(&model);
    assert_eq!(result[center], 150.0);
    for i in [center - 20, center + 20, center - 1, center + 1] {
        assert_eq!(result[i], 37.5);
    }
    assert_eq!(result.iter().sum::<f64>(), 300.0);
    assert_eq!(result[center + 21], 0.0);
    assert_eq!(model.get_current_time()?, 0.125);
    assert_eq!(model.get_time_step()?, 0.25);
    Ok(())
}

#[test]
fn fixed_boundaries_preserve_a_uniform_temperature() -> BmiResult<()> {
    let mut model = initialized();
    model.set_value(TEMPERATURE, RefValues::F64(&[273.0; 200]))?;
    model.update_until(model.get_end_time()?)?;
    assert_eq!(values(&model), vec![273.0; 200]);
    assert!(model.update().is_err());
    Ok(())
}

#[test]
fn fractional_updates_match_explicit_steps_and_reject_invalid_times() -> BmiResult<()> {
    let mut direct = initialized();
    let mut split = initialized();
    for model in [&mut direct, &mut split] {
        model.set_value_at_indices(TEMPERATURE, &[110], RefValues::F64(&[300.0]))?;
    }
    direct.update_until(0.375)?;
    split.update()?;
    split.update_until(0.375)?;
    assert_eq!(values(&direct), values(&split));
    let before = values(&direct);
    direct.update_until(0.375)?;
    for time in [-1.0, 0.25, 11.0, f64::NAN, f64::INFINITY] {
        assert!(direct.update_until(time).is_err());
    }
    assert_eq!(direct.get_current_time()?, 0.375);
    assert_eq!(values(&direct), before);
    Ok(())
}

#[test]
fn configuration_controls_shape_diffusivity_and_end_time() -> BmiResult<()> {
    let mut model = BmiHeat::default();
    assert!(model.initialize("nonexistent-heat-config.cfg").is_err());
    model.initialize(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/heat.cfg"))?;
    assert_eq!(model.get_grid_shape(0)?, [4, 5]);
    assert_eq!(model.get_time_step()?, 0.5);
    assert_eq!(model.get_end_time()?, 1.0);
    model.update_until(0.9)?;
    model.update()?;
    assert_eq!(model.get_current_time()?, 1.0);
    Ok(())
}

#[test]
fn invalid_names_grids_and_unsupported_queries_return_errors() -> BmiResult<()> {
    let model = initialized();
    assert!(model.get_var_type("unknown").is_err());
    assert!(model.get_value_ptr("unknown").is_err());
    assert!(model.get_grid_shape(-1).is_err());
    assert!(model.get_grid_shape(1).is_err());
    let error = model.get_grid_edge_count(0).unwrap_err();
    assert_eq!(
        error.downcast_ref::<std::io::Error>().unwrap().kind(),
        std::io::ErrorKind::Unsupported
    );
    assert!(model.get_grid_face_count(0).is_err());
    assert!(model.get_grid_x(0).is_err());
    assert!(model.get_grid_y(0).is_err());
    assert!(model.get_grid_z(0).is_err());
    assert!(model.get_grid_edge_nodes(0).is_err());
    assert!(model.get_grid_face_edges(0).is_err());
    assert!(model.get_grid_face_nodes(0).is_err());
    assert!(model.get_grid_nodes_per_face(0).is_err());
    Ok(())
}
