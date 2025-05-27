mod test_stuff;
use test_stuff::*;

#[test]
fn sm_to_osu_test() {
    parse_and_convert!(
        sm_to_osu,
        "./tests/Maps/etterna/Kil_ChineseTea/ct.sm",
        chart_convertion::parse::parse_sm,
        chart_convertion::convert::convert_to_osu,
        true
    );
}
