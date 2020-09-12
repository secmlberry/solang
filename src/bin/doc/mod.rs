use handlebars::Handlebars;
use serde::Serialize;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use solang::parser::pt;
use solang::sema::ast::{ContractVariable, Namespace, Tag};
use solang::sema::contracts::visit_bases;

#[derive(Serialize)]
struct Field<'a> {
    name: &'a str,
    ty: String,
    indexed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    doc: Option<&'a str>,
}

#[derive(Serialize)]
struct StructDecl<'a> {
    #[serde(skip_serializing)]
    loc: pt::Loc,
    name: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    contract: Option<&'a str>,
    field: Vec<Field<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    notice: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    author: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    dev: Option<&'a str>,
}

#[derive(Serialize)]
struct EventDecl<'a> {
    #[serde(skip_serializing)]
    loc: pt::Loc,
    name: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    contract: Option<&'a str>,
    anonymous: bool,
    field: Vec<Field<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    notice: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    author: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    dev: Option<&'a str>,
}

#[derive(Serialize)]
struct EnumDecl<'a> {
    #[serde(skip_serializing)]
    loc: pt::Loc,
    name: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    contract: Option<&'a str>,
    field: Vec<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    notice: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    author: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    dev: Option<&'a str>,
}

#[derive(Serialize)]
struct Contract<'a> {
    #[serde(skip_serializing)]
    loc: pt::Loc,
    name: &'a str,
    ty: String,
    variables: Vec<Variable<'a>>,
    base_variables: Vec<Variable<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    notice: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    author: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    dev: Option<&'a str>,
}

#[derive(Serialize)]
struct Variable<'a> {
    name: &'a str,
    constant: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    base_contract: Option<&'a str>,
    ty: String,
    visibility: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    notice: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    author: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    dev: Option<&'a str>,
}

#[derive(Serialize)]
struct Top<'a> {
    contracts: Vec<Contract<'a>>,
    events: Vec<EventDecl<'a>>,
    structs: Vec<StructDecl<'a>>,
    enums: Vec<EnumDecl<'a>>,
}

fn get_tag<'a>(name: &str, tags: &'a [Tag]) -> Option<&'a str> {
    tags.iter()
        .find(|e| e.tag == name)
        .map(|e| &e.value as &str)
}

fn get_tag_no<'a>(name: &str, no: usize, tags: &'a [Tag]) -> Option<&'a str> {
    tags.iter()
        .find(|e| e.tag == name && e.no == no)
        .map(|e| &e.value as &str)
}

