mod test_stuff;
use test_stuff::*;

#[test]
fn osu_to_sm_test() {
    parse_and_convert!(
        osu_to_sm,
        "./tests/Maps/osu/360565_HatsuneMikuNoShoushitsu/cosMo@BousouP feat. Hatsune Miku - Hatsune Miku no Shoushitsu (juankristal) [Disappearance].osu",
        parse::from_osu,
        write::to_sm,
        true
    );
}