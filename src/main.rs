use std::{
    env,
    error::Error,
    fs,
    io::Write,
    path::{Path, PathBuf},
    process, str,
};

use serde_json::{from_str, Map, Value};

use lazy_static::lazy_static;

lazy_static! {
    static ref DEFS: Value = from_str(include_str!("../defs.json")).unwrap();
}

fn main() {
    _main();
}

fn _main() -> Option<()> {
    let mut args: Vec<_> = env::args().collect();
    let first = args.get(1)?;
    let components = parse(&first)?;
    let loaded = load(components.as_slice())?;
    let fig_command = fig_command(loaded)?;

    let res = call_fzf(first.clone(), dbg!(fig_command)).ok()?;

    println!("{} {}", first, res);

    Some(())
}

#[derive(Debug, Clone)]
struct FigCommand {
    cmd: &'static str,
    options: Vec<CmdOption>,
    arguments: Vec<CmdArgument>,
    subcommands: Vec<Subcommand>,
}

#[derive(Debug, Clone)]
struct CmdOption {
    name: Vec<&'static str>,
    description: Option<&'static str>,
}

#[derive(Debug, Clone)]
struct CmdArgument {
    name: &'static str,
    optional: bool,
    variadic: bool,
    template: Vec<Template>,
    suggestions: Vec<&'static str>,
}

#[derive(Debug, Clone)]
enum Template {
    Files,
    Folders,
}

#[derive(Debug, Clone)]
struct Subcommand {
    name: &'static str,
    description: Option<&'static str>,
    options: Vec<CmdOption>,
    arguments: Vec<CmdArgument>,
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

fn load(components: &[String]) -> Option<&'static Value> {
    let cmd_name = components.first()?;
    DEFS.as_object()?.get(cmd_name)
}

fn fig_command(value: &'static Value) -> Option<FigCommand> {
    let cmd = get_str(value, "name")?;
    let arguments = get_vec(value, "args")
        .iter()
        .cloned()
        .flatten()
        .map(|a| get_argument(a))
        .flatten()
        .collect::<Vec<_>>();
    let options = get_vec(value, "options")
        .iter()
        .cloned()
        .flatten()
        .map(|a| get_option(a))
        .flatten()
        .collect::<Vec<_>>();

    let subcommands = get_vec(value, "subcommands")
        .iter()
        .cloned()
        .flatten()
        .filter_map(|a| get_subcommand(a))
        .collect::<Vec<_>>();

    Some(FigCommand {
        cmd,
        options,
        arguments,
        subcommands,
    })
}

fn call_fzf(cmd: String, def: FigCommand) -> std::io::Result<String> {
    let options_iter = def
        .options
        .iter()
        .map(|it| (it.name.clone(), it.description));

    let subcommands_iter = def
        .subcommands
        .iter()
        .map(|it| (vec![it.name], it.description));

    let completions = options_iter.chain(subcommands_iter).collect::<Vec<_>>();

    let c_str = completions
        .iter()
        .cloned()
        .flat_map(|(name, desc)| name)
        .collect::<Vec<_>>()
        .join("\n");

    let mut proc = process::Command::new("fzf")
        .stdin(process::Stdio::piped())
        .stdout(process::Stdio::piped())
        .spawn()?;

    let mut stdin = proc.stdin.take().unwrap();

    stdin.write(c_str.as_bytes())?;

    let out = proc.wait_with_output()?;

    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

fn get_option(value: &'static Value) -> Option<CmdOption> {
    let name = value
        .get("name")
        .map(|v| match v {
            Value::String(s) => Some(vec![s.as_str()]),
            Value::Array(v) => Some(v.iter().map(|s| s.as_str().unwrap()).collect::<Vec<_>>()),
            _ => None,
        })
        .flatten()?;
    let description = get_str(value, "description");
    Some(CmdOption { name, description })
}

fn get_argument(value: &'static Value) -> Option<CmdArgument> {
    let name = get_str(value, "name")?;
    let optional = get_bool(value, "isOptional").unwrap_or(false);
    let variadic = get_bool(value, "isVariadic").unwrap_or(false);
    let template = get_vec(value, "template")
        .map(|vec| {
            vec.iter()
                .map(|v| get_template(v.as_str().unwrap()))
                .flatten()
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let suggestions = get_vec(value, "suggestions")
        .map(|vec| vec.iter().map(|v| v.as_str()).flatten().collect::<Vec<_>>())
        .unwrap_or_default();

    Some(CmdArgument {
        name,
        optional,
        variadic,
        template,
        suggestions,
    })
}

fn get_subcommand(value: &'static Value) -> Option<Subcommand> {
    let name = get_str(value, "name")?;
    let description = get_str(value, "description");
    let options = get_vec(value, "options")
        .iter()
        .cloned()
        .flatten()
        .filter_map(|opt| get_option(opt))
        .collect::<Vec<_>>();
    let arguments = get_vec(value, "args")
        .iter()
        .cloned()
        .flatten()
        .filter_map(|arg| get_argument(arg))
        .collect::<Vec<_>>();

    Some(Subcommand {
        name,
        description,
        options,
        arguments,
    })
}

fn get_template(value: &str) -> Option<Template> {
    match value {
        "filepaths" => Some(Template::Files),
        "folders" => Some(Template::Folders),
        _ => None,
    }
}

fn get_str(value: &'static Value, id: &str) -> Option<&'static str> {
    value.get(id)?.as_str()
}

fn get_bool(value: &'static Value, id: &str) -> Option<bool> {
    value.get(id)?.as_bool()
}

fn get_vec(value: &'static Value, id: &str) -> Option<&'static Vec<Value>> {
    value.get(id)?.as_array()
}

fn get_map(value: &'static Value, id: &str) -> Option<&'static Map<String, Value>> {
    value.get(id)?.as_object()
}
