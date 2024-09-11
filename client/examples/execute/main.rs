use arma_bench::{Client, ExecuteResult, ServerConfig};

fn main() {
    let client = Client::connect("localhost", ServerConfig::default()).unwrap();
    let ExecuteResult { time, iter, ret } = client
        .execute("private _a = 1; private _b = 2; _a + _b")
        .unwrap();
    println!("Time: {time} ms, Iterations: {iter}");
    println!("Result: {ret}");
}
