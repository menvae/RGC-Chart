mod test_stuff;
use test_stuff::*;

#[test]
fn osu_to_sm_test() {
    parse_and_convert!(
        osu_to_sm,
        "./tests/Maps/osu/1888601_LunaticEyes/COOL&CREATE - Lunatic Eyes ~ Invisible Full Moon (Cut Ver.) (TheFunk) [Blood Moon].osu",
        parse::from_osu,
        write::to_sm,
        true
    );
}

#[test]
fn qua_to_sm_test() {
    parse_and_convert!(
        qua_to_sm,
        "./tests/Maps/quaver/4548_886_Ziqqurat/34785.qua",
        parse::from_qua,
        write::to_sm,
        true
    );
}