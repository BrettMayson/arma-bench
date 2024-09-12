use std::{io::Cursor, path::PathBuf};

use arma_bench::Request;
use hemtt_pbo::WritablePbo;
use uuid::Uuid;

pub struct BuiltRequest {
    pub path: PathBuf,
}

impl Drop for BuiltRequest {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.path).expect("Failed to remove temp directory");
    }
}

pub fn build(request: &Request) -> BuiltRequest {
    let id = Uuid::new_v4().to_string();
    let path = std::env::temp_dir()
        .join("arma_bench")
        .join(&id)
        .join("addons");
    std::fs::create_dir_all(&path).expect("Failed to create temp directory");
    let config = r#"
    class CfgPatches {
        class TAB {
            units[] = {};
            weapons[] = {};
            requiredVersion = 1.0;
            requiredAddons[] = {};
        };
    };

    class CfgFunctions {
        class TAB {
            class Bench {
                class Bootstrap {
                    file = "\tab\bootstrap.sqf";
                    preStart = 1;
                };
            };
        };
    };"#;
    let mut file = std::fs::File::create(path.join("execute.pbo")).expect("Failed to create PBO");
    let mut pbo = WritablePbo::new();
    pbo.add_property("prefix", "tab");
    pbo.add_file("config.cpp", Cursor::new(config.as_bytes()))
        .expect("Failed to add config.cpp");
    match request {
        Request::Execute(content) => {
            let bootstrap = format!(
                r#"
            private _out = diag_codePerformance [{{
                {content}
            }}];
            private _ret = call {{ {content} }};
            "tab" callExtension ["execute", ["{id}", _out, _ret]];
            "#
            );
            pbo.add_file("bootstrap.sqf", Cursor::new(bootstrap.as_bytes()))
                .expect("Failed to add bootstrap.sqf");
            pbo.write(&mut file, true).expect("Failed to write PBO");
        }
        Request::Compare(files) => {
            let mut ids = Vec::new();
            for file in files {
                ids.push(file.id.to_string());
                let filename = format!("{}.{}", file.id, if file.sqfc { "sqfc" } else { "sqf" });
                pbo.add_file(&filename, Cursor::new(&file.content))
                    .expect("Failed to add file");
            }
            let bootstrap = format!(
                r#"
            private _out = [];
            {{
                private _code = compileScript [format["\tab\%1.sqf", _x]];
                private _ret = [_x];
                _ret pushBack diag_codePerformance [_code];
                _ret pushBack call _code;
                _out pushBack _ret;
            }} forEach ["{}"];
            "tab" callExtension ["compare", ["{}", _out]];
            "#,
                ids.join("\", \""),
                id
            );
            pbo.add_file("bootstrap.sqf", Cursor::new(bootstrap.as_bytes()))
                .expect("Failed to add bootstrap.sqf");
            pbo.write(&mut file, true).expect("Failed to write PBO");
        }
    }
    BuiltRequest {
        path: path.parent().expect("Failed to get parent").to_path_buf(),
    }
}
