use std::path::PathBuf;

use arma_bench::{CompareResult, ExecuteResult};
use arma_rs::{arma, Extension, Value};

#[arma]
fn init() -> Extension {
    Extension::build()
        .command("execute", execute)
        .command("compare", compare)
        .finish()
}

fn execute(id: String, data: (f64, u32), value: Value) {
    {
        let mut out = std::fs::File::create(
            PathBuf::from("/tmp/arma_bench")
                .join(&id)
                .join("execute.txt"),
        )
        .unwrap();
        let data = ExecuteResult {
            time: data.0,
            iter: data.1,
            ret: value,
        };
        serde_json::to_writer(&mut out, &data).unwrap();
    }
    std::process::exit(0);
}

fn compare(id: String, data: Vec<(String, (f64, u32), Value)>) {
    {
        let mut out = std::fs::File::create(
            PathBuf::from("/tmp/arma_bench")
                .join(&id)
                .join("compare.txt"),
        )
        .unwrap();
        let data = data
            .into_iter()
            .map(|(id, (time, iter), ret)| CompareResult {
                id: id.parse().unwrap(),
                time,
                iter,
                ret,
            })
            .collect::<Vec<_>>();
        serde_json::to_writer(&mut out, &data).unwrap();
    }
    std::process::exit(0);
}
