// Local dates in time zones that change their UTC offset at local 1900-01-01T00:00. CI runs in
// UTC, so each test writes a TZif file (RFC 8536), points `TZ` at it, and re-runs this test binary
// as a child. Setting `TZ` in this process would race the other tests.

use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

use time::{Month, UtcOffset};

use crate::helper_functions::tests::*;
use crate::helper_functions::{as_datetime_local, as_double_local};

// tells the child which kind of zone it runs in
const ZONE_KIND: &str = "DXF_TEST_LOCAL_TIME_ZONE_KIND";
const EXPECTED_OFFSET: &str = "DXF_TEST_LOCAL_TIME_EXPECTED_OFFSET";

// printed by the child once its assertions pass, so a child that ran no test can't pass
const CHILD_PASSED: &str = "local time child passed";

// 1900-01-01T00:00:00Z as a Unix time
const UTC_1900: i64 = -2_208_988_800;

#[test]
fn ambiguous_local_epoch() {
    // `Europe/Stockholm` in the tz database's backzone data: the offset drops from +01:00:14 to
    // +01:00 at local midnight, so the last 14 seconds of 1899, and midnight, happen twice
    run_child(
        "ambiguous",
        Zone {
            before: ("SET", 3_614),
            at: UTC_1900 - 3_614,
            after: ("CET", 3_600),
            footer: "CET-1",
        },
    );
}

#[test]
fn ambiguous_local_epoch_west_of_utc() {
    // the same kind of change west of UTC, where UTC midnight comes before local midnight: the
    // offset drops from -04:56:16 to -05:00, so midnight happens twice
    run_child(
        "ambiguous",
        Zone {
            before: ("LMT", -17_776),
            at: UTC_1900 + 17_776,
            after: ("EST", -18_000),
            footer: "EST5",
        },
    );
}

#[test]
fn nonexistent_local_epoch() {
    // clocks jump from 23:30 (+01:00) to 00:30 (+02:00), so local midnight never happens
    run_child(
        "nonexistent",
        Zone {
            before: ("CET", 3_600),
            at: UTC_1900 - 5_400,
            after: ("EET", 7_200),
            footer: "EET-2",
        },
    );
}

#[test]
fn single_local_epoch() {
    // the change from `ambiguous_local_epoch` a day earlier, so local midnight happens once
    run_child(
        "single",
        Zone {
            before: ("SET", 3_614),
            at: UTC_1900 - 86_400 - 3_614,
            after: ("CET", 3_600),
            footer: "CET-1",
        },
    );
}

#[test]
#[ignore = "run by the tests above in a crafted time zone"]
fn local_time_child() {
    let kind = match std::env::var(ZONE_KIND) {
        Ok(kind) => kind,
        Err(_) => return, // run directly, without a crafted zone
    };
    let expected_offset = std::env::var(EXPECTED_OFFSET)
        .unwrap()
        .parse::<i32>()
        .unwrap();
    let expected_offset = UtcOffset::from_whole_seconds(expected_offset).unwrap();
    let midnight = as_datetime_local(0.0);
    assert_eq!(
        expected_offset,
        midnight.offset(),
        "wrong offset in the {kind} zone"
    );
    assert_eq!(
        (1900, Month::January, 1, 0, 0, 0),
        (
            midnight.year(),
            midnight.month(),
            midnight.day(),
            midnight.hour(),
            midnight.minute(),
            midnight.second()
        )
    );

    // local dates read and write as the wall-clock times they name
    // from AutoDesk spec: 2451544.91568287 = 31 December 1999, 9:58:35PM
    let date = as_datetime_local(2_451_544.915_682_87);
    assert_eq!(
        (1999, Month::December, 31, 21, 58, 35),
        (
            date.year(),
            date.month(),
            date.day(),
            date.hour(),
            date.minute(),
            date.second()
        )
    );
    assert_eq!(date, as_datetime_local(2_451_544.915_682_87));
    assert_eq!(2_451_544.915_682_870_4, as_double_local(date));

    // and so drawings load and save
    let drawing = parse_drawing(
        [
            "0",
            "SECTION",
            "2",
            "HEADER",
            "9",
            "$TDCREATE",
            "40",
            "2451544.91568287",
            "0",
            "ENDSEC",
            "0",
            "EOF",
        ]
        .join("\n")
        .as_str(),
    );
    assert_eq!(date, drawing.header.creation_date);
    assert_contains(
        &drawing,
        ["  9", "$TDCREATE", " 40", "2451544.915682870429"].join("\r\n"),
    );

    println!("{CHILD_PASSED}");
}

