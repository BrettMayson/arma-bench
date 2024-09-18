use std::{io::Write, path::PathBuf};

use arma_bench::{CompareResult, ExecuteResult};
use arma_rs::{arma, Extension, Value};

#[arma]
fn init() -> Extension {
    Extension::build()
        .command("timeout", timeout)
        .command("execute", execute)
        .command("compare", compare)
        .command("die", die)
        .finish()
}

#[allow(clippy::needless_pass_by_value)]
fn timeout(id: String, time: u64) {
    std::thread::sleep(std::time::Duration::from_secs(time));
    // create a file to indicate the timeout, write the time
    std::fs::File::create(
        PathBuf::from("/tmp/arma_bench")
            .join(&id)
            .join("timeout.txt"),
    )
    .expect("Failed to create timeout.txt")
    .write_all(time.to_string().as_bytes())
    .expect("Failed to write timeout.txt");
    die();
}

#[allow(clippy::needless_pass_by_value)]
fn execute(id: String, data: (f64, u32), value: Value) {
    {
        let mut out = std::fs::File::create(
            PathBuf::from("/tmp/arma_bench")
                .join(&id)
                .join("execute.txt"),
        )
        .expect("Failed to create execute.txt");
        let data = ExecuteResult {
            time: data.0,
            iter: data.1,
            ret: value,
        };
        serde_json::to_writer(&mut out, &data).expect("Failed to write execute.txt");
    }
}

#[allow(clippy::needless_pass_by_value)]
fn compare(id: String, data: Vec<(String, (f64, u32), Value)>) {
    {
        let mut out = std::fs::File::create(
            PathBuf::from("/tmp/arma_bench")
                .join(&id)
                .join("compare.txt"),
        )
        .expect("Failed to create compare.txt");
        let data = data
            .into_iter()
            .map(|(id, (time, iter), ret)| CompareResult {
                id: id.parse().expect("Failed to parse ID"),
                time,
                iter,
                ret,
            })
            .collect::<Vec<_>>();
        serde_json::to_writer(&mut out, &data).expect("Failed to write compare.txt");
    }
}

fn die() {
    std::process::exit(0);
}
