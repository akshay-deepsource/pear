use std::{
    env, fs,
    path::{Path, PathBuf},
    str,
};

use serde_json::{from_str, Value};

fn main() {
    _main();
}

fn _main() -> Option<()> {
    let mut args = env::args();
    let first = args.nth(1).unwrap();
    let components = parse(&first)?;
    let loaded = load(components.as_slice())?;
    let fig_command = fig_command(&loaded)?;

    dbg!(fig_command);

    Some(())
}

#[derive(Debug)]
struct FigCommand {
    cmd: String,
    options: Vec<CmdOption>,
    arguments: Vec<CmdArgument>,
    subcommand: Option<Subcommand>,
}

#[derive(Debug)]
struct CmdOption {
    name: Vec<String>,
    description: String,
}

#[derive(Debug)]
struct CmdArgument {
    name: String,
    optional: bool,
    variadic: bool,
    template: Vec<Template>,
}

#[derive(Debug)]
enum Template {
    Files,
    Folders,
}

#[derive(Debug)]
struct Subcommand {
    name: String,
    description: String,
}

// returns a list of components
fn parse(input: &str) -> Option<Vec<String>> {
    let chars = input.chars();
    let mut components = Vec::new();
    let mut current_component = Vec::new();
    let mut in_quotes = false;

    for c in chars {
        match c {
            '"' if in_quotes => {
                in_quotes = false;
                current_component.push('"');
            }
            '"' if !in_quotes => {
                in_quotes = true;
                current_component.push('"');
            }
            c if in_quotes => {
                current_component.push(c);
            }
            ' ' => {
                let component = current_component.drain(..).collect::<String>();
                components.push(component);
            }
            _ => current_component.push(c),
        }
    }
    let component = current_component.drain(..).collect::<String>();
    components.push(component);

    Some(components)
}

fn load(components: &[String]) -> Option<Value> {
    let cmd_name = components.first()?;
    let path = ["defs", &cmd_name]
        .iter()
        .collect::<PathBuf>()
        .with_extension("json");
    let cmd_file = fs::read_to_string(path).ok()?;
    serde_json::from_str(&cmd_file).ok()
}

fn fig_command(value: &Value) -> Option<FigCommand> {
    let cmd = get_str(value, "name")?;
    let arguments = get_vec(value, "args")?
        .iter()
        .map(|a| get_argument(a))
        .flatten()
        .collect::<Vec<_>>();
    let options = get_vec(value, "options")?
        .iter()
        .map(|a| get_option(a))
        .flatten()
        .collect::<Vec<_>>();
    Some(FigCommand {
        cmd,
        options,
        arguments,
        subcommand: None,
    })
}

fn get_option(value: &Value) -> Option<CmdOption> {
    let name = value
        .get("name")
        .map(|v| match v {
            Value::String(s) => Some(vec![s.to_owned()]),
            Value::Array(v) => Some(
                v.iter()
                    .map(|s| s.as_str().unwrap().to_owned())
                    .collect::<Vec<_>>(),
            ),
            _ => None,
        })
        .flatten()?;
    let description = get_str(value, "description")?;
    Some(CmdOption { name, description })
}

fn get_argument(value: &Value) -> Option<CmdArgument> {
    let name = get_str(value, "name")?;
    let optional = get_bool(value, "isOptional").unwrap_or(false);
    let variadic = get_bool(value, "isVariadic").unwrap_or(false);
    let template = get_vec(value, "template")
        .unwrap_or_default()
        .iter()
        .map(|v| get_template(v.as_str().unwrap()))
        .flatten()
        .collect::<Vec<_>>();
    Some(CmdArgument {
        name,
        optional,
        variadic,
        template,
    })
}

fn get_template(value: &str) -> Option<Template> {
    match value {
        "filepaths" => Some(Template::Files),
        "folders" => Some(Template::Folders),
        _ => None,
    }
}

fn get_str(value: &Value, id: &str) -> Option<String> {
    value.get(id)?.as_str().map(|s| s.to_owned())
}

fn get_bool(value: &Value, id: &str) -> Option<bool> {
    value.get(id)?.as_bool()
}

fn get_vec(value: &Value, id: &str) -> Option<Vec<Value>> {
    value.get(id)?.as_array().map(|v| v.to_owned())
}
