mod test_stuff;
use test_stuff::*;

#[test]
fn sm_to_osu_test() {
    parse_and_convert!(
        sm_to_osu,
        "./tests/Maps/etterna/Kil_ChineseTea/ct.sm",
        parse::from_sm,
        write::to_osu,
        true
    );
}

#[test]
fn qua_to_osu_test() {
    parse_and_convert!(
        qua_to_osu,
        "./tests/Maps/quaver/2366_177_NewNonBiyori/19248.qua",
        parse::from_qua,
        write::to_osu,
        true
    );
}
