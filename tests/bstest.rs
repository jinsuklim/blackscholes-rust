use blackscholes::OptionInputs;

fn inputs_call_otm() -> OptionInputs {
    OptionInputs::new(true, 100.0, 110.0, 0.05, 0.05, 20.0 / 365.25)
}

fn inputs_call_itm() -> OptionInputs {
    OptionInputs::new(true, 100.0, 90.0, 0.05, 0.05, 20.0 / 365.25)
}

fn inputs_put_otm() -> OptionInputs {
    OptionInputs::new(false, 100.0, 90.0, 0.05, 0.05, 20.0 / 365.25)
}

fn inputs_put_itm() -> OptionInputs {
    OptionInputs::new(false, 100.0, 110.0, 0.05, 0.05, 20.0 / 365.25)
}

#[test]
fn price_call_otm() {
    assert!((inputs_call_otm().with_implied_vol(0.2).price() - 0.0376).abs() < 0.001);
}
#[test]
fn price_call_itm() {
    assert!((inputs_call_itm().with_implied_vol(0.2).price() - 9.9913).abs() < 0.001);
}

#[test]
fn price_put_otm() {
    assert!((inputs_put_otm().with_implied_vol(0.2).price() - 0.01867).abs() < 0.001);
}
#[test]
fn price_put_itm() {
    assert!((inputs_put_itm().with_implied_vol(0.2).price() - 10.0103).abs() < 0.001);
}
