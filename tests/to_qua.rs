mod test_stuff;
use test_stuff::*;

#[test]
fn osu_to_qua_test() {
    parse_and_convert!(
        osu_to_qua,
        "./tests/Maps/osu/360565_HatsuneMikuNoShoushitsu/cosMo@BousouP feat. Hatsune Miku - Hatsune Miku no Shoushitsu (juankristal) [Disappearance].osu",
        parse::from_osu,
        write::to_qua,
        true
    );
}

#[test]
fn sm_to_osu_test() {
    parse_and_convert!(
        sm_to_qua,
        "./tests/Maps/etterna/Kil_ChineseTea/ct.sm",
        parse::from_sm,
        write::to_qua,
        true
    );
}