// re-runs this test binary as a child that runs only `local_time_child`, with `TZ` naming a file
// that holds `zone`, and fails unless the child's assertions pass
fn run_child(kind: &str, zone: Zone) {
    // tests run in parallel, so each child gets its own zone file
    static NEXT_FILE: AtomicUsize = AtomicUsize::new(0);
    let file = NEXT_FILE.fetch_add(1, Ordering::Relaxed);
    let path =
        std::env::temp_dir().join(format!("dxf-local-time-{}-{file}.tzif", std::process::id()));
    std::fs::write(&path, zone.to_tzif()).unwrap();
    let output = Command::new(std::env::current_exe().unwrap())
        .args([
            "misc_tests::local_time::local_time_child",
            "--exact",
            "--ignored",
            "--nocapture",
        ])
        .env("TZ", &path)
        .env(ZONE_KIND, kind)
        .env(EXPECTED_OFFSET, zone.after.1.to_string())
        .output()
        .unwrap();
    std::fs::remove_file(&path).unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success() && stdout.contains(CHILD_PASSED),
        "child in the {kind} zone failed ({}):\n{stdout}\n{stderr}",
        output.status
    );
}

// a time zone with one change of UTC offset
struct Zone {
    // abbreviation and UTC offset in seconds before the change
    before: (&'static str, i32),
    // Unix time of the change
    at: i64,
    // abbreviation and UTC offset in seconds from the change on
    after: (&'static str, i32),
    // POSIX TZ string for times after the change; the local offset reader requires this to match
    // `after`
    footer: &'static str,
}

impl Zone {
    fn to_tzif(&self) -> Vec<u8> {
        let mut tzif = vec![];
        // version 1 block, 32-bit times: the change predates 1901, the earliest 32-bit time, so,
        // as zic writes it, this block holds no changes, only the type in force from 1901 on
        let names = format!("{}\0", self.after.0);
        tzif_header(&mut tzif, 0, 1, names.len());
        tzif_type(&mut tzif, self.after.1, 0);
        tzif.extend_from_slice(names.as_bytes());
        // version 2 block, 64-bit times: type 0 applies before the change, type 1 from it on
        let names = format!("{}\0{}\0", self.before.0, self.after.0);
        tzif_header(&mut tzif, 1, 2, names.len());
        tzif.extend_from_slice(&self.at.to_be_bytes());
        tzif.push(1);
        tzif_type(&mut tzif, self.before.1, 0);
        tzif_type(&mut tzif, self.after.1, self.before.0.len() + 1);
        tzif.extend_from_slice(names.as_bytes());
        tzif.extend_from_slice(format!("\n{}\n", self.footer).as_bytes());
        tzif
    }
}

// a TZif header with no leap seconds and no standard/UT indicators
fn tzif_header(tzif: &mut Vec<u8>, changes: u32, types: u32, name_bytes: usize) {
    tzif.extend_from_slice(b"TZif2");
    tzif.extend_from_slice(&[0; 15]);
    // isutcnt, isstdcnt, leapcnt, timecnt, typecnt, charcnt
    for count in [0, 0, 0, changes, types, name_bytes as u32] {
        tzif.extend_from_slice(&count.to_be_bytes());
    }
}

// a TZif local time type: UTC offset, not DST, abbreviation index
fn tzif_type(tzif: &mut Vec<u8>, utc_offset: i32, name_index: usize) {
    tzif.extend_from_slice(&utc_offset.to_be_bytes());
    tzif.push(0);
    tzif.push(name_index as u8);
}