pub fn generate_docs(outdir: &str, files: &[Namespace], verbose: bool) {
    let mut top = Top {
        contracts: Vec::new(),
        events: Vec::new(),
        structs: Vec::new(),
        enums: Vec::new(),
    };

    for file in files {
        // events
        for event_decl in &file.events {
            if top.events.iter().any(|e| e.loc == event_decl.loc) {
                continue;
            }

            let mut field = Vec::new();

            for (i, f) in event_decl.fields.iter().enumerate() {
                field.push(Field {
                    name: &f.name,
                    ty: f.ty.to_string(&file),
                    indexed: f.indexed,
                    doc: get_tag_no("param", i, &event_decl.tags),
                });
            }

            top.events.push(EventDecl {
                name: &event_decl.name,
                contract: event_decl.contract.as_deref(),
                title: get_tag("title", &event_decl.tags),
                notice: get_tag("notice", &event_decl.tags),
                author: get_tag("author", &event_decl.tags),
                dev: get_tag("dev", &event_decl.tags),
                anonymous: event_decl.anonymous,
                loc: event_decl.loc,
                field,
            });
        }

        // structs
        for struct_decl in &file.structs {
            if top.structs.iter().any(|e| e.loc == struct_decl.loc) {
                continue;
            }

            let mut field = Vec::new();

            for (i, f) in struct_decl.fields.iter().enumerate() {
                field.push(Field {
                    name: &f.name,
                    ty: f.ty.to_string(&file),
                    indexed: false,
                    doc: get_tag_no("param", i, &struct_decl.tags),
                });
            }

            top.structs.push(StructDecl {
                name: &struct_decl.name,
                contract: struct_decl.contract.as_deref(),
                title: get_tag("title", &struct_decl.tags),
                notice: get_tag("notice", &struct_decl.tags),
                author: get_tag("author", &struct_decl.tags),
                dev: get_tag("dev", &struct_decl.tags),
                loc: struct_decl.loc,
                field,
            });
        }

        // enum
        for enum_decl in &file.enums {
            if top.enums.iter().any(|e| e.loc == enum_decl.loc) {
                continue;
            }

            let mut field: Vec<&str> = Vec::new();
            field.resize(enum_decl.values.len(), &"");
            for (value, (_, pos)) in &enum_decl.values {
                field[*pos] = &value;
            }

            top.enums.push(EnumDecl {
                name: &enum_decl.name,
                contract: enum_decl.contract.as_deref(),
                title: get_tag("title", &enum_decl.tags),
                notice: get_tag("notice", &enum_decl.tags),
                author: get_tag("author", &enum_decl.tags),
                dev: get_tag("dev", &enum_decl.tags),
                loc: enum_decl.loc,
                field,
            });
        }

        for contract_no in 0..file.contracts.len() {
            let contract = &file.contracts[contract_no];

            if top.contracts.iter().any(|e| e.loc == contract.loc) {
                continue;
            }

            fn map_var<'a>(
                file: &'a Namespace,
                base_contract: Option<&'a str>,
                var: &'a ContractVariable,
            ) -> Variable<'a> {
                Variable {
                    name: &var.name,
                    ty: var.ty.to_string(file),
                    base_contract,
                    title: get_tag("title", &var.tags),
                    notice: get_tag("notice", &var.tags),
                    author: get_tag("author", &var.tags),
                    dev: get_tag("dev", &var.tags),
                    constant: !var.is_storage(),
                    visibility: format!("{}", var.visibility),
                }
            }

            let variables = contract
                .variables
                .iter()
                .map(|var| map_var(file, None, var))
                .collect();

            let bases = visit_bases(contract_no, file);

            let mut base_variables = Vec::new();

            for base_no in bases.into_iter() {
                if contract_no == base_no {
                    continue;
                }

                let base = &file.contracts[base_no];

                for var in base
                    .variables
                    .iter()
                    .map(|var| map_var(file, Some(&base.name), var))
                {
                    base_variables.push(var);
                }
            }

            top.contracts.push(Contract {
                loc: contract.loc,
                name: &contract.name,
                ty: format!("{}", contract.ty),
                title: get_tag("title", &contract.tags),
                notice: get_tag("notice", &contract.tags),
                author: get_tag("author", &contract.tags),
                dev: get_tag("dev", &contract.tags),
                variables,
                base_variables,
            });
        }
    }

    let mut reg = Handlebars::new();

    reg.set_strict_mode(true);

    reg.register_template_string(
        "types",
        r##"<!doctype html><head><title>types</title><meta charset="utf-8"></head><body>
<h2>Contracts</h2>
{{#each contracts}}
<h3>{{ty}} {{name}}</h3>
{{#if title}}{{title}}<p>{{/if}}
{{#if notice}}{{notice}}<p>{{/if}}
{{#if dev}}Development note: {{dev}}<p>{{/if}}
{{#if author}}Author: {{author}}<p>{{/if}}
<h4>Variables</h4>
{{#each variables}}
<h5>{{#if constants}}constant{{/if}} {{ty}} {{visibility}} {{name}}</h5>
{{#if title}}{{title}}<p>{{/if}}
{{#if notice}}{{notice}}<p>{{/if}}
{{#if dev}}Development note: {{dev}}<p>{{/if}}
{{#if author}}Author: {{author}}<p>{{/if}}
{{/each}}
<h4>Inherited Variables</h4>
{{#each base_variables}}
<h5>{{#if constant}}constant{{/if}} {{ty}} {{visibility}} {{name}}</h5>
Base contract: {{base_contract}}<p>
{{#if title}}{{title}}<p>{{/if}}
{{#if notice}}{{notice}}<p>{{/if}}
{{#if dev}}Development note: {{dev}}<p>{{/if}}
{{#if author}}Author: {{author}}<p>{{/if}}
{{/each}}
{{/each}}
<h2>Events</h2>
{{#each events}}
<h3>{{#if contract}}{{contract}}.{{/if}}{{name}}</h3>
{{#if title}}{{title}}<p>{{/if}}
{{#if notice}}{{notice}}<p>{{/if}}
{{#if dev}}Development note: {{dev}}<p>{{/if}}
{{#if author}}Author: {{author}}<p>{{/if}}
Fields:<dl>
{{#each field}}
<dt><code>{{ty}} {{#if indexed}}indexed{{/if}}</code> {{name}}</dt>
{{#if doc}}<dd>{{doc}}</dd>{{/if}}
{{/each}}</dl>
Anonymous: {{#if anonymous}}true{{else}}false{{/if}}
{{/each}}
<h2>Structs</h2>
{{#each structs}}
<h3>{{#if contract}}{{contract}}.{{/if}}{{name}}</h3>
{{#if title}}{{title}}<p>{{/if}}
{{#if notice}}{{notice}}<p>{{/if}}
{{#if dev}}Development note: {{dev}}<p>{{/if}}
{{#if author}}Author: {{author}}<p>{{/if}}
Fields:<dl>
{{#each field}}
<dt><code>{{ty}}</code> {{name}}</dt>
{{#if doc}}<dd>{{doc}}</dd>{{/if}}
{{/each}}</dl>
{{/each}}
<h2>Enums</h2>
{{#each enums}}
<h3>{{#if contract}}{{contract}}.{{/if}}{{name}}</h3>
{{#if title}}{{title}}<p>{{/if}}
{{#if notice}}{{notice}}<p>{{/if}}
{{#if dev}}Development note: {{dev}}<p>{{/if}}
{{#if author}}Author: {{author}}<p>{{/if}}
Values: {{field}}
{{/each}}
</body></html>"##,
    )
    .expect("template should be good");

    let res = reg.render("types", &top).expect("template should render");

    let filename = Path::new(outdir).join("doc.html");

    if verbose {
        println!(
            "debug: writing documentation to ‘{}’",
            filename.to_string_lossy()
        );
    }

    let mut file = File::create(filename).expect("cannot create doc.html");

    file.write_all(res.as_bytes())
        .expect("should be able to write");
}
