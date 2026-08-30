use serde::Deserialize;
use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::process;

#[derive(Deserialize)]
struct JadxProject {
    #[serde(rename = "codeData")]
    code_data: Option<CodeData>,
}

#[derive(Deserialize)]
struct CodeData {
    renames: Option<Vec<RenameItem>>,
}

#[derive(Deserialize)]
struct RenameItem {
    #[serde(rename = "nodeRef")]
    node_ref: Option<NodeRef>,
    #[serde(rename = "newName")]
    new_name: Option<String>,
}

#[derive(Deserialize)]
struct NodeRef {
    #[serde(rename = "refType")]
    ref_type: Option<String>,
    #[serde(rename = "declaringClass")]
    declaring_class: Option<String>,
    #[serde(rename = "shortId")]
    short_id: Option<String>,
}

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <input.jadx> <output.jobf>", args[0]);
        process::exit(1);
    }

    let input_path = &args[1];
    let output_path = &args[2];

    let content = fs::read_to_string(input_path).unwrap_or_else(|err| {
        eprintln!("Error reading file '{input_path}': {err}");
        process::exit(1);
    });

    let project: JadxProject = serde_json::from_str(&content).unwrap_or_else(|err| {
        eprintln!("Error parsing JSON: {err}");
        process::exit(1);
    });

    let renames = project
        .code_data
        .and_then(|cd| cd.renames)
        .unwrap_or_default();

    let mut jobf_lines = Vec::new();

    for item in renames {
        let new_name = match item.new_name {
            Some(name) if !name.is_empty() => name,
            _ => continue,
        };

        if let Some(node) = item.node_ref {
            let ref_type = node.ref_type.as_deref().unwrap_or_default();
            let declaring_class = node.declaring_class.as_deref().unwrap_or_default();
            let short_id = node.short_id.as_deref().unwrap_or_default();

            let line = match ref_type {
                "CLASS" => format!("c {declaring_class} = {new_name}"),
                "FIELD" => format!("f {declaring_class}.{short_id} = {new_name}"),
                "METHOD" => format!("m {declaring_class}.{short_id} = {new_name}"),
                "PARAM" => format!("p {declaring_class}.{short_id} = {new_name}"),
                _ => continue,
            };

            jobf_lines.push(line);
        }
    }

    let mut out_file = File::create(output_path).unwrap_or_else(|err| {
        eprintln!("Error creating output file '{output_path}': {err}");
        process::exit(1);
    });

    let count = jobf_lines.len();
    for line in jobf_lines {
        if let Err(err) = writeln!(out_file, "{line}") {
            eprintln!("Error writing to file: {err}");
            process::exit(1);
        }
    }

    println!("Successfully extracted {count} renames into '{output_path}'");
}
