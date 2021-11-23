mod utils;
use utils::*;

use std::{env, io::Write, process, str};

use lazy_static::lazy_static;
use serde_json::{from_str, Value};

lazy_static! {
    static ref DEFS: Value = from_str(include_str!("../defs.json")).unwrap();
}

fn main() {
    _main();
}

fn _main() -> Option<()> {
    let args: Vec<_> = env::args().collect();
    let first = args.get(1)?;
    let components = parse(&first)?;
    let loaded = load(components.as_slice())?;
    let fig_command = FigCommand::new(loaded)?;

    let ctx = Context { ctx: components };

    let res = call_fzf(&ctx, fig_command).ok()?;

    println!("{} {}", first, res);

    Some(())
}

struct Context {
    ctx: Vec<String>,
}

struct Candidate {
    name: String,
    preview: String,
}

enum ContextKind<'a> {
    InSubcommand,
    InArgument,
    InOption(&'a str),
    NoContext,
}

#[derive(Debug, Clone)]
struct FigCommand {
    cmd: &'static str,
    options: Vec<CmdOption>,
    arguments: Vec<CmdArgument>,
    subcommands: Vec<Subcommand>,
}

impl FigCommand {
    fn new(value: &'static Value) -> Option<Self> {
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

    fn context_kind<'a>(&self, ctx: &'a Context) -> ContextKind<'a> {
        let last = ctx.ctx.last();
        if let Some(last) = last {
            // user typed a space, determine context from non-empty last component
            if last.is_empty() {
                // find last non-empty component
                let non_empty_last = ctx.ctx.iter().rev().find(|c| !c.is_empty());
                if let Some(non_empty_last) = non_empty_last {
                    if self
                        .subcommands
                        .iter()
                        .any(|cmd| cmd.name == non_empty_last)
                    {
                        return ContextKind::InSubcommand;
                    } else if self
                        .options
                        .iter()
                        .any(|opt| opt.name.iter().any(|n| n == non_empty_last))
                    {
                        return ContextKind::InOption(non_empty_last);
                    } else if self.arguments.iter().any(|cmd| cmd.name == non_empty_last) {
                        return ContextKind::InArgument;
                    }
                }
                return ContextKind::NoContext;
            }
            // user typed some stuff, determine context from typed text
            // TODO
        }
        return ContextKind::NoContext;
    }

    fn completions(&self, ctx: &Context) -> Vec<Candidate> {
        let context_kind = self.context_kind(&ctx);
        match context_kind {
            ContextKind::InArgument => {
                vec![]
            }
            ContextKind::InOption(o) => {
                if let Some(found_opt) = self
                    .options
                    .iter()
                    .find(|opt| opt.name.iter().any(|n| *n == o))
                {
                    found_opt
                        .arguments
                        .iter()
                        .map(|a| Candidate {
                            name: a.name.to_owned(),
                            preview: String::new(),
                        })
                        .collect()
                } else {
                    vec![]
                }
            }
            // suggest everything
            _ => {
                let options_iter = self.options.iter().map(|it| it.name.clone());

                let subcommands_iter = self.subcommands.iter().map(|it| vec![it.name]);

                let completions = options_iter
                    .chain(subcommands_iter)
                    .flatten()
                    .map(|s| Candidate {
                        name: s.to_owned(),
                        preview: String::new(),
                    })
                    .collect::<Vec<_>>();

                completions
            }
        }
    }
}

#[derive(Debug, Clone)]
struct CmdOption {
    name: Vec<&'static str>,
    description: Option<&'static str>,
    arguments: Vec<CmdArgument>,
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

fn call_fzf(ctx: &Context, def: FigCommand) -> std::io::Result<String> {
    // let options_iter = def
    //     .options
    //     .iter()
    //     .map(|it| (it.name.clone(), it.description));

    // let subcommands_iter = def
    //     .subcommands
    //     .iter()
    //     .map(|it| (vec![it.name], it.description));

    // let completions = options_iter.chain(subcommands_iter).collect::<Vec<_>>();

    // let c_str = completions
    //     .iter()
    //     .cloned()
    //     .flat_map(|(name, desc)| name)
    //     .collect::<Vec<_>>()
    //     .join("\n");

    let c_str = def
        .completions(&ctx)
        .iter()
        .map(|f| f.name.clone())
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
    let arguments = get_vec(value, "args")
        .iter()
        .cloned()
        .flatten()
        .filter_map(|arg| get_argument(arg))
        .collect::<Vec<_>>();

    Some(CmdOption {
        name,
        description,
        arguments,
    })
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
