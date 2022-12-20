use std::{path::PathBuf, process::Command, sync::Arc};

use spdlog::{
    formatter::{pattern, PatternFormatter},
    sink::FileSink,
    Logger,
};

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
enum CrashType {
    Panic,
    NullPtrDeref,
    CAbort,
}

#[test]
fn test_flush_on_crash() {
    if let Ok(crash_type_env) = std::env::var("CRASH_TYPE") {
        let crash_type: CrashType = unsafe { std::mem::transmute(crash_type_env.parse::<u8>().unwrap()) };

        setup_global_logger();

        spdlog::info!("hello");

        match crash_type {
            CrashType::Panic => panic!("panic delibrately"),
            CrashType::NullPtrDeref => {
                let null_ptr: *const i32 = std::ptr::null();
                unsafe { std::ptr::read_volatile(null_ptr); }
            }
            CrashType::CAbort => {
                extern "C" { fn abort(); }
                unsafe { abort(); }
            }
        }
    }

    fn test_crash_type(crash_type: CrashType) {
        let log_file_path = get_log_file_path();
        std::fs::remove_file(&log_file_path).ok();

        // Launch the victim child process and inspect the log file after it finishes.
        Command::new(std::env::current_exe().unwrap())
            .args(std::env::args().skip(1))
            .env("CRASH_TYPE", (crash_type as u8).to_string())
            .spawn()
            .unwrap()
            .wait()
            .unwrap();

        let log_file_content = std::fs::read_to_string(&log_file_path).unwrap();
        assert_eq!(log_file_content.trim(), "hello", "crash type {:?} failed", crash_type);
    }

    test_crash_type(CrashType::Panic);
    test_crash_type(CrashType::NullPtrDeref);
    test_crash_type(CrashType::CAbort);
}

fn setup_global_logger() {
    let formatter = PatternFormatter::new(pattern!("{payload}"));
    let sink = FileSink::builder()
        .path(get_log_file_path())
        .truncate(true)
        .formatter(Box::new(formatter))
        .build()
        .unwrap();
    let logger = Logger::builder().sink(Arc::new(sink)).build().unwrap();

    spdlog::set_default_logger(Arc::new(logger));
}

fn get_log_file_path() -> PathBuf {
    const LOG_FILE_NAME: &'static str = "./test-crash-log.txt";

    let mut ret = PathBuf::from(env!("CARGO_TARGET_TMPDIR"));
    ret.push(LOG_FILE_NAME);
    ret
}
