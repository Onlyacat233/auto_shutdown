use std::{collections::HashMap, thread, time::Duration};
use clokwerk::{Scheduler, TimeUnits, Job};
use config::Config;
use native_dialog::{DialogBuilder};
use rust_i18n::t;
use system_shutdown::shutdown;
use chrono::{DateTime, Datelike, Local, Weekday};

rust_i18n::i18n!("locales");

fn alart(title: String, message: String) -> Result<bool, native_dialog::Error> {
    DialogBuilder::message()
        .set_level(native_dialog::MessageLevel::Warning)
        .set_title(title)
        .set_text(message)
        .confirm()
        .show()
}

fn error(message: String) {
    let _ = DialogBuilder::message()
        .set_level(native_dialog::MessageLevel::Error)
        .set_title(t!("error.title"))
        .set_text(message)
        .confirm()
        .show();
}

#[cfg(not(feature = "do_not_really_shutdown"))]
fn shutdown_with_cooldown(wait_time: u64) {
    println!("a");
    use std::sync::{Arc, Mutex};
    let res_arc = Arc::new(Mutex::new(1));
    let t_res_arc = Arc::clone(&res_arc);

    thread::spawn(move || {
        let mut i = 0;
        loop {
            println!("{}", 10);
            if i < wait_time {
                i += 1;
            } else {
                break;
            }
            if *t_res_arc.lock().unwrap() == 0 {
                return;
            }
            thread::sleep(Duration::from_secs(1));
        }
        match shutdown() {
            Ok(_) => (),
            Err(e) => error(format!("{}\n {}", "error.type.failed_to_shutdown", e)),
        }
    });
    let res = alart(t!("on_shutdown.title").to_string(), t!("on_shutdown.content").to_string()).unwrap();
    if !res {
        *res_arc.lock().unwrap() = 0;
    }
}

#[cfg(feature = "do_not_really_shutdown")]
fn shutdown_with_cooldown(wait_time: u64) {
    println!("a");
    use std::sync::{Arc, Mutex};
    let res_arc = Arc::new(Mutex::new(1));
    let t_res_arc = Arc::clone(&res_arc);

    thread::spawn(move || {
        let mut i = 0;
        loop {
            println!("{}", 10);
            if i < wait_time*10 {
                i += 1;
            } else {
                break;
            }
            if *t_res_arc.lock().unwrap() == 0 {
                return;
            }
            thread::sleep(Duration::from_millis(100));
        }
        println!("Shutdown !");
    });
    let res = alart(t!("on_shutdown.title").to_string(), t!("on_shutdown.content").to_string()).unwrap();
    if !res {
        *res_arc.lock().unwrap() = 0;
    }
}

fn get_settings() -> HashMap<String, String> {
    Config::builder()
        .add_source(config::File::with_name("config.yml"))
        .build()
        .unwrap()
        .try_deserialize::<HashMap<String, String>>()
        .unwrap()
}

fn check_date() {
    let now: DateTime<Local> = Local::now();
    if now.month() == 6 && now.day() >=7 && (now.day() <= 10)
        | (now.weekday() == Weekday::Sat) | (now.weekday() == Weekday::Sun) {
            shutdown().unwrap();
    }
}

fn main() {
    check_date();

    let settings = get_settings();
    // shutdown_with_cooldown(settings.get("wait_time").unwrap().parse::<u64>().unwrap());

    let mut sched: Scheduler = Scheduler::new();
    sched.every(1.day()).at(settings.get("shutdown_time").unwrap()).run(move || shutdown_with_cooldown(settings.get("wait_time").unwrap().parse::<u64>().unwrap()));

    let thread_handle = sched.watch_thread(Duration::from_millis(100));

    thread::sleep(Duration::from_secs(std::u64::MAX));

    thread_handle.stop();
